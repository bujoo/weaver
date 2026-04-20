use crate::mqtt::types::{PhaseStateMessage, PlanStateMessage, TodoStateMessage};
use std::collections::HashMap;
use std::path::PathBuf;

/// Cache for retained MQTT state messages published by Brain.
/// Stores plans, phases, and todos so the executor can look up
/// full specs when building Claude Code prompts.
pub struct MissionStateCache {
    plans: HashMap<String, PlanStateMessage>,
    phases: HashMap<String, PhaseStateMessage>,   // key: "{mission_id}:{phase_id}"
    todos: HashMap<String, TodoStateMessage>,      // key: "{todo_id}"
    claims: HashMap<String, String>,              // mission_id -> instance_id
}

fn claims_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".config"));
    config_dir.join("contexthub-weaver").join("claims.json")
}

impl MissionStateCache {
    pub fn new() -> Self {
        let claims = Self::load_claims_from_disk();
        Self {
            plans: HashMap::new(),
            phases: HashMap::new(),
            todos: HashMap::new(),
            claims,
        }
    }

    fn load_claims_from_disk() -> HashMap<String, String> {
        let path = claims_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => HashMap::new(),
        }
    }

    fn persist_claims(&self) {
        let path = claims_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string(&self.claims) {
            let _ = std::fs::write(&path, json);
        }
    }

    pub fn store_plan(&mut self, msg: PlanStateMessage) {
        crate::debug_log::log_info(&format!(
            "[StateCache] Stored plan: {} ({})",
            msg.title, msg.mission_id
        ));
        self.plans.insert(msg.mission_id.clone(), msg);
    }

    pub fn store_phase(&mut self, msg: PhaseStateMessage) {
        let key = format!("{}:{}", msg.mission_id, msg.phase_id);
        // Never downgrade status: completed > executing > pending
        if let Some(existing) = self.phases.get(&key) {
            if existing.status == "completed" && msg.status != "completed" {
                crate::debug_log::log_info(&format!(
                    "[StateCache] Skipping phase {} status downgrade: {} -> {}",
                    key, existing.status, msg.status
                ));
                return;
            }
        }
        // Preserve existing metadata if incoming message has bare/missing fields
        let merged = if let Some(existing) = self.phases.get(&key) {
            PhaseStateMessage {
                name: if msg.name.is_empty() || msg.name == msg.phase_id {
                    existing.name.clone()
                } else {
                    msg.name
                },
                order: if msg.order == 0 && existing.order > 0 { existing.order } else { msg.order },
                todo_count: if msg.todo_count == 0 && existing.todo_count > 0 { existing.todo_count } else { msg.todo_count },
                completed_count: if msg.completed_count == 0 && existing.completed_count > 0 { existing.completed_count } else { msg.completed_count },
                ..msg
            }
        } else {
            msg
        };
        crate::debug_log::log_info(&format!(
            "[StateCache] Stored phase: {} ({}) status={}",
            merged.name, key, merged.status
        ));
        self.phases.insert(key, merged);
    }

    pub fn store_todo(&mut self, msg: TodoStateMessage) {
        // Never downgrade status: completed > executing > pending
        if let Some(existing) = self.todos.get(&msg.todo_id) {
            if existing.status == "completed" && msg.status != "completed" {
                crate::debug_log::log_info(&format!(
                    "[StateCache] Skipping todo {} status downgrade: {} -> {}",
                    msg.todo_id, existing.status, msg.status
                ));
                return;
            }
        }
        crate::debug_log::log_info(&format!(
            "[StateCache] Stored todo: {} role={} status={} has_spec={}",
            msg.todo_id, msg.role, msg.status, msg.spec.is_some()
        ));
        self.todos.insert(msg.todo_id.clone(), msg);
    }

    pub fn get_plan(&self, mission_id: &str) -> Option<&PlanStateMessage> {
        self.plans.get(mission_id)
    }

    pub fn get_phase(&self, mission_id: &str, phase_id: &str) -> Option<&PhaseStateMessage> {
        let key = format!("{}:{}", mission_id, phase_id);
        self.phases.get(&key)
    }

    /// Update a todo's status (e.g., when Claude Code completes it via channel).
    pub fn update_todo_status(&mut self, todo_id: &str, status: &str) -> bool {
        if let Some(todo) = self.todos.get_mut(todo_id) {
            todo.status = status.to_string();
            true
        } else {
            false
        }
    }

    /// Mark a phase as completed.
    pub fn mark_phase_completed(&mut self, mission_id: &str, phase_id: &str) {
        let key = format!("{}:{}", mission_id, phase_id);
        if let Some(phase) = self.phases.get_mut(&key) {
            phase.status = "completed".to_string();
            phase.completed_count = phase.todo_count;
        }
    }

    /// Update a phase's completed count.
    pub fn increment_phase_completed(&mut self, mission_id: &str, phase_id: &str) {
        let key = format!("{}:{}", mission_id, phase_id);
        if let Some(phase) = self.phases.get_mut(&key) {
            phase.completed_count += 1;
            // Auto-set phase status to completed when all done
            if phase.completed_count >= phase.todo_count && phase.todo_count > 0 {
                phase.status = "completed".to_string();
            }
        }
    }

    /// Resolve a short or full mission ID to the full ID in the cache.
    pub fn resolve_mission_id(&self, short_or_full: &str) -> Option<String> {
        if self.plans.contains_key(short_or_full) {
            return Some(short_or_full.to_string());
        }
        self.plans
            .keys()
            .find(|k| k.starts_with(short_or_full))
            .cloned()
    }

    pub fn get_todo(&self, todo_id: &str) -> Option<&TodoStateMessage> {
        self.todos.get(todo_id)
    }

    /// Get all todos for a given phase.
    pub fn get_todos_for_phase(&self, mission_id: &str, phase_id: &str) -> Vec<&TodoStateMessage> {
        self.todos
            .values()
            .filter(|t| t.mission_id == mission_id && t.phase_id == phase_id)
            .collect()
    }

    /// Snapshot of all cached state for frontend/debug queries.
    pub fn snapshot(&self) -> serde_json::Value {
        let phase_details: Vec<serde_json::Value> = self.phases.values().map(|p| {
            let todo_count_actual = self.todos.values()
                .filter(|t| t.mission_id == p.mission_id && t.phase_id == p.phase_id)
                .count();
            serde_json::json!({
                "phase_id": p.phase_id,
                "name": p.name,
                "order": p.order,
                "status": p.status,
                "todo_count_declared": p.todo_count,
                "todo_count_actual": todo_count_actual,
                "completed_count": p.completed_count,
            })
        }).collect();
        serde_json::json!({
            "plans": self.plans.len(),
            "phases": self.phases.len(),
            "todos": self.todos.len(),
            "plan_ids": self.plans.keys().collect::<Vec<_>>(),
            "todo_ids": self.todos.keys().collect::<Vec<_>>(),
            "phase_details": phase_details,
        })
    }

    /// Load a full WeaverPlan JSON (as Brain produces it) and decompose into
    /// PlanState, PhaseState, and TodoState entries.
    /// Works for both fixture files and full-plan retained MQTT messages.
    pub fn load_weaver_plan(&mut self, plan_json: &serde_json::Value) -> Result<LoadResult, String> {
        let mission_id = plan_json
            .get("mission_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing mission_id")?
            .to_string();
        let workspace = plan_json
            .get("workspace")
            .and_then(|v| v.as_str())
            .unwrap_or("dev")
            .to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // Store plan
        let plan_msg = PlanStateMessage {
            mission_id: mission_id.clone(),
            workspace: workspace.clone(),
            version: plan_json.get("version").and_then(|v| v.as_u64()).unwrap_or(1) as u32,
            title: plan_json
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            scope: plan_json
                .get("scope")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            status: plan_json
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("draft")
                .to_string(),
            phases: plan_json
                .get("phases")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default(),
            published_at: now.clone(),
            mission_context: plan_json
                .get("mission_context")
                .and_then(|v| v.as_str())
                .map(String::from),
            roles: plan_json
                .get("roles")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default(),
        };
        self.store_plan(plan_msg);

        let mut phase_count = 0u32;
        let mut todo_count = 0u32;

        // Decompose phases and todos
        let phases = plan_json
            .get("phases")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        for phase in &phases {
            let phase_id = phase
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let todos_arr = phase
                .get("todos")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            let phase_msg = PhaseStateMessage {
                mission_id: mission_id.clone(),
                phase_id: phase_id.clone(),
                name: phase
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                order: phase.get("order").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                status: phase
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("pending")
                    .to_string(),
                blocked_by: phase
                    .get("blocked_by")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                config: phase.get("config").cloned().unwrap_or_default(),
                todo_count: todos_arr.len() as u32,
                completed_count: 0,
                published_at: now.clone(),
                execution_target: phase.get("execution_target").cloned(),
            };
            self.store_phase(phase_msg);
            phase_count += 1;

            for todo in &todos_arr {
                let todo_id = todo
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let todo_msg = TodoStateMessage {
                    mission_id: mission_id.clone(),
                    phase_id: phase_id.clone(),
                    todo_id: todo_id.clone(),
                    role: todo
                        .get("role")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: todo
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    status: todo
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("pending")
                        .to_string(),
                    file_paths: todo
                        .get("file_paths")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    blocked_by: todo
                        .get("blocked_by")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    spec: todo.get("spec").cloned(),
                    published_at: now.clone(),
                };
                self.store_todo(todo_msg);
                todo_count += 1;
            }
        }

        Ok(LoadResult {
            mission_id,
            title: plan_json
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            phases: phase_count,
            todos: todo_count,
        })
    }

    /// Get all cached phases (for iteration in autopilot).
    pub fn all_phases(&self) -> impl Iterator<Item = &PhaseStateMessage> {
        self.phases.values()
    }

    /// Get all phases for a mission, sorted by order.
    pub fn get_phases_for_mission(&self, mission_id: &str) -> Vec<&PhaseStateMessage> {
        let mut phases: Vec<&PhaseStateMessage> = self
            .phases
            .values()
            .filter(|p| p.mission_id == mission_id)
            .collect();
        phases.sort_by_key(|p| p.order);
        phases
    }

    /// Get the currently active phase (first non-completed by order).
    pub fn get_active_phase(&self, mission_id: &str) -> Option<&PhaseStateMessage> {
        self.get_phases_for_mission(mission_id)
            .into_iter()
            .find(|p| p.status != "completed" && p.status != "skipped")
    }

    // ── Mission lifecycle ──────────────────────────────────────────

    /// Purge all cached data for a mission. Returns topic info for MQTT cleanup.
    pub fn purge_mission(&mut self, mission_id: &str) -> PurgeResult {
        let phase_ids: Vec<String> = self.phases.keys()
            .filter(|k| k.starts_with(&format!("{}:", mission_id)))
            .map(|k| k.split(':').nth(1).unwrap_or("").to_string())
            .collect();
        let todo_ids: Vec<String> = self.todos.values()
            .filter(|t| t.mission_id == mission_id)
            .map(|t| t.todo_id.clone())
            .collect();

        self.plans.remove(mission_id);
        self.phases.retain(|k, _| !k.starts_with(&format!("{}:", mission_id)));
        self.todos.retain(|_, t| t.mission_id != mission_id);
        self.claims.remove(mission_id);

        crate::debug_log::log_info(&format!(
            "[StateCache] Purged mission {}: {} phases, {} todos",
            mission_id, phase_ids.len(), todo_ids.len()
        ));

        PurgeResult { phase_ids, todo_ids }
    }

    /// Claim a mission for an instance. Persists to disk.
    pub fn claim_mission(&mut self, mission_id: &str, instance_id: &str) {
        self.claims.insert(mission_id.to_string(), instance_id.to_string());
        self.persist_claims();
    }

    /// Release a mission claim. Persists to disk.
    pub fn release_mission(&mut self, mission_id: &str) {
        self.claims.remove(mission_id);
        self.persist_claims();
    }

    /// Get the instance that claimed a mission.
    pub fn get_claim(&self, mission_id: &str) -> Option<&str> {
        self.claims.get(mission_id).map(|s| s.as_str())
    }

    /// Get all claimed mission IDs (for heartbeat).
    pub fn get_claimed_mission_ids(&self) -> Vec<String> {
        self.claims.keys().cloned().collect()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadResult {
    pub mission_id: String,
    pub title: String,
    pub phases: u32,
    pub todos: u32,
}

#[derive(Debug, Clone)]
pub struct PurgeResult {
    pub phase_ids: Vec<String>,
    pub todo_ids: Vec<String>,
}
