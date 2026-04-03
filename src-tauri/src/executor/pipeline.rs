use crate::executor::spawner::ClaudeCodeSpawner;
use crate::mqtt::client::MqttClient;
use crate::mqtt::state_cache::MissionStateCache;
use crate::mqtt::types::PhaseAssignment;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

/// Execute a phase assignment: spawn Claude Code with channel plugin, POST assignment.
/// The channel + hooks handle all communication and monitoring.
pub async fn execute_assignment(
    assignment: PhaseAssignment,
    workspace_mount: &Path,
    spawner: Arc<ClaudeCodeSpawner>,
    _mqtt: Arc<Mutex<Option<MqttClient>>>,
    _state_cache: Arc<Mutex<MissionStateCache>>,
    _instance_id: String,
    _abort_rx: watch::Receiver<bool>,
    _app: tauri::AppHandle,
) -> Result<(), String> {
    let mission_short = if assignment.mission_id.len() > 8 {
        &assignment.mission_id[..8]
    } else {
        &assignment.mission_id
    };

    // Check weaver/ exists
    let weaver_cwd = workspace_mount
        .join(".worktrees")
        .join(mission_short)
        .join("weaver");

    if !weaver_cwd.exists() {
        return Err(format!(
            "No weaver/ workspace at {}. Run autopilot first.",
            weaver_cwd.display()
        ));
    }

    // Plugin directory (relative to the contexthub-weaver project)
    let plugin_dir = workspace_mount
        .parent()
        .unwrap_or(workspace_mount)
        .join("contexthub-weaver")
        .join("weaver-plugin");

    // Fallback: check common locations
    let plugin_dir = if plugin_dir.exists() {
        plugin_dir
    } else {
        // Try relative to the weaver source
        let home = dirs::home_dir().unwrap_or_default();
        let alt = home
            .join("Sonic-Web-Dev")
            .join("contexthub")
            .join("contexthub-weaver")
            .join("weaver-plugin");
        if alt.exists() {
            alt
        } else {
            return Err("Weaver plugin directory not found".to_string());
        }
    };

    crate::debug_log::log_info(&format!(
        "[Pipeline] Starting Claude Code for phase {} in {}",
        assignment.phase_name,
        weaver_cwd.display()
    ));

    // Spawn Claude Code with channel plugin in tmux
    let _session = spawner
        .spawn_session(&assignment.mission_id, &weaver_cwd, &plugin_dir)
        .await?;

    // Wait for the channel server to start and write its port
    let port = ClaudeCodeSpawner::wait_for_channel_port(&weaver_cwd).await?;

    // POST the phase assignment to the channel
    let payload = serde_json::json!({
        "type": "assignment",
        "mission_id": assignment.mission_id,
        "phase_id": assignment.phase_id,
        "phase_name": assignment.phase_name,
        "todos": assignment.todos,
        "content": format!(
            "Execute Phase '{}' ({} todos). Read .weaver/specs/ for each todo spec. Follow the phase-workflow skill.",
            assignment.phase_name,
            assignment.todos.len()
        ),
    });

    let url = format!("http://127.0.0.1:{}", port);
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to POST assignment to channel: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Channel rejected assignment: {}", resp.status()));
    }

    crate::debug_log::log_info(&format!(
        "[Pipeline] Assignment posted to channel port {} for phase {}",
        port, assignment.phase_name
    ));

    // Done -- hooks and channel handle the rest
    // Phase completion detected via /channel/phase-complete hook endpoint
    Ok(())
}
