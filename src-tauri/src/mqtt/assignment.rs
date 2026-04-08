use crate::executor::pipeline::execute_assignment;
use crate::executor::spawner::ClaudeCodeSpawner;
use crate::mqtt::client::MqttClient;
use crate::mqtt::control::ControlHandler;
use crate::mqtt::state_cache::MissionStateCache;
use crate::mqtt::types::{MqttIncoming, PhaseAssignment};
use crate::workspace::autopilot;
use chrono::Utc;
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskQueueEntry {
    pub mission_id: String,
    pub phase_id: String,
    pub phase_name: String,
    pub todos: Vec<String>,
    pub status: String, // queued | preparing | executing | completed | failed
    pub received_at: String,
    pub context_bundles: Vec<serde_json::Value>,
}

pub struct AssignmentHandler {
    queue: Arc<Mutex<Vec<TaskQueueEntry>>>,
}

impl AssignmentHandler {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn queue(&self) -> Arc<Mutex<Vec<TaskQueueEntry>>> {
        self.queue.clone()
    }

    pub fn start(
        &self,
        mut rx: broadcast::Receiver<MqttIncoming>,
        app: tauri::AppHandle,
    ) {
        let queue = self.queue.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(MqttIncoming::Assignment(assignment)) => {
                        handle_assignment(&queue, &app, assignment).await;
                    }
                    Ok(MqttIncoming::ContextBundle(todo_id, bundle)) => {
                        handle_context_bundle(&queue, &todo_id, bundle).await;
                    }
                    Ok(MqttIncoming::Registry(reg)) => {
                        use tauri::{Emitter, Manager};
                        // Store in managed state for later retrieval
                        let state: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<Option<crate::mqtt::types::WorkspaceRegistryMessage>>>> = app.state();
                        let mut guard = state.lock().await;
                        *guard = Some(reg.clone());
                        drop(guard);
                        let _ = app.emit("mqtt-registry", &serde_json::to_value(&reg).unwrap_or_default());
                    }
                    Ok(MqttIncoming::PlanState(plan)) => {
                        use tauri::{Emitter, Manager};
                        let mission_id = plan.mission_id.clone();
                        let cache: tauri::State<'_, Arc<Mutex<MissionStateCache>>> = app.state();
                        let mut guard = cache.lock().await;
                        // Decompose plan into phase/todo cache entries
                        // (load_weaver_plan calls store_plan internally)
                        let plan_json = serde_json::to_value(&plan).unwrap_or_default();
                        let _ = guard.load_weaver_plan(&plan_json);
                        drop(guard);
                        let _ = app.emit("mission-phases-updated", serde_json::json!({
                            "mission_id": mission_id
                        }));
                    }
                    Ok(MqttIncoming::PhaseState(phase)) => {
                        use tauri::{Emitter, Manager};
                        let mission_id = phase.mission_id.clone();
                        let cache: tauri::State<'_, Arc<Mutex<MissionStateCache>>> = app.state();
                        cache.lock().await.store_phase(phase);
                        let _ = app.emit("mission-phases-updated", serde_json::json!({
                            "mission_id": mission_id
                        }));
                    }
                    Ok(MqttIncoming::TodoState(todo)) => {
                        use tauri::{Emitter, Manager};
                        let mission_id = todo.mission_id.clone();
                        let cache: tauri::State<'_, Arc<Mutex<MissionStateCache>>> = app.state();
                        cache.lock().await.store_todo(todo);
                        let _ = app.emit("mission-phases-updated", serde_json::json!({
                            "mission_id": mission_id
                        }));
                    }
                    Ok(_) => {}
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        crate::debug_log::log_warn(&format!(
                            "[Assignment] Skipped {} messages (lagged)",
                            n
                        ));
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
    }

    pub async fn get_queue(&self) -> Vec<TaskQueueEntry> {
        self.queue.lock().await.clone()
    }

    pub async fn update_status(&self, mission_id: &str, phase_id: &str, status: &str) {
        let mut queue = self.queue.lock().await;
        if let Some(entry) = queue
            .iter_mut()
            .find(|e| e.mission_id == mission_id && e.phase_id == phase_id)
        {
            entry.status = status.to_string();
        }
    }

    /// Start auto-execution: when assignments come in, automatically execute them.
    pub fn start_auto_execute(
        &self,
        mut rx: broadcast::Receiver<MqttIncoming>,
        mqtt: Arc<Mutex<Option<MqttClient>>>,
        spawner: Arc<ClaudeCodeSpawner>,
        control: Arc<ControlHandler>,
        state_cache: Arc<Mutex<MissionStateCache>>,
        instance_id: String,
        workspace_mount: PathBuf,
        app: tauri::AppHandle,
    ) {
        let queue = self.queue.clone();

        tokio::spawn(async move {
            // Track which human phases we've already notified about
            let mut notified_phases: HashSet<String> = HashSet::new();

            loop {
                match rx.recv().await {
                    Ok(MqttIncoming::Assignment(assignment)) => {
                        // Queue it
                        handle_assignment(&queue, &app, assignment.clone()).await;

                        // Start execution in background
                        let mqtt_c = mqtt.clone();
                        let spawner_c = spawner.clone();
                        let control_c = control.clone();
                        let cache_c = state_cache.clone();
                        let iid = instance_id.clone();
                        let mount = workspace_mount.clone();
                        let app_c = app.clone();
                        let q = queue.clone();
                        let mid = assignment.mission_id.clone();
                        let pid = assignment.phase_id.clone();

                        tokio::spawn(async move {
                            // Update status to executing
                            update_queue_status(&q, &mid, &pid, "executing").await;

                            // Register abort signal
                            let abort_rx = control_c.register_abort(&mid, &pid).await;

                            let result = execute_assignment(
                                assignment,
                                &mount,
                                spawner_c,
                                mqtt_c,
                                cache_c,
                                iid,
                                abort_rx,
                                app_c,
                            )
                            .await;

                            match result {
                                Ok(()) => {
                                    update_queue_status(&q, &mid, &pid, "completed").await;
                                }
                                Err(e) => {
                                    crate::debug_log::log_error(&format!(
                                        "[Pipeline] Phase {} failed: {}",
                                        pid, e
                                    ));
                                    update_queue_status(&q, &mid, &pid, "failed").await;
                                }
                            }
                        });
                    }
                    Ok(MqttIncoming::Registry(reg)) => {
                        // Auto-setup workspaces for discovered missions
                        let mount = workspace_mount.clone();
                        let app_c = app.clone();
                        let sc = state_cache.clone();
                        tokio::spawn(async move {
                            let cache = sc.lock().await;
                            autopilot::setup_mission_workspaces(
                                &reg, &mount, &app_c, Some(&cache),
                            )
                            .await;
                        });
                    }
                    Ok(MqttIncoming::PhaseState(phase)) => {
                        // Cache the phase state, emit event, and check for human phases
                        let mission_id = phase.mission_id.clone();
                        state_cache.lock().await.store_phase(phase);
                        {
                            use tauri::Emitter;
                            let _ = app.emit("mission-phases-updated", serde_json::json!({
                                "mission_id": mission_id
                            }));
                        }
                        let cache = state_cache.lock().await;
                        autopilot::check_human_phases(
                            &cache,
                            &app,
                            &mut notified_phases,
                        )
                        .await;
                    }
                    Ok(MqttIncoming::TodoState(todo)) => {
                        let mission_id = todo.mission_id.clone();
                        state_cache.lock().await.store_todo(todo);
                        {
                            use tauri::Emitter;
                            let _ = app.emit("mission-phases-updated", serde_json::json!({
                                "mission_id": mission_id
                            }));
                        }
                    }
                    Ok(_) => {}
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
    }
}

async fn update_queue_status(
    queue: &Arc<Mutex<Vec<TaskQueueEntry>>>,
    mission_id: &str,
    phase_id: &str,
    status: &str,
) {
    let mut q = queue.lock().await;
    if let Some(entry) = q
        .iter_mut()
        .find(|e| e.mission_id == mission_id && e.phase_id == phase_id)
    {
        entry.status = status.to_string();
    }
}

async fn handle_assignment(
    queue: &Arc<Mutex<Vec<TaskQueueEntry>>>,
    app: &tauri::AppHandle,
    assignment: PhaseAssignment,
) {
    let entry = TaskQueueEntry {
        mission_id: assignment.mission_id.clone(),
        phase_id: assignment.phase_id.clone(),
        phase_name: assignment.phase_name.clone(),
        todos: assignment.todos.clone(),
        status: "queued".into(),
        received_at: Utc::now().to_rfc3339(),
        context_bundles: vec![],
    };

    crate::debug_log::log_info(&format!(
        "[Assignment] Queued: {} ({}) with {} todos",
        entry.phase_name,
        entry.phase_id,
        entry.todos.len()
    ));
    eprintln!(
        "[Assignment] Received: {} ({}) with {} todos",
        entry.phase_name, entry.phase_id, entry.todos.len()
    );
    // Debug: write to file
    let _ = std::fs::write("/tmp/weaver-assignment.log", format!(
        "Assignment received: phase={} mission={} todos={:?}\n",
        entry.phase_name, entry.mission_id, entry.todos
    ));

    queue.lock().await.push(entry);

    // Emit to frontend
    use tauri::Emitter;
    let _ = app.emit("assignment-received", &assignment);
}

async fn handle_context_bundle(
    queue: &Arc<Mutex<Vec<TaskQueueEntry>>>,
    todo_id: &str,
    bundle: serde_json::Value,
) {
    let mut queue = queue.lock().await;
    // Find the task that contains this todo_id
    if let Some(entry) = queue
        .iter_mut()
        .find(|e| e.todos.iter().any(|t| t == todo_id))
    {
        entry.context_bundles.push(bundle);
        crate::debug_log::log_info(&format!(
            "[Assignment] Context bundle received for todo {}",
            todo_id
        ));
    }
}
