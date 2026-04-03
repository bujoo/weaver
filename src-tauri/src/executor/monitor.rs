use crate::executor::spawner::TmuxSession;
use crate::mqtt::client::MqttClient;
use crate::mqtt::types::TodoStatusEvent;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

/// Monitor a tmux session running Claude Code.
/// Streams output to frontend, detects completion, publishes status to Brain.
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
    // Stream log output to frontend in background
    let log_path = session.log_path.clone();
    let app_stream = app.clone();
    let tid_stream = todo_id.clone();
    let stream_handle = tokio::spawn(async move {
        // Wait for log file to appear
        for _ in 0..60 {
            if log_path.exists() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        if let Ok(file) = tokio::fs::File::open(&log_path).await {
            let mut reader = BufReader::new(file).lines();
            loop {
                match reader.next_line().await {
                    Ok(Some(line)) => {
                        use tauri::Emitter;
                        let _ = app_stream.emit(
                            "executor-output",
                            serde_json::json!({
                                "todoId": &tid_stream,
                                "line": &line,
                            }),
                        );
                    }
                    Ok(None) => {
                        // No more lines yet -- wait and retry
                        tokio::time::sleep(Duration::from_millis(300)).await;
                    }
                    Err(_) => break,
                }
            }
        }
    });

    // Poll tmux session existence until it exits
    loop {
        let has_session = Command::new("tmux")
            .args(["has-session", "-t", &session.session_name])
            .output()
            .await;

        match has_session {
            Ok(output) if !output.status.success() => break, // Session exited
            Err(_) => break,                                  // tmux not available
            _ => {}
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    // Give log streaming a moment to catch up
    tokio::time::sleep(Duration::from_secs(1)).await;
    stream_handle.abort();

    // Determine success/failure from log content
    let todo_status = determine_exit_status(&session.log_path).await;
    let todo_error = if todo_status == "failed" {
        Some("Claude Code session exited with errors".to_string())
    } else {
        None
    };

    crate::debug_log::log_info(&format!(
        "[Executor] tmux session '{}' ended: {}",
        session.session_name, todo_status
    ));

    // Publish status to Brain via MQTT
    let event = TodoStatusEvent {
        mission_id: mission_id.clone(),
        todo_id: todo_id.clone(),
        phase_id: phase_id.clone(),
        status: todo_status.clone(),
        instance_id,
        files: vec![],
        cost: 0.0,
        tokens_used: 0,
        error: todo_error,
        published_at: Utc::now().to_rfc3339(),
    };

    let topic = format!("brain/{}/status/{}/{}", workspace, mission_id, todo_id);

    let guard = mqtt.lock().await;
    if let Some(client) = guard.as_ref() {
        if let Err(e) = client.publish_json(&topic, &event).await {
            crate::debug_log::log_error(&format!(
                "[Executor] Failed to publish status for {}: {}",
                todo_id, e
            ));
        }
    }

    // Emit completion to frontend
    use tauri::Emitter;
    let _ = app.emit(
        "todo-completed",
        serde_json::json!({
            "todoId": &todo_id,
            "status": &todo_status,
            "sessionName": &session.session_name,
        }),
    );

    Ok(())
}

/// Check log output for error indicators to determine status.
async fn determine_exit_status(log_path: &std::path::Path) -> String {
    if let Ok(content) = tokio::fs::read_to_string(log_path).await {
        // Check for common error patterns in Claude Code output
        let lower = content.to_lowercase();
        if lower.contains("error:") || lower.contains("fatal:") || lower.contains("panic") {
            return "failed".to_string();
        }
    }
    "completed".to_string()
}
