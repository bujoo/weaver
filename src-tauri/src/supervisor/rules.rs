use serde::{Deserialize, Serialize};

/// Configurable detection rules for the Supervisor agent.
/// These thresholds control when the supervisor flags issues and whether
/// it takes autonomous action or merely observes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupervisorRules {
    /// Same error repeated more than N times triggers a retry-loop flag.
    pub retry_threshold: u32,
    /// Window (in seconds) within which retries are counted.
    pub retry_window_secs: u64,
    /// No activity for longer than this triggers an idle flag.
    pub idle_timeout_secs: u64,
    /// Automatically restart crashed Claude Code sessions.
    pub crash_auto_restart: bool,
    /// Maximum number of auto-restarts before escalating to a human.
    pub max_auto_restarts: u32,
    /// Unanswered permission request timeout (seconds).
    pub permission_timeout_secs: u64,
    /// Autonomous mode: take action on observations. When false, only observe and suggest.
    pub autopilot: bool,
}

impl Default for SupervisorRules {
    fn default() -> Self {
        Self {
            retry_threshold: 3,
            retry_window_secs: 300,
            idle_timeout_secs: 300,
            crash_auto_restart: true,
            max_auto_restarts: 2,
            permission_timeout_secs: 120,
            autopilot: false,
        }
    }
}
