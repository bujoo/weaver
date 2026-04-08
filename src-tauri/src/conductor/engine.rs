use crate::conductor::types::{ConductorConfig, ConductorDecision, ConductorEvent, ModelTier};
use crate::mqtt::state_cache::MissionStateCache;
use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::types::{
    ContentBlock, ConversationRole, Message, SpecificToolChoice, SystemContentBlock, Tool,
    ToolChoice, ToolConfiguration, ToolInputSchema, ToolSpecification,
};
use aws_sdk_bedrockruntime::Client;

/// Calls Bedrock Converse API to make mission management decisions.
pub struct ConductorEngine {
    config: ConductorConfig,
}

impl ConductorEngine {
    pub fn new(config: ConductorConfig) -> Self {
        Self { config }
    }

    /// Call Bedrock to make a decision based on aggregated events.
    pub async fn decide(
        &self,
        events: &[ConductorEvent],
        state_cache: &MissionStateCache,
        tier: ModelTier,
    ) -> Result<(ConductorDecision, u32, u32), String> {
        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(self.config.aws_region.clone()))
            .profile_name(&self.config.aws_profile)
            .load()
            .await;
        let client = Client::new(&aws_config);

        let system_prompt = self.build_system_prompt(state_cache);
        let user_message = self.format_events(events);
        let tool_schema = self.decision_tool_schema();

        let tool_spec = ToolSpecification::builder()
            .name("make_decision")
            .description("Make a session management decision based on the events.")
            .input_schema(ToolInputSchema::Json(tool_schema))
            .build()
            .map_err(|e| format!("Failed to build tool spec: {}", e))?;

        let tool_config = ToolConfiguration::builder()
            .tools(Tool::ToolSpec(tool_spec))
            .tool_choice(ToolChoice::Tool(
                SpecificToolChoice::builder()
                    .name("make_decision")
                    .build()
                    .map_err(|e| format!("Failed to build tool choice: {}", e))?,
            ))
            .build()
            .map_err(|e| format!("Failed to build tool config: {}", e))?;

        let message = Message::builder()
            .role(ConversationRole::User)
            .content(ContentBlock::Text(user_message))
            .build()
            .map_err(|e| format!("Failed to build message: {}", e))?;

        let response = client
            .converse()
            .model_id(tier.model_id())
            .system(SystemContentBlock::Text(system_prompt))
            .messages(message)
            .tool_config(tool_config)
            .send()
            .await
            .map_err(|e| format!("Bedrock Converse failed: {}", e))?;

        // Extract token usage
        let usage = response.usage();
        let input_tokens = usage.map(|u| u.input_tokens()).unwrap_or(0) as u32;
        let output_tokens = usage.map(|u| u.output_tokens()).unwrap_or(0) as u32;

        // Extract tool_use from response
        let output = response.output().ok_or("No output in response")?;
        let content = match output {
            aws_sdk_bedrockruntime::types::ConverseOutput::Message(msg) => msg.content().to_vec(),
            _ => return Err("Unexpected output type".to_string()),
        };

        let tool_block = content
            .iter()
            .find_map(|block| {
                if let ContentBlock::ToolUse(tool_use) = block {
                    if tool_use.name() == "make_decision" {
                        return Some(tool_use.input().clone());
                    }
                }
                None
            })
            .ok_or("No make_decision tool use in response")?;

        // Convert aws Document to serde_json::Value, then parse into ConductorDecision
        let json_value = document_to_json(&tool_block);
        let decision: ConductorDecision = serde_json::from_value(json_value)
            .map_err(|e| format!("Failed to parse decision: {}", e))?;

        Ok((decision, input_tokens, output_tokens))
    }

    /// Free-form chat: call Bedrock without tools, get a text response.
    pub async fn chat(
        &self,
        message: &str,
        state_cache: &MissionStateCache,
    ) -> Result<String, String> {
        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(self.config.aws_region.clone()))
            .profile_name(&self.config.aws_profile)
            .load()
            .await;
        let client = Client::new(&aws_config);

        let system_prompt = self.build_chat_prompt(state_cache);

        let msg = Message::builder()
            .role(ConversationRole::User)
            .content(ContentBlock::Text(message.to_string()))
            .build()
            .map_err(|e| format!("Failed to build message: {}", e))?;

        let response = client
            .converse()
            .model_id("us.anthropic.claude-haiku-4-5-20251001")
            .system(SystemContentBlock::Text(system_prompt))
            .messages(msg)
            // No tool_config -- free-form text response
            .send()
            .await
            .map_err(|e| format!("Bedrock Converse failed: {}", e))?;

        // Extract text from response
        let output = response.output().ok_or("No output")?;
        let content = match output {
            aws_sdk_bedrockruntime::types::ConverseOutput::Message(m) => m.content().to_vec(),
            _ => return Err("Unexpected output type".to_string()),
        };

        for block in &content {
            if let ContentBlock::Text(text) = block {
                return Ok(text.clone());
            }
        }

        Err("No text in response".to_string())
    }

    fn build_chat_prompt(&self, cache: &MissionStateCache) -> String {
        let snapshot = cache.snapshot();
        let plans = snapshot.get("plans").and_then(|v| v.as_u64()).unwrap_or(0);
        let phases = snapshot.get("phases").and_then(|v| v.as_u64()).unwrap_or(0);
        let todos = snapshot.get("todos").and_then(|v| v.as_u64()).unwrap_or(0);

        let mut phase_details = String::new();
        if let Some(plan_ids) = snapshot.get("plan_ids").and_then(|v| v.as_array()) {
            for pid in plan_ids {
                if let Some(mid) = pid.as_str() {
                    for phase in cache.get_phases_for_mission(mid) {
                        phase_details.push_str(&format!(
                            "\n- {} ({}): status={}, {}/{} done",
                            phase.name, phase.phase_id, phase.status,
                            phase.completed_count, phase.todo_count
                        ));
                    }
                }
            }
        }

        format!(
            "You are Weavy, a friendly AI dev sidekick inside ContextHub Weaver.\n\
             Answer concisely and helpfully. You have full awareness of the system state.\n\n\
             Current state: {} plans, {} phases, {} todos cached.\n\
             Phases:{}\n\n\
             The user can also use slash commands: /status, /watch, /continue, /push P1, /missions, /workspace, /help\n\
             If the user asks to do something actionable (push a phase, continue, etc.), suggest the appropriate slash command.\n\
             Keep responses short and direct.",
            plans, phases, todos,
            if phase_details.is_empty() { " none loaded".to_string() } else { phase_details }
        )
    }

    fn build_system_prompt(&self, cache: &MissionStateCache) -> String {
        let mut prompt = String::from(
            "You are Weavy, the AI conductor for ContextHub Weaver. \
             You manage Claude Code development sessions that execute mission phases.\n\n\
             Your job: analyze events from Claude Code sessions and decide what to do next.\n\n\
             You have ONE tool: make_decision. Call it with exactly one decision.\n\n\
             Decision types:\n\
             - push_next_phase: Push the next phase after current completes. Include phase_id.\n\
             - inject_context: Send guidance to a stuck Claude session.\n\
             - retry: Retry a failed phase or todo.\n\
             - skip_phase: Skip a blocked/unnecessary phase.\n\
             - escalate: Escalate to human when AI cannot resolve.\n\
             - report_status: Report status to Brain via MQTT.\n\
             - spawn_session: Start a new Claude Code session.\n\
             - kill_session: Terminate a misbehaving session.\n\
             - no_action: Nothing to do right now.\n\n\
             Be decisive. Prefer push_next_phase when a phase completes successfully.\n\
             Use escalate only when truly stuck.\n\n\
             CURRENT STATE:\n",
        );

        let snapshot = cache.snapshot();
        prompt.push_str(
            &serde_json::to_string_pretty(&snapshot).unwrap_or_else(|_| "{}".to_string()),
        );

        // Add phase details
        prompt.push_str("\n\nPHASE DETAILS:\n");
        for plan_id in snapshot
            .get("plan_ids")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
        {
            if let Some(mid) = plan_id.as_str() {
                let phases = cache.get_phases_for_mission(mid);
                for phase in &phases {
                    prompt.push_str(&format!(
                        "- {} ({}): status={}, {}/{} todos done\n",
                        phase.name,
                        phase.phase_id,
                        phase.status,
                        phase.completed_count,
                        phase.todo_count
                    ));
                }
            }
        }

        prompt
    }

    fn format_events(&self, events: &[ConductorEvent]) -> String {
        format!(
            "The following events occurred since the last decision:\n\n```json\n{}\n```\n\n\
             What should I do next?",
            serde_json::to_string_pretty(events).unwrap_or_else(|_| "[]".to_string())
        )
    }

    fn decision_tool_schema(&self) -> aws_smithy_types::Document {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": [
                        "push_next_phase", "inject_context", "retry", "skip_phase",
                        "escalate", "report_status", "spawn_session", "kill_session", "no_action"
                    ]
                },
                "mission_id": { "type": "string", "description": "Mission ID" },
                "phase_id": { "type": "string", "description": "Phase ID (e.g., P0, P1)" },
                "todo_id": { "type": "string", "description": "Todo ID if applicable" },
                "session_name": { "type": "string", "description": "tmux session name" },
                "status": { "type": "string", "description": "Status string" },
                "summary": { "type": "string", "description": "Summary text" },
                "message": { "type": "string", "description": "Message to inject or report" },
                "reason": { "type": "string", "description": "Why this decision was made" }
            },
            "required": ["action", "reason"]
        });

        json_to_document(&schema)
    }
}

/// Convert aws_smithy_types::Document to serde_json::Value
fn document_to_json(doc: &aws_smithy_types::Document) -> serde_json::Value {
    match doc {
        aws_smithy_types::Document::Null => serde_json::Value::Null,
        aws_smithy_types::Document::Bool(b) => serde_json::Value::Bool(*b),
        aws_smithy_types::Document::Number(n) => {
            match n {
                aws_smithy_types::Number::PosInt(i) => serde_json::json!(*i),
                aws_smithy_types::Number::NegInt(i) => serde_json::json!(*i),
                aws_smithy_types::Number::Float(f) => serde_json::json!(*f),
                _ => serde_json::Value::Null,
            }
        }
        aws_smithy_types::Document::String(s) => serde_json::Value::String(s.clone()),
        aws_smithy_types::Document::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(document_to_json).collect())
        }
        aws_smithy_types::Document::Object(obj) => {
            let map: serde_json::Map<String, serde_json::Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), document_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
    }
}

/// Convert serde_json::Value to aws_smithy_types::Document
fn json_to_document(value: &serde_json::Value) -> aws_smithy_types::Document {
    match value {
        serde_json::Value::Null => aws_smithy_types::Document::Null,
        serde_json::Value::Bool(b) => aws_smithy_types::Document::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                aws_smithy_types::Document::Number(aws_smithy_types::Number::Float(f))
            } else {
                aws_smithy_types::Document::Null
            }
        }
        serde_json::Value::String(s) => aws_smithy_types::Document::String(s.clone()),
        serde_json::Value::Array(arr) => {
            aws_smithy_types::Document::Array(arr.iter().map(json_to_document).collect())
        }
        serde_json::Value::Object(obj) => aws_smithy_types::Document::Object(
            obj.iter()
                .map(|(k, v)| (k.clone(), json_to_document(v)))
                .collect(),
        ),
    }
}
