pub mod actions;
pub mod aggregator;
pub mod engine;
pub mod types;

pub use types::{ConductorConfig, ConductorDecision, ConductorEvent, DecisionRecord};

use crate::conductor::actions::ActionExecutor;
use crate::conductor::aggregator::EventAggregator;
use crate::conductor::engine::ConductorEngine;
use crate::executor::spawner::ClaudeCodeSpawner;
use crate::mqtt::client::MqttClient;
use crate::mqtt::state_cache::MissionStateCache;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Weavy -- the AI conductor agent for Weaver.
/// Receives events from Claude Code sessions via hooks/channel,
/// calls Bedrock for decisions, and executes them.
pub struct ConductorAgent {
    config: Arc<Mutex<ConductorConfig>>,
    decisions: Arc<Mutex<Vec<DecisionRecord>>>,
}

impl ConductorAgent {
    pub fn new(config: ConductorConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            decisions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start the background decision loop.
    pub fn start(
        &self,
        mut event_rx: mpsc::Receiver<ConductorEvent>,
        mqtt: Arc<Mutex<Option<MqttClient>>>,
        spawner: Arc<ClaudeCodeSpawner>,
        state_cache: Arc<Mutex<MissionStateCache>>,
        app: tauri::AppHandle,
    ) {
        let config = self.config.clone();
        let decisions = self.decisions.clone();

        tokio::spawn(async move {
            let cfg = config.lock().await.clone();
            let engine = ConductorEngine::new(cfg.clone());
            let executor = ActionExecutor::new(
                mqtt,
                spawner,
                state_cache.clone(),
                app.clone(),
            );
            let mut aggregator = EventAggregator::new(
                cfg.min_decision_interval_secs,
                cfg.max_event_buffer,
            );
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(
                cfg.min_decision_interval_secs,
            ));

            crate::debug_log::log_info("[Weavy] Conductor agent started");

            loop {
                tokio::select! {
                    Some(event) = event_rx.recv() => {
                        let cfg = config.lock().await;
                        if !cfg.enabled {
                            Self::fallback_logic(&event).await;
                            continue;
                        }
                        drop(cfg);

                        if let Some((events, tier)) = aggregator.push(event) {
                            let cache = state_cache.lock().await;
                            match engine.decide(&events, &*cache, tier.clone()).await {
                                Ok((decision, input_tok, output_tok)) => {
                                    let record = DecisionRecord {
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                        model_used: tier.model_id().to_string(),
                                        decision: decision.clone(),
                                        input_tokens: input_tok,
                                        output_tokens: output_tok,
                                        event_count: events.len(),
                                    };

                                    crate::debug_log::log_info(&format!(
                                        "[Weavy] Decision: {:?} ({}tok in, {}tok out)",
                                        decision, input_tok, output_tok
                                    ));

                                    decisions.lock().await.push(record.clone());

                                    use tauri::Emitter;
                                    let _ = app.emit("conductor-decision", &record);

                                    drop(cache);
                                    if let Err(e) = executor.execute(&decision).await {
                                        crate::debug_log::log_error(&format!(
                                            "[Weavy] Action failed: {}", e
                                        ));
                                    }
                                }
                                Err(e) => {
                                    crate::debug_log::log_error(&format!(
                                        "[Weavy] Decision error: {}", e
                                    ));
                                }
                            }
                        }
                    }
                    _ = tick.tick() => {
                        let cfg = config.lock().await;
                        if !cfg.enabled { continue; }
                        drop(cfg);

                        if let Some((events, tier)) = aggregator.tick() {
                            let cache = state_cache.lock().await;
                            match engine.decide(&events, &*cache, tier.clone()).await {
                                Ok((decision, input_tok, output_tok)) => {
                                    let record = DecisionRecord {
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                        model_used: tier.model_id().to_string(),
                                        decision: decision.clone(),
                                        input_tokens: input_tok,
                                        output_tokens: output_tok,
                                        event_count: events.len(),
                                    };
                                    decisions.lock().await.push(record.clone());
                                    use tauri::Emitter;
                                    let _ = app.emit("conductor-decision", &record);
                                    drop(cache);
                                    if let Err(e) = executor.execute(&decision).await {
                                        crate::debug_log::log_error(&format!(
                                            "[Weavy] Action failed: {}", e
                                        ));
                                    }
                                }
                                Err(e) => {
                                    crate::debug_log::log_error(&format!(
                                        "[Weavy] Tick decision error: {}", e
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    /// Fallback when conductor is disabled -- just log events.
    async fn fallback_logic(event: &ConductorEvent) {
        match event {
            ConductorEvent::PhaseCompleted {
                phase_id, ..
            } => {
                crate::debug_log::log_info(&format!(
                    "[Weavy/Fallback] Phase {} completed -- manual transition required",
                    phase_id
                ));
            }
            ConductorEvent::TodoCompleted { todo_id, .. } => {
                crate::debug_log::log_info(&format!(
                    "[Weavy/Fallback] Todo {} completed",
                    todo_id
                ));
            }
            _ => {}
        }
    }

    // ── Public query methods ──

    pub async fn get_decisions(&self) -> Vec<DecisionRecord> {
        self.decisions.lock().await.clone()
    }

    pub async fn get_config(&self) -> ConductorConfig {
        self.config.lock().await.clone()
    }

    pub async fn update_config(&self, new_config: ConductorConfig) {
        *self.config.lock().await = new_config;
    }
}
