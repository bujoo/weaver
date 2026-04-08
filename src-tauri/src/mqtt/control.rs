use crate::mqtt::types::{ControlMessage, MqttIncoming};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

/// Tracks pause state and abort signals per mission/phase.
pub struct ControlHandler {
    /// mission_id -> paused
    paused: Arc<Mutex<HashMap<String, bool>>>,
    /// mission_id:phase_id -> abort signal
    abort_signals: Arc<Mutex<HashMap<String, tokio::sync::watch::Sender<bool>>>>,
}

impl ControlHandler {
    pub fn new() -> Self {
        Self {
            paused: Arc::new(Mutex::new(HashMap::new())),
            abort_signals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start(
        &self,
        mut rx: broadcast::Receiver<MqttIncoming>,
        app: tauri::AppHandle,
    ) {
        let paused = self.paused.clone();
        let abort_signals = self.abort_signals.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(MqttIncoming::Control(ctrl)) => {
                        handle_control(&paused, &abort_signals, &app, ctrl).await;
                    }
                    Ok(_) => {}
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
    }

    /// Register an abort signal for a phase execution. Returns a watch receiver
    /// that the executor can poll to check if it should abort.
    pub async fn register_abort(
        &self,
        mission_id: &str,
        phase_id: &str,
    ) -> tokio::sync::watch::Receiver<bool> {
        let key = format!("{}:{}", mission_id, phase_id);
        let (tx, rx) = tokio::sync::watch::channel(false);
        self.abort_signals.lock().await.insert(key, tx);
        rx
    }

    pub async fn is_paused(&self, mission_id: &str) -> bool {
        self.paused
            .lock()
            .await
            .get(mission_id)
            .copied()
            .unwrap_or(false)
    }
}

async fn handle_control(
    paused: &Arc<Mutex<HashMap<String, bool>>>,
    abort_signals: &Arc<Mutex<HashMap<String, tokio::sync::watch::Sender<bool>>>>,
    app: &tauri::AppHandle,
    ctrl: ControlMessage,
) {
    crate::debug_log::log_info(&format!(
        "[Control] action={} mission={:?} phase={:?} todo={:?}",
        ctrl.action, ctrl.mission_id, ctrl.phase_id, ctrl.todo_id
    ));

    match ctrl.action.as_str() {
        "kill" | "skip" => {
            if let Some(mid) = &ctrl.mission_id {
                let signals = abort_signals.lock().await;
                if let Some(pid) = &ctrl.phase_id {
                    // Phase-specific abort
                    let key = format!("{}:{}", mid, pid);
                    if let Some(tx) = signals.get(&key) {
                        let _ = tx.send(true);
                        crate::debug_log::log_info(&format!(
                            "[Control] Abort signal sent for {}",
                            key
                        ));
                    }
                } else {
                    // Mission-level kill: abort ALL phases for this mission
                    let prefix = format!("{}:", mid);
                    let mut count = 0;
                    for (key, tx) in signals.iter() {
                        if key.starts_with(&prefix) {
                            let _ = tx.send(true);
                            count += 1;
                        }
                    }
                    crate::debug_log::log_info(&format!(
                        "[Control] Mission-level {} for {}: aborted {} phases",
                        ctrl.action, mid, count
                    ));
                }
            }
        }
        "pause" => {
            if let Some(mid) = &ctrl.mission_id {
                paused.lock().await.insert(mid.clone(), true);
                crate::debug_log::log_info(&format!("[Control] Mission {} paused", mid));
            }
        }
        "resume" => {
            if let Some(mid) = &ctrl.mission_id {
                paused.lock().await.insert(mid.clone(), false);
                crate::debug_log::log_info(&format!("[Control] Mission {} resumed", mid));
            }
        }
        other => {
            crate::debug_log::log_warn(&format!("[Control] Unknown action: {}", other));
        }
    }

    // Emit to frontend
    use tauri::Emitter;
    let _ = app.emit("control-received", &serde_json::to_value(&ctrl).unwrap_or_default());
}
