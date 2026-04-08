use crate::conductor::types::ConductorDecision;
use crate::executor::spawner::ClaudeCodeSpawner;
use crate::mqtt::client::MqttClient;
use crate::mqtt::state_cache::MissionStateCache;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Executes conductor decisions against the real system.
pub struct ActionExecutor {
    mqtt: Arc<Mutex<Option<MqttClient>>>,
    spawner: Arc<ClaudeCodeSpawner>,
    state_cache: Arc<Mutex<MissionStateCache>>,
    app: tauri::AppHandle,
}

impl ActionExecutor {
    pub fn new(
        mqtt: Arc<Mutex<Option<MqttClient>>>,
        spawner: Arc<ClaudeCodeSpawner>,
        state_cache: Arc<Mutex<MissionStateCache>>,
        app: tauri::AppHandle,
    ) -> Self {
        Self {
            mqtt,
            spawner,
            state_cache,
            app,
        }
    }

    pub async fn execute(&self, decision: &ConductorDecision) -> Result<(), String> {
        match decision {
            ConductorDecision::PushNextPhase {
                mission_id,
                phase_id,
                reason,
            } => {
                self.push_phase(mission_id, phase_id, reason).await
            }
            ConductorDecision::InjectContext {
                mission_id,
                message,
                ..
            } => self.inject_context(mission_id, message).await,
            ConductorDecision::ReportStatus {
                mission_id,
                phase_id,
                status,
                summary,
            } => {
                self.report_mqtt(mission_id, phase_id, status, summary)
                    .await
            }
            ConductorDecision::Escalate {
                mission_id,
                reason,
            } => self.escalate(mission_id, reason).await,
            ConductorDecision::NoAction { .. } => Ok(()),
            _ => {
                crate::debug_log::log_info(&format!(
                    "[Weavy] Unhandled decision: {:?}",
                    decision
                ));
                Ok(())
            }
        }
    }

    async fn push_phase(
        &self,
        mission_id: &str,
        phase_id: &str,
        reason: &str,
    ) -> Result<(), String> {
        let mid_short = if mission_id.len() > 8 {
            &mission_id[..8]
        } else {
            mission_id
        };

        // Find the channel port
        let workspace_mount = crate::settings::load_settings().workspace_mount;
        let port_file = std::path::PathBuf::from(&workspace_mount)
            .join(".worktrees")
            .join(mid_short)
            .join("weaver")
            .join(".weaver")
            .join("channel-port");

        let port: u16 = std::fs::read_to_string(&port_file)
            .map_err(|e| format!("No channel port: {}", e))?
            .trim()
            .parse()
            .map_err(|e| format!("Invalid port: {}", e))?;

        // Build assignment from state cache
        let cache = self.state_cache.lock().await;
        let phase = cache
            .get_phase(mission_id, phase_id)
            .ok_or(format!("Phase {} not found in cache", phase_id))?;
        let todos = cache.get_todos_for_phase(mission_id, phase_id);
        let todo_ids: Vec<String> = todos.iter().map(|t| t.todo_id.clone()).collect();

        let payload = serde_json::json!({
            "type": "assignment",
            "mission_id": mission_id,
            "phase_id": phase_id,
            "phase_name": phase.name,
            "content": format!(
                "Execute Phase '{}' ({} todos). Read .weaver/specs/ for each todo spec. Reason: {}",
                phase.name, todo_ids.len(), reason
            ),
            "todos": todo_ids,
        });
        drop(cache);

        let client = reqwest::Client::new();
        client
            .post(format!("http://127.0.0.1:{}", port))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Channel POST failed: {}", e))?;

        crate::debug_log::log_info(&format!(
            "[Weavy] Pushed phase {} to channel: {}",
            phase_id, reason
        ));

        // Emit to frontend
        use tauri::Emitter;
        let _ = self.app.emit(
            "conductor-action",
            serde_json::json!({
                "action": "push_next_phase",
                "mission_id": mission_id,
                "phase_id": phase_id,
                "reason": reason,
            }),
        );

        Ok(())
    }

    async fn inject_context(
        &self,
        mission_id: &str,
        message: &str,
    ) -> Result<(), String> {
        let mid_short = if mission_id.len() > 8 {
            &mission_id[..8]
        } else {
            mission_id
        };

        let workspace_mount = crate::settings::load_settings().workspace_mount;
        let port_file = std::path::PathBuf::from(&workspace_mount)
            .join(".worktrees")
            .join(mid_short)
            .join("weaver")
            .join(".weaver")
            .join("channel-port");

        let port: u16 = std::fs::read_to_string(&port_file)
            .map_err(|e| format!("No channel port: {}", e))?
            .trim()
            .parse()
            .map_err(|e| format!("Invalid port: {}", e))?;

        let payload = serde_json::json!({
            "type": "context",
            "mission_id": mission_id,
            "content": message,
        });

        let client = reqwest::Client::new();
        client
            .post(format!("http://127.0.0.1:{}", port))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Channel POST failed: {}", e))?;

        crate::debug_log::log_info(&format!(
            "[Weavy] Injected context for {}: {}",
            mission_id,
            &message[..80.min(message.len())]
        ));

        Ok(())
    }

    async fn report_mqtt(
        &self,
        mission_id: &str,
        phase_id: &str,
        status: &str,
        summary: &str,
    ) -> Result<(), String> {
        let guard = self.mqtt.lock().await;
        if let Some(client) = guard.as_ref() {
            let config = client.config();
            let topic = format!(
                "brain/{}/status/{}/{}",
                config.workspace, mission_id, phase_id
            );
            let payload = serde_json::json!({
                "mission_id": mission_id,
                "phase_id": phase_id,
                "status": status,
                "summary": summary,
                "source": "weavy",
                "published_at": chrono::Utc::now().to_rfc3339(),
            });
            client.publish_json(&topic, &payload).await?;
        }
        Ok(())
    }

    async fn escalate(&self, mission_id: &str, reason: &str) -> Result<(), String> {
        crate::debug_log::log_info(&format!(
            "[Weavy] ESCALATION for {}: {}",
            mission_id, reason
        ));

        // Desktop notification
        use tauri::Emitter;
        let _ = self.app.emit(
            "conductor-escalation",
            serde_json::json!({
                "mission_id": mission_id,
                "reason": reason,
            }),
        );

        // macOS notification
        #[cfg(target_os = "macos")]
        {
            use tauri_plugin_notification::NotificationExt;
            let _ = self
                .app
                .notification()
                .builder()
                .title("Weavy needs your help")
                .body(reason)
                .show();
        }

        Ok(())
    }
}
