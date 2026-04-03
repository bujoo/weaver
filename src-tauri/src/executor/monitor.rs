use crate::executor::spawner::TmuxSession;
use crate::mqtt::client::MqttClient;
use crate::mqtt::types::TodoStatusEvent;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

/// Monitor a tmux session running Claude Code.
/// Tails the log file for output streaming, watches for exit marker.
pub async fn monitor_tmux_session(
    session: TmuxSession,
    todo_id: String,
    mission_id: String,
    phase_id: String,
    instance_id: String,
    mqtt: Arc<Mutex<Option<MqttClient>>>,
    workspace: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let log_path = session.log_path.clone();

    // Wait for log file to appear (max 30s)
    let mut waited = 0;
    while !log_path.exists() && waited < 60 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        waited += 1;
    }

    if !log_path.exists() {
        crate::debug_log::log_error(&format!(
            "[Monitor] Log file never appeared for {}",
            session.session_name
        ));
        // Fall back to polling tmux session
        return monitor_by_session_poll(session, todo_id, mission_id, phase_id, instance_id, mqtt, workspace, app).await;
    }

    // Tail the log file, stream to frontend, watch for exit marker
    let exit_code = tail_log_file(&log_path, &todo_id, &app).await;

    let (todo_status, todo_error) = if exit_code == 0 {
        ("completed".to_string(), None)
    } else {
        (
            "failed".to_string(),
            Some(format!("Claude Code exited with code {}", exit_code)),
        )
    };

    crate::debug_log::log_info(&format!(
        "[Monitor] Session '{}' completed: {} (exit code {})",
        session.session_name, todo_status, exit_code
    ));

    // Publish status to Brain via MQTT
    publish_status(
        &todo_id, &mission_id, &phase_id, &instance_id,
        &todo_status, todo_error, &mqtt, &workspace, &app,
    ).await;

    Ok(())
}

/// Tail a log file, emit lines to frontend, return exit code when marker found.
async fn tail_log_file(
    log_path: &std::path::Path,
    todo_id: &str,
    app: &tauri::AppHandle,
) -> i32 {
    let file = match tokio::fs::File::open(log_path).await {
        Ok(f) => f,
        Err(e) => {
            crate::debug_log::log_error(&format!("[Monitor] Failed to open log: {}", e));
            return -1;
        }
    };

    let mut reader = BufReader::new(file);
    let mut line_buf = String::new();

    loop {
        line_buf.clear();
        match reader.read_line(&mut line_buf).await {
            Ok(0) => {
                // EOF -- no more data yet, wait and retry
                tokio::time::sleep(Duration::from_millis(300)).await;

                // Check if tmux session is still alive (safety net)
                let alive = tokio::process::Command::new("tmux")
                    .args(["has-session", "-t", &format!("weaver-{}", todo_id.replace('.', "-"))])
                    .output()
                    .await
                    .map(|o| o.status.success())
                    .unwrap_or(false);

                if !alive {
                    // Session ended without exit marker -- check for marker in file
                    if let Ok(content) = tokio::fs::read_to_string(log_path).await {
                        if let Some(code) = parse_exit_marker(&content) {
                            return code;
                        }
                    }
                    return -1; // Unknown exit
                }
            }
            Ok(_) => {
                let line = line_buf.trim_end().to_string();

                // Check for exit marker
                if let Some(code) = parse_exit_marker(&line) {
                    return code;
                }

                // Emit to frontend
                use tauri::Emitter;
                let _ = app.emit(
                    "executor-output",
                    serde_json::json!({
                        "todoId": todo_id,
                        "line": &line,
                    }),
                );
            }
            Err(e) => {
                crate::debug_log::log_error(&format!("[Monitor] Read error: {}", e));
                return -1;
            }
        }
    }
}

/// Parse __WEAVER_EXIT_{code}__ marker from a line.
fn parse_exit_marker(text: &str) -> Option<i32> {
    if let Some(start) = text.find("__WEAVER_EXIT_") {
        let after = &text[start + 14..]; // skip "__WEAVER_EXIT_"
        if let Some(end) = after.find("__") {
            return after[..end].parse::<i32>().ok();
        }
    }
    None
}

/// Fallback: poll tmux session existence.
async fn monitor_by_session_poll(
    session: TmuxSession,
    todo_id: String,
    mission_id: String,
    phase_id: String,
    instance_id: String,
    mqtt: Arc<Mutex<Option<MqttClient>>>,
    workspace: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    loop {
        let has_session = tokio::process::Command::new("tmux")
            .args(["has-session", "-t", &session.session_name])
            .output()
            .await;

        match has_session {
            Ok(o) if !o.status.success() => break,
            Err(_) => break,
            _ => {}
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    let todo_status = "completed".to_string();
    crate::debug_log::log_info(&format!(
        "[Monitor] Session '{}' ended (poll fallback)",
        session.session_name
    ));

    publish_status(
        &todo_id, &mission_id, &phase_id, &instance_id,
        &todo_status, None, &mqtt, &workspace, &app,
    ).await;

    Ok(())
}

/// Publish todo status to Brain via MQTT + emit to frontend.
async fn publish_status(
    todo_id: &str,
    mission_id: &str,
    phase_id: &str,
    instance_id: &str,
    status: &str,
    error: Option<String>,
    mqtt: &Arc<Mutex<Option<MqttClient>>>,
    workspace: &str,
    app: &tauri::AppHandle,
) {
    let event = TodoStatusEvent {
        mission_id: mission_id.to_string(),
        todo_id: todo_id.to_string(),
        phase_id: phase_id.to_string(),
        status: status.to_string(),
        instance_id: instance_id.to_string(),
        files: vec![],
        cost: 0.0,
        tokens_used: 0,
        error,
        published_at: Utc::now().to_rfc3339(),
    };

    let topic = format!("brain/{}/status/{}/{}", workspace, mission_id, todo_id);

    let guard = mqtt.lock().await;
    if let Some(client) = guard.as_ref() {
        if let Err(e) = client.publish_json(&topic, &event).await {
            crate::debug_log::log_error(&format!(
                "[Monitor] Failed to publish status for {}: {}",
                todo_id, e
            ));
        }
    }

    use tauri::Emitter;
    let _ = app.emit(
        "todo-completed",
        serde_json::json!({
            "todoId": todo_id,
            "status": status,
        }),
    );
}
