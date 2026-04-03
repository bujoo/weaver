use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Tracks phase execution progress via HTTP hook callbacks.
/// No polling, no log tailing -- hooks push state updates.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseMonitor {
    pub mission_id: String,
    pub phase_id: String,
    pub todos: Vec<String>,
    pub completed: HashSet<String>,
    pub failed: HashSet<String>,
    pub tool_use_count: u32,
    pub session_active: bool,
}

impl PhaseMonitor {
    pub fn new(mission_id: String, phase_id: String, todos: Vec<String>) -> Self {
        Self {
            mission_id,
            phase_id,
            todos,
            completed: HashSet::new(),
            failed: HashSet::new(),
            tool_use_count: 0,
            session_active: false,
        }
    }

    pub fn session_started(&mut self) {
        self.session_active = true;
    }

    pub fn session_ended(&mut self) {
        self.session_active = false;
    }

    pub fn tool_used(&mut self) {
        self.tool_use_count += 1;
    }

    pub fn todo_completed(&mut self, todo_id: &str) -> bool {
        self.completed.insert(todo_id.to_string());
        self.is_phase_complete()
    }

    pub fn todo_failed(&mut self, todo_id: &str) {
        self.failed.insert(todo_id.to_string());
    }

    pub fn is_phase_complete(&self) -> bool {
        self.completed.len() == self.todos.len()
    }

    pub fn progress(&self) -> f32 {
        if self.todos.is_empty() {
            return 1.0;
        }
        self.completed.len() as f32 / self.todos.len() as f32
    }
}

/// Manages monitors for all active phases across missions.
pub struct MonitorRegistry {
    monitors: HashMap<String, PhaseMonitor>, // key: "{mission_id}:{phase_id}"
}

impl MonitorRegistry {
    pub fn new() -> Self {
        Self {
            monitors: HashMap::new(),
        }
    }

    pub fn register(&mut self, monitor: PhaseMonitor) {
        let key = format!("{}:{}", monitor.mission_id, monitor.phase_id);
        self.monitors.insert(key, monitor);
    }

    pub fn get(&self, mission_id: &str, phase_id: &str) -> Option<&PhaseMonitor> {
        let key = format!("{}:{}", mission_id, phase_id);
        self.monitors.get(&key)
    }

    pub fn get_mut(&mut self, mission_id: &str, phase_id: &str) -> Option<&mut PhaseMonitor> {
        let key = format!("{}:{}", mission_id, phase_id);
        self.monitors.get_mut(&key)
    }

    pub fn all(&self) -> Vec<&PhaseMonitor> {
        self.monitors.values().collect()
    }
}

/// Hook payload from Claude Code HTTP hooks.
#[derive(Debug, Deserialize)]
pub struct HookPayload {
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    pub hook_event_name: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
}

/// Channel reply from Claude Code via the weaver channel.
#[derive(Debug, Deserialize)]
pub struct ChannelReply {
    pub mission_id: Option<String>,
    pub todo_id: Option<String>,
    pub phase_id: Option<String>,
    pub r#type: Option<String>,
    pub message: Option<String>,
    pub summary: Option<String>,
    pub files_modified: Option<Vec<String>>,
}
