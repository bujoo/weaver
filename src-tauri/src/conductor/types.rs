use serde::{Deserialize, Serialize};

/// Events the conductor receives from hooks and channel replies.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConductorEvent {
    TodoCompleted {
        mission_id: String,
        phase_id: String,
        todo_id: String,
        summary: String,
        files_modified: Vec<String>,
    },
    PhaseCompleted {
        mission_id: String,
        phase_id: String,
        summary: String,
    },
    SessionStarted {
        mission_id: String,
        session_id: String,
    },
    SessionEnded {
        mission_id: String,
        session_id: String,
        reason: String,
    },
    ToolUsed {
        mission_id: String,
        tool_name: String,
    },
    ClaudeStopped {
        mission_id: String,
        session_id: String,
    },
    TeammateIdle {
        mission_id: String,
        agent_name: String,
    },
    ChannelReply {
        mission_id: String,
        reply_type: String,
        message: String,
    },
    AssignmentReceived {
        mission_id: String,
        phase_id: String,
        phase_name: String,
        todo_count: usize,
    },
    ErrorDetected {
        mission_id: String,
        phase_id: String,
        error: String,
    },
}

/// Decisions the conductor AI can make.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ConductorDecision {
    PushNextPhase {
        mission_id: String,
        phase_id: String,
        reason: String,
    },
    InjectContext {
        mission_id: String,
        message: String,
        reason: String,
    },
    Retry {
        mission_id: String,
        phase_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        todo_id: Option<String>,
        reason: String,
    },
    SkipPhase {
        mission_id: String,
        phase_id: String,
        reason: String,
    },
    Escalate {
        mission_id: String,
        reason: String,
    },
    ReportStatus {
        mission_id: String,
        phase_id: String,
        status: String,
        summary: String,
    },
    SpawnSession {
        mission_id: String,
        phase_id: String,
        reason: String,
    },
    KillSession {
        mission_id: String,
        session_name: String,
        reason: String,
    },
    NoAction {
        reason: String,
    },
}

/// Which model tier to use for a decision.
#[derive(Clone, Debug, PartialEq)]
pub enum ModelTier {
    Haiku,
    Sonnet,
    Opus,
}

impl ModelTier {
    pub fn model_id(&self) -> &'static str {
        match self {
            ModelTier::Haiku => "us.anthropic.claude-haiku-4-5-20251001",
            ModelTier::Sonnet => "us.anthropic.claude-sonnet-4-6-20250514",
            ModelTier::Opus => "us.anthropic.claude-opus-4-6-v1",
        }
    }
}

/// Configuration for the conductor.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConductorConfig {
    pub enabled: bool,
    pub aws_profile: String,
    pub aws_region: String,
    pub min_decision_interval_secs: u64,
    pub max_event_buffer: usize,
}

impl Default for ConductorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            aws_profile: "wds_dev".into(),
            aws_region: "us-west-2".into(),
            min_decision_interval_secs: 30,
            max_event_buffer: 20,
        }
    }
}

/// A recorded decision for logging and dashboard display.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionRecord {
    pub timestamp: String,
    pub model_used: String,
    pub decision: ConductorDecision,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub event_count: usize,
}
