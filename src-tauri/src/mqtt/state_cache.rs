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
}
