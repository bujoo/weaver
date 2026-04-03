use crate::mqtt::types::{PhaseStateMessage, PlanStateMessage, TodoStateMessage};
use std::collections::HashMap;

/// Cache for retained MQTT state messages published by Brain.
/// Stores plans, phases, and todos so the executor can look up
/// full specs when building Claude Code prompts.
pub struct MissionStateCache {
    plans: HashMap<String, PlanStateMessage>,
    phases: HashMap<String, PhaseStateMessage>,   // key: "{mission_id}:{phase_id}"
    todos: HashMap<String, TodoStateMessage>,      // key: "{todo_id}"
}

impl MissionStateCache {
    pub fn new() -> Self {
        Self {
            plans: HashMap::new(),
            phases: HashMap::new(),
            todos: HashMap::new(),
        }
    }

    pub fn store_plan(&mut self, msg: PlanStateMessage) {
        crate::debug_log::log_info(&format!(
            "[StateCache] Stored plan: {} ({})",
            msg.title, msg.mission_id
        ));
        self.plans.insert(msg.mission_id.clone(), msg);
    }

    pub fn store_phase(&mut self, msg: PhaseStateMessage) {
        let key = format!("{}:{}", msg.mission_id, msg.phase_id);
        crate::debug_log::log_info(&format!(
            "[StateCache] Stored phase: {} ({}) status={}",
            msg.name, key, msg.status
        ));
        self.phases.insert(key, msg);
    }

    pub fn store_todo(&mut self, msg: TodoStateMessage) {
        crate::debug_log::log_info(&format!(
            "[StateCache] Stored todo: {} role={} has_spec={}",
            msg.todo_id, msg.role, msg.spec.is_some()
        ));
        self.todos.insert(msg.todo_id.clone(), msg);
    }

    pub fn get_plan(&self, mission_id: &str) -> Option<&PlanStateMessage> {
        self.plans.get(mission_id)
    }

    pub fn get_phase(&self, mission_id: &str, phase_id: &str) -> Option<&PhaseStateMessage> {
        let key = format!("{}:{}", mission_id, phase_id);
        self.phases.get(&key)
    }

    pub fn get_todo(&self, todo_id: &str) -> Option<&TodoStateMessage> {
        self.todos.get(todo_id)
    }

    /// Get all todos for a given phase.
    pub fn get_todos_for_phase(&self, mission_id: &str, phase_id: &str) -> Vec<&TodoStateMessage> {
        self.todos
            .values()
            .filter(|t| t.mission_id == mission_id && t.phase_id == phase_id)
            .collect()
    }

    /// Snapshot of all cached state for frontend/debug queries.
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({
            "plans": self.plans.len(),
            "phases": self.phases.len(),
            "todos": self.todos.len(),
            "plan_ids": self.plans.keys().collect::<Vec<_>>(),
            "todo_ids": self.todos.keys().collect::<Vec<_>>(),
        })
    }

    /// Load a full WeaverPlan JSON (as Brain produces it) and decompose into
    /// PlanState, PhaseState, and TodoState entries.
    /// Works for both fixture files and full-plan retained MQTT messages.
    pub fn load_weaver_plan(&mut self, plan_json: &serde_json::Value) -> Result<LoadResult, String> {
        let mission_id = plan_json
            .get("mission_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing mission_id")?
            .to_string();
        let workspace = plan_json
            .get("workspace")
            .and_then(|v| v.as_str())
            .unwrap_or("dev")
            .to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // Store plan
        let plan_msg = PlanStateMessage {
            mission_id: mission_id.clone(),
            workspace: workspace.clone(),
            version: plan_json.get("version").and_then(|v| v.as_u64()).unwrap_or(1) as u32,
            title: plan_json
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            scope: plan_json
                .get("scope")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            status: plan_json
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("draft")
                .to_string(),
            phases: plan_json
                .get("phases")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default(),
            published_at: now.clone(),
        };
        self.store_plan(plan_msg);

        let mut phase_count = 0u32;
        let mut todo_count = 0u32;

        // Decompose phases and todos
        let phases = plan_json
            .get("phases")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        for phase in &phases {
            let phase_id = phase
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let todos_arr = phase
                .get("todos")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            let phase_msg = PhaseStateMessage {
                mission_id: mission_id.clone(),
                phase_id: phase_id.clone(),
                name: phase
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                order: phase.get("order").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                status: phase
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("pending")
                    .to_string(),
                blocked_by: phase
                    .get("blocked_by")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                config: phase.get("config").cloned().unwrap_or_default(),
                todo_count: todos_arr.len() as u32,
                completed_count: 0,
                published_at: now.clone(),
                execution_target: phase.get("execution_target").cloned(),
            };
            self.store_phase(phase_msg);
            phase_count += 1;

            for todo in &todos_arr {
                let todo_id = todo
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let todo_msg = TodoStateMessage {
                    mission_id: mission_id.clone(),
                    phase_id: phase_id.clone(),
                    todo_id: todo_id.clone(),
                    role: todo
                        .get("role")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: todo
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    status: todo
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("pending")
                        .to_string(),
                    file_paths: todo
                        .get("file_paths")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    blocked_by: todo
                        .get("blocked_by")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    spec: todo.get("spec").cloned(),
                    published_at: now.clone(),
                };
                self.store_todo(todo_msg);
                todo_count += 1;
            }
        }

        Ok(LoadResult {
            mission_id,
            title: plan_json
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            phases: phase_count,
            todos: todo_count,
        })
    }

    /// Get all cached phases (for iteration in autopilot).
    pub fn all_phases(&self) -> impl Iterator<Item = &PhaseStateMessage> {
        self.phases.values()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadResult {
    pub mission_id: String,
    pub title: String,
    pub phases: u32,
    pub todos: u32,
}
