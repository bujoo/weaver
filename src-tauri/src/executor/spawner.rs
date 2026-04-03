use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnedProcess {
    pub todo_id: String,
    pub session_name: String,
    pub status: String, // running | completed | failed | killed
}

pub struct TmuxSession {
    pub session_name: String,
    pub mission_id: String,
}

pub struct ClaudeCodeSpawner {
    processes: Arc<Mutex<HashMap<String, SpawnedProcess>>>,
}

impl ClaudeCodeSpawner {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawn Claude Code in a tmux session with the Weaver channel plugin.
    /// The channel handles all communication -- no prompts, no monitoring.
    pub async fn spawn_session(
        &self,
        mission_id: &str,
        cwd: &Path,
        plugin_dir: &Path,
    ) -> Result<TmuxSession, String> {
        let short_mid = if mission_id.len() > 8 {
            &mission_id[..8]
        } else {
            mission_id
        };
        let session_name = format!("weaver-{}", short_mid);

        // Clean up stale channel-port file
        let port_file = cwd.join(".weaver").join("channel-port");
        let _ = std::fs::remove_file(&port_file);

        // Build the claude command with channel plugin
        let shell_cmd = format!(
            "export CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1; \
             claude --dangerously-load-development-channels server:weaver \
             --dangerously-skip-permissions \
             --plugin-dir {}",
            plugin_dir.display()
        );

        // Kill existing session with same name
        let _ = Command::new("tmux")
            .args(["kill-session", "-t", &session_name])
            .output()
            .await;

        // Create tmux session
        let output = Command::new("tmux")
            .args(["new-session", "-d", "-s", &session_name, "-c"])
            .arg(cwd)
            .arg(&shell_cmd)
            .output()
            .await
            .map_err(|e| format!("Failed to create tmux session: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("tmux new-session failed: {}", stderr));
        }

        crate::debug_log::log_info(&format!(
            "[Executor] Spawned Claude Code session '{}' in {}",
            session_name,
            cwd.display()
        ));

        self.processes.lock().await.insert(
            mission_id.to_string(),
            SpawnedProcess {
                todo_id: mission_id.to_string(),
                session_name: session_name.clone(),
                status: "running".into(),
            },
        );

        Ok(TmuxSession {
            session_name,
            mission_id: mission_id.to_string(),
        })
    }

    /// Read the channel port from .weaver/channel-port (written by the channel server).
    /// Polls until the file appears or timeout.
    pub async fn wait_for_channel_port(cwd: &Path) -> Result<u16, String> {
        let port_file = cwd.join(".weaver").join("channel-port");
        for _ in 0..30 {
            // 30 * 1s = 30s timeout
            if port_file.exists() {
                let content = std::fs::read_to_string(&port_file)
                    .map_err(|e| format!("Failed to read channel-port: {}", e))?;
                let port = content
                    .trim()
                    .parse::<u16>()
                    .map_err(|e| format!("Invalid port in channel-port: {}", e))?;
                crate::debug_log::log_info(&format!(
                    "[Executor] Channel server ready on port {}",
                    port
                ));
                return Ok(port);
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        Err("Channel server did not start within 30s".to_string())
    }

    pub async fn mark_completed(&self, id: &str) {
        if let Some(proc) = self.processes.lock().await.get_mut(id) {
            proc.status = "completed".into();
        }
    }

    pub async fn mark_failed(&self, id: &str) {
        if let Some(proc) = self.processes.lock().await.get_mut(id) {
            proc.status = "failed".into();
        }
    }

    pub async fn get_processes(&self) -> Vec<SpawnedProcess> {
        self.processes.lock().await.values().cloned().collect()
    }

    pub async fn active_count(&self) -> usize {
        self.processes
            .lock()
            .await
            .values()
            .filter(|p| p.status == "running")
            .count()
    }
}
