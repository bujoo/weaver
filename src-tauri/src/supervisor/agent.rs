use crate::debug_log;
use crate::mqtt::types::MqttIncoming;
use crate::supervisor::rules::SupervisorRules;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::sync::{broadcast, mpsc, Mutex};

// ── Hook events from Claude Code ──────────────────────────────────────

/// Events forwarded from the web_server hook endpoints to the supervisor.
#[derive(Clone, Debug)]
pub enum HookEvent {
    SessionStart {
        mission_id: String,
        phase_id: String,
    },
    ToolUse {
        mission_id: String,
        phase_id: String,
        todo_id: String,
        tool: String,
        file: Option<String>,
    },
    SessionStop {
        mission_id: String,
        phase_id: String,
        exit_code: i32,
    },
    TodoCompleted {
        mission_id: String,
        phase_id: String,
        todo_id: String,
    },
    Error {
        mission_id: String,
        phase_id: String,
        todo_id: String,
        error: String,
    },
}

// ── Observations ──────────────────────────────────────────────────────

/// An observation is something the supervisor noticed.
/// Emitted to the frontend via Tauri events.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Observation {
    RetryLoop {
        mission_id: String,
        phase_id: String,
        todo_id: String,
        error_pattern: String,
        count: u32,
    },
    IdleDetected {
        mission_id: String,
        phase_id: String,
        todo_id: String,
        idle_secs: u64,
    },
    SessionCrashed {
        mission_id: String,
        phase_id: String,
        exit_code: i32,
    },
    PermissionTimeout {
        mission_id: String,
        phase_id: String,
    },
    ContextIncomplete {
        mission_id: String,
        missing: Vec<String>,
    },
    ValidationPassed {
        mission_id: String,
    },
    PhaseCompleted {
        mission_id: String,
        phase_id: String,
        duration_secs: u64,
    },
}

// ── Interventions ─────────────────────────────────────────────────────

/// An intervention is an action the supervisor took (or suggests).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Intervention {
    SentHint {
        mission_id: String,
        todo_id: String,
        hint: String,
    },
    RestartedSession {
        mission_id: String,
        phase_id: String,
    },
    RequeuedTodo {
        mission_id: String,
        todo_id: String,
    },
    EscalatedToHuman {
        mission_id: String,
        reason: String,
    },
}

// ── Serializable wrappers for frontend queries ────────────────────────

/// A timestamped observation record suitable for JSON serialization.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationRecord {
    pub elapsed_secs: u64,
    pub observation: Observation,
}

/// A timestamped intervention record suitable for JSON serialization.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterventionRecord {
    pub elapsed_secs: u64,
    pub intervention: Intervention,
}

// ── Supervisor Agent ──────────────────────────────────────────────────

pub struct SupervisorAgent {
    rules: Arc<Mutex<SupervisorRules>>,
    observations: Arc<Mutex<Vec<(Instant, Observation)>>>,
    interventions: Arc<Mutex<Vec<(Instant, Intervention)>>>,
    /// Track retry patterns per todo: todo_id -> Vec<(when, error_message)>
    retry_tracker: Arc<Mutex<HashMap<String, Vec<(Instant, String)>>>>,
    /// Track last activity per todo: todo_id -> last_seen
    activity_tracker: Arc<Mutex<HashMap<String, Instant>>>,
    /// Track auto-restart count per mission
    restart_counts: Arc<Mutex<HashMap<String, u32>>>,
    /// When the agent was created (used as epoch for elapsed_secs)
    created_at: Instant,
    /// Internal broadcast sender so external code can feed MQTT messages
    /// into the supervisor after MQTT connects.
    mqtt_feed_tx: broadcast::Sender<MqttIncoming>,
}

impl SupervisorAgent {
    pub fn new(rules: SupervisorRules) -> Self {
        let (mqtt_feed_tx, _) = broadcast::channel::<MqttIncoming>(128);
        Self {
            rules: Arc::new(Mutex::new(rules)),
            observations: Arc::new(Mutex::new(Vec::new())),
            interventions: Arc::new(Mutex::new(Vec::new())),
            retry_tracker: Arc::new(Mutex::new(HashMap::new())),
            activity_tracker: Arc::new(Mutex::new(HashMap::new())),
            restart_counts: Arc::new(Mutex::new(HashMap::new())),
            created_at: Instant::now(),
            mqtt_feed_tx,
        }
    }

    /// Feed MQTT messages from an external broadcast receiver into the supervisor.
    /// Call this after MQTT connects to bridge the MqttClient broadcast into
    /// the supervisor's internal channel.
    pub fn attach_mqtt(&self, mut source: broadcast::Receiver<MqttIncoming>) {
        let tx = self.mqtt_feed_tx.clone();
        tauri::async_runtime::spawn(async move {
            loop {
                match source.recv().await {
                    Ok(msg) => {
                        let _ = tx.send(msg);
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        debug_log::log_warn(&format!(
                            "[Supervisor] MQTT forwarder lagged {} messages", n
                        ));
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        debug_log::log_info("[Supervisor] MQTT source closed");
                        break;
                    }
                }
            }
        });
    }

    /// Start the supervisor background task.
    /// Listens to MQTT messages (via internal feed) and hook events concurrently.
    pub fn start(
        &self,
        app: AppHandle,
        mut hook_rx: mpsc::Receiver<HookEvent>,
    ) {
        let rules = self.rules.clone();
        let observations = self.observations.clone();
        let interventions = self.interventions.clone();
        let retry_tracker = self.retry_tracker.clone();
        let activity_tracker = self.activity_tracker.clone();
        let restart_counts = self.restart_counts.clone();
        let created_at = self.created_at;
        let mut mqtt_rx = self.mqtt_feed_tx.subscribe();

        tauri::async_runtime::spawn(async move {
            debug_log::log_info("[Supervisor] Background task started");

            // Tick interval for periodic checks (idle detection, permission timeouts)
            let mut idle_check = tokio::time::interval(tokio::time::Duration::from_secs(30));

            loop {
                tokio::select! {
                    msg = mqtt_rx.recv() => {
                        match msg {
                            Ok(incoming) => {
                                Self::process_mqtt_static(
                                    &app, &rules, &observations, &interventions,
                                    &retry_tracker, &activity_tracker, &restart_counts,
                                    created_at, incoming,
                                ).await;
                            }
                            Err(broadcast::error::RecvError::Lagged(n)) => {
                                debug_log::log_warn(&format!(
                                    "[Supervisor] Lagged {} MQTT messages", n
                                ));
                            }
                            Err(broadcast::error::RecvError::Closed) => {
                                debug_log::log_info("[Supervisor] MQTT broadcast closed, stopping");
                                break;
                            }
                        }
                    }
                    Some(event) = hook_rx.recv() => {
                        Self::process_hook_static(
                            &app, &rules, &observations, &interventions,
                            &retry_tracker, &activity_tracker, &restart_counts,
                            created_at, event,
                        ).await;
                    }
                    _ = idle_check.tick() => {
                        Self::check_idle_static(
                            &app, &rules, &observations, &interventions,
                            &activity_tracker, &restart_counts,
                            created_at,
                        ).await;
                    }
                }
            }
        });
    }

    // ── MQTT message processing ───────────────────────────────────────

    async fn process_mqtt_static(
        app: &AppHandle,
        rules: &Arc<Mutex<SupervisorRules>>,
        observations: &Arc<Mutex<Vec<(Instant, Observation)>>>,
        interventions: &Arc<Mutex<Vec<(Instant, Intervention)>>>,
        retry_tracker: &Arc<Mutex<HashMap<String, Vec<(Instant, String)>>>>,
        activity_tracker: &Arc<Mutex<HashMap<String, Instant>>>,
        restart_counts: &Arc<Mutex<HashMap<String, u32>>>,
        created_at: Instant,
        msg: MqttIncoming,
    ) {
        match msg {
            MqttIncoming::TodoState(todo) => {
                // Update activity tracker
                {
                    let mut tracker = activity_tracker.lock().await;
                    tracker.insert(todo.todo_id.clone(), Instant::now());
                }

                // Check for completed phase
                if todo.status == "completed" {
                    let obs = Observation::ValidationPassed {
                        mission_id: todo.mission_id.clone(),
                    };
                    Self::record_and_emit(app, observations, created_at, &obs).await;
                }
            }
            MqttIncoming::PhaseState(phase) => {
                if phase.status == "completed" {
                    // We don't know exact duration, so use 0 -- real duration
                    // could be computed from phase start tracking if needed.
                    let obs = Observation::PhaseCompleted {
                        mission_id: phase.mission_id.clone(),
                        phase_id: phase.phase_id.clone(),
                        duration_secs: 0,
                    };
                    Self::record_and_emit(app, observations, created_at, &obs).await;
                }
            }
            MqttIncoming::Assignment(assignment) => {
                // Track activity for all assigned todos
                let mut tracker = activity_tracker.lock().await;
                for todo_id in &assignment.todos {
                    tracker.insert(todo_id.clone(), Instant::now());
                }
            }
            MqttIncoming::ContextBundle(todo_id, _bundle) => {
                // Activity update
                let mut tracker = activity_tracker.lock().await;
                tracker.insert(todo_id, Instant::now());
            }
            // Control, Registry, PlanState, Unknown -- observed but no action needed
            _ => {}
        }

        // Suppress unused warnings for state refs that will be used as the
        // supervisor gains more rules.
        let _ = (rules, interventions, retry_tracker, restart_counts);
    }

    // ── Hook event processing ─────────────────────────────────────────

    async fn process_hook_static(
        app: &AppHandle,
        rules: &Arc<Mutex<SupervisorRules>>,
        observations: &Arc<Mutex<Vec<(Instant, Observation)>>>,
        interventions: &Arc<Mutex<Vec<(Instant, Intervention)>>>,
        retry_tracker: &Arc<Mutex<HashMap<String, Vec<(Instant, String)>>>>,
        activity_tracker: &Arc<Mutex<HashMap<String, Instant>>>,
        restart_counts: &Arc<Mutex<HashMap<String, u32>>>,
        created_at: Instant,
        event: HookEvent,
    ) {
        match event {
            HookEvent::SessionStart {
                mission_id,
                phase_id,
            } => {
                debug_log::log_info(&format!(
                    "[Supervisor] Session started: mission={} phase={}",
                    &mission_id[..8.min(mission_id.len())],
                    &phase_id[..8.min(phase_id.len())],
                ));
                // Reset activity tracker for this phase
                let mut tracker = activity_tracker.lock().await;
                tracker.insert(
                    format!("session:{}:{}", mission_id, phase_id),
                    Instant::now(),
                );
            }

            HookEvent::ToolUse {
                mission_id: _,
                phase_id: _,
                todo_id,
                tool: _,
                file: _,
            } => {
                // Update activity tracker
                let mut tracker = activity_tracker.lock().await;
                tracker.insert(todo_id, Instant::now());
            }

            HookEvent::SessionStop {
                mission_id,
                phase_id,
                exit_code,
            } => {
                if exit_code != 0 {
                    let obs = Observation::SessionCrashed {
                        mission_id: mission_id.clone(),
                        phase_id: phase_id.clone(),
                        exit_code,
                    };
                    Self::record_and_emit(app, observations, created_at, &obs).await;

                    // Handle auto-restart if enabled
                    let current_rules = rules.lock().await.clone();
                    if current_rules.crash_auto_restart {
                        let mut counts = restart_counts.lock().await;
                        let count = counts.entry(mission_id.clone()).or_insert(0);
                        *count += 1;

                        if *count <= current_rules.max_auto_restarts {
                            let intervention = Intervention::RestartedSession {
                                mission_id: mission_id.clone(),
                                phase_id: phase_id.clone(),
                            };
                            debug_log::log_info(&format!(
                                "[Supervisor] Auto-restart #{} for mission {}",
                                count,
                                &mission_id[..8.min(mission_id.len())],
                            ));
                            Self::record_and_emit_intervention(
                                app,
                                interventions,
                                created_at,
                                &intervention,
                            )
                            .await;
                            // Actual restart happens via the executor -- for now just log.
                            // The frontend can trigger restart from the intervention event.
                        } else {
                            let intervention = Intervention::EscalatedToHuman {
                                mission_id: mission_id.clone(),
                                reason: format!(
                                    "Session crashed {} times (exit code {}), exceeded max auto-restarts",
                                    count, exit_code
                                ),
                            };
                            Self::record_and_emit_intervention(
                                app,
                                interventions,
                                created_at,
                                &intervention,
                            )
                            .await;
                        }
                    }
                }
            }

            HookEvent::TodoCompleted {
                mission_id,
                phase_id: _,
                todo_id,
            } => {
                // Clear retry tracker for this todo
                {
                    let mut tracker = retry_tracker.lock().await;
                    tracker.remove(&todo_id);
                }
                // Update activity
                {
                    let mut tracker = activity_tracker.lock().await;
                    tracker.insert(todo_id, Instant::now());
                }
                debug_log::log_info(&format!(
                    "[Supervisor] Todo completed for mission {}",
                    &mission_id[..8.min(mission_id.len())],
                ));
            }

            HookEvent::Error {
                mission_id,
                phase_id,
                todo_id,
                error,
            } => {
                let current_rules = rules.lock().await.clone();
                let mut tracker = retry_tracker.lock().await;
                let entries = tracker.entry(todo_id.clone()).or_default();

                // Purge old entries outside the window
                let window = std::time::Duration::from_secs(current_rules.retry_window_secs);
                let cutoff = Instant::now() - window;
                entries.retain(|(t, _)| *t > cutoff);

                // Add new error
                entries.push((Instant::now(), error.clone()));

                // Check threshold
                if entries.len() as u32 >= current_rules.retry_threshold {
                    let obs = Observation::RetryLoop {
                        mission_id: mission_id.clone(),
                        phase_id: phase_id.clone(),
                        todo_id: todo_id.clone(),
                        error_pattern: error.clone(),
                        count: entries.len() as u32,
                    };
                    Self::record_and_emit(app, observations, created_at, &obs).await;

                    // If autopilot, escalate
                    if current_rules.autopilot {
                        let intervention = Intervention::EscalatedToHuman {
                            mission_id,
                            reason: format!(
                                "Retry loop detected on todo {}: {} occurrences of '{}'",
                                &todo_id[..8.min(todo_id.len())],
                                entries.len(),
                                &error[..80.min(error.len())],
                            ),
                        };
                        drop(tracker); // release lock before emitting
                        Self::record_and_emit_intervention(
                            app,
                            interventions,
                            created_at,
                            &intervention,
                        )
                        .await;
                    }
                }
            }
        }

        // Suppress unused warning for restart_counts in hook paths that don't use it
        let _ = restart_counts;
    }

    // ── Periodic idle check ───────────────────────────────────────────

    async fn check_idle_static(
        app: &AppHandle,
        rules: &Arc<Mutex<SupervisorRules>>,
        observations: &Arc<Mutex<Vec<(Instant, Observation)>>>,
        _interventions: &Arc<Mutex<Vec<(Instant, Intervention)>>>,
        activity_tracker: &Arc<Mutex<HashMap<String, Instant>>>,
        _restart_counts: &Arc<Mutex<HashMap<String, u32>>>,
        created_at: Instant,
    ) {
        let current_rules = rules.lock().await.clone();
        let timeout = std::time::Duration::from_secs(current_rules.idle_timeout_secs);
        let now = Instant::now();

        let tracker = activity_tracker.lock().await;
        for (key, last_seen) in tracker.iter() {
            if now.duration_since(*last_seen) > timeout {
                // Parse key to extract mission/phase/todo info
                // Keys are either "todo_id" or "session:mission_id:phase_id"
                if let Some(rest) = key.strip_prefix("session:") {
                    let parts: Vec<&str> = rest.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let obs = Observation::IdleDetected {
                            mission_id: parts[0].to_string(),
                            phase_id: parts[1].to_string(),
                            todo_id: String::new(),
                            idle_secs: now.duration_since(*last_seen).as_secs(),
                        };
                        Self::record_and_emit(app, observations, created_at, &obs).await;
                    }
                }
            }
        }
    }

    // ── Helpers ────────────────────────────────────────────────────────

    async fn record_and_emit(
        app: &AppHandle,
        observations: &Arc<Mutex<Vec<(Instant, Observation)>>>,
        _created_at: Instant,
        obs: &Observation,
    ) {
        {
            let mut log = observations.lock().await;
            log.push((Instant::now(), obs.clone()));
        }
        if let Err(e) = app.emit("supervisor-observation", obs) {
            debug_log::log_error(&format!("[Supervisor] Failed to emit observation: {}", e));
        }
    }

    async fn record_and_emit_intervention(
        app: &AppHandle,
        interventions: &Arc<Mutex<Vec<(Instant, Intervention)>>>,
        _created_at: Instant,
        intervention: &Intervention,
    ) {
        {
            let mut log = interventions.lock().await;
            log.push((Instant::now(), intervention.clone()));
        }
        if let Err(e) = app.emit("supervisor-intervention", intervention) {
            debug_log::log_error(&format!(
                "[Supervisor] Failed to emit intervention: {}",
                e
            ));
        }
    }

    // ── Public query methods ──────────────────────────────────────────

    /// Get all observations as serializable records.
    pub async fn get_observations(&self) -> Vec<ObservationRecord> {
        let log = self.observations.lock().await;
        log.iter()
            .map(|(instant, obs)| ObservationRecord {
                elapsed_secs: instant.duration_since(self.created_at).as_secs(),
                observation: obs.clone(),
            })
            .collect()
    }

    /// Get all interventions as serializable records.
    pub async fn get_interventions(&self) -> Vec<InterventionRecord> {
        let log = self.interventions.lock().await;
        log.iter()
            .map(|(instant, intervention)| InterventionRecord {
                elapsed_secs: instant.duration_since(self.created_at).as_secs(),
                intervention: intervention.clone(),
            })
            .collect()
    }

    /// Get the current rules.
    pub async fn get_rules(&self) -> SupervisorRules {
        self.rules.lock().await.clone()
    }

    /// Update the rules at runtime.
    pub async fn update_rules(&self, new_rules: SupervisorRules) {
        let mut rules = self.rules.lock().await;
        *rules = new_rules;
        debug_log::log_info("[Supervisor] Rules updated");
    }
}
