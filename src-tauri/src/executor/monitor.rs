use crate::mqtt::client::MqttClient;
use crate::mqtt::types::TodoStatusEvent;
use chrono::Utc;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::Mutex;

/// Monitor a spawned Claude Code process, capture output, detect completion.
pub async fn monitor_process(
    mut child: Child,
    todo_id: String,
    mission_id: String,
    phase_id: String,
    instance_id: String,
    mqtt: Arc<Mutex<Option<MqttClient>>>,
    workspace: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // Read stdout
    let stdout = child
        .stdout
        .take()
        .ok_or("Failed to capture stdout")?;
    let mut reader = BufReader::new(stdout).lines();

    let mut output_lines = Vec::new();

    // Stream output
    while let Ok(Some(line)) = reader.next_line().await {
        output_lines.push(line.clone());

        // Emit to frontend for live monitoring
        use tauri::Emitter;
        let _ = app.emit(
            "executor-output",
            serde_json::json!({
                "todoId": &todo_id,
                "line": &line,
            }),
        );
    }

    // Wait for process to complete
    let exit_status = child
        .wait()
        .await
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    let (todo_status, todo_error): (String, Option<String>) = if exit_status.success() {
        ("completed".to_string(), None)
    } else {
        let code = exit_status.code().unwrap_or(-1);
        ("failed".to_string(), Some(format!("Exit code: {}", code)))
    };

    crate::debug_log::log_info(&format!(
        "[Executor] Todo {} finished with status: {}",
        todo_id, todo_status
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

    let topic = format!(
        "brain/{}/status/{}/{}",
        workspace, mission_id, todo_id
    );

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
        }),
    );

    Ok(())
}
