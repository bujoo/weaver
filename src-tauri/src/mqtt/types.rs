use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Assignment (Brain -> Weaver) ────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseAssignment {
    pub mission_id: String,
    pub workspace: String,
    pub phase_id: String,
    pub phase_name: String,
    pub todos: Vec<String>,
    pub context_topics: Vec<String>,
    pub config: serde_json::Value,
    pub strategy: String,
    pub published_at: String,
}

// ── Control (Brain -> Weaver) ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMessage {
    pub action: String,
    #[serde(default)]
    pub mission_id: Option<String>,
    #[serde(default)]
    pub phase_id: Option<String>,
    #[serde(default)]
    pub todo_id: Option<String>,
    #[serde(default)]
    pub reason: String,
    pub published_at: String,
}

// ── Status (Weaver -> Brain) ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoStatusEvent {
    pub mission_id: String,
    pub todo_id: String,
    pub phase_id: String,
    pub status: String,
    #[serde(default)]
    pub instance_id: String,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub cost: f64,
    #[serde(default)]
    pub tokens_used: u64,
    #[serde(default)]
    pub error: Option<String>,
    pub published_at: String,
}

// ── Heartbeat (Weaver -> Brain) ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatEvent {
    pub instance_id: String,
    pub workspace: String,
    pub capacity: u32,
    pub active_agents: u32,
    pub missions: Vec<String>,
    pub instance_type: String,
    pub tags: HashMap<String, String>,
    pub uptime_seconds: u64,
    pub published_at: String,
}

// ── State messages (Brain -> Weaver, retained) ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStateMessage {
    pub mission_id: String,
    pub workspace: String,
    pub version: u32,
    pub title: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub phases: Vec<serde_json::Value>,
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseStateMessage {
    pub mission_id: String,
    pub phase_id: String,
    pub name: String,
    pub order: u32,
    pub status: String,
    #[serde(default)]
    pub blocked_by: Vec<String>,
    #[serde(default)]
    pub config: serde_json::Value,
    #[serde(default)]
    pub todo_count: u32,
    #[serde(default)]
    pub completed_count: u32,
    pub published_at: String,
    #[serde(default)]
    pub execution_target: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoStateMessage {
    pub mission_id: String,
    pub phase_id: String,
    pub todo_id: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub description: String,
    pub status: String,
    #[serde(default)]
    pub file_paths: Vec<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
    #[serde(default)]
    pub spec: Option<serde_json::Value>,
    pub published_at: String,
}

// ── Workspace registry (Brain -> Weaver, retained) ─────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRegistryMessage {
    pub workspace: String,
    pub missions: Vec<MissionSummary>,
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionSummary {
    pub mission_id: String,
    pub title: String,
    pub status: String,
    #[serde(default)]
    pub repo_url: Option<String>,
    #[serde(default)]
    pub repos: Vec<RepoInfoMqtt>,
    #[serde(default)]
    pub phase_count: u32,
    #[serde(default)]
    pub todo_count: u32,
    #[serde(default)]
    pub available_phases: Vec<AvailablePhaseInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailablePhaseInfo {
    pub phase_id: String,
    pub name: String,
    pub order: u32,
    pub status: String,
    #[serde(default)]
    pub todo_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfoMqtt {
    pub repo_id: String,
    #[serde(default)]
    pub repo_url: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
}

// ── MQTT connection config ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub instance_id: String,
    pub workspace: String,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            host: "localhost".into(),
            port: 1883,
            username: "weaver-dev".into(),
            password: "weaver-dev-secret".into(),
            instance_id: format!("weaver-{}", &uuid::Uuid::new_v4().to_string()[..8]),
            workspace: "dev".into(),
        }
    }
}

// ── Incoming MQTT message wrapper ───────────────────────────────────

#[derive(Debug, Clone)]
pub enum MqttIncoming {
    Assignment(PhaseAssignment),
    Control(ControlMessage),
    Registry(WorkspaceRegistryMessage),
    PlanState(PlanStateMessage),
    PhaseState(PhaseStateMessage),
    TodoState(TodoStateMessage),
    ContextBundle(String, serde_json::Value), // (todo_id, bundle)
    Unknown(String, Vec<u8>),                 // (topic, payload)
}
