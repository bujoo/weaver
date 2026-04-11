// The tauri_nspanel macro expands panel_event! in a way that requires `-> ()` syntax,
// which clippy flags as unused_unit. Suppress it since we cannot change the macro invocation.
#![cfg_attr(target_os = "macos", allow(clippy::unused_unit))]

// Desktop-only modules
#[cfg(not(mobile))]
pub mod actions;
#[cfg(not(mobile))]
pub mod auth;
#[cfg(not(mobile))]
pub mod polling;
#[cfg(not(mobile))]
pub mod web_server;
#[cfg(not(mobile))]
pub mod debug_log;
#[cfg(not(mobile))]
pub mod conductor;
#[cfg(not(mobile))]
pub mod executor;
#[cfg(not(mobile))]
pub mod mqtt;
#[cfg(not(mobile))]
pub mod settings;
#[cfg(not(mobile))]
pub mod supervisor;
#[cfg(not(mobile))]
pub mod workspace;

// Shared modules (types used by both desktop and mobile builds)
pub mod session;

#[cfg(not(mobile))]
use actions::{open_session as open_session_action, stop_session as stop_session_action};
#[cfg(not(mobile))]
use polling::{detect_and_enrich_sessions, start_polling, Session};
use serde::Serialize;
use session::{extract_messages, parse_all_entries, ImageBlock, MessageType};
#[cfg(not(mobile))]
use std::sync::Arc;
#[cfg(not(mobile))]
use std::time::Duration;
#[cfg(not(mobile))]
use tokio::sync::Mutex;
#[cfg(not(mobile))]
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, PhysicalPosition,
};
use tauri::{AppHandle, Manager};
#[cfg(target_os = "macos")]
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt as PanelManagerExt, PanelLevel, StyleMask,
    WebviewWindowExt as PanelExt,
};

// ── Shared types ────────────────────────────────────────────────────

/// Conversation structure for the frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub session_id: String,
    pub messages: Vec<ConversationMessage>,
}

/// Individual message in a conversation
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessage {
    pub timestamp: String,
    pub message_type: MessageType,
    pub content: String,
    /// Images attached to this message (screenshots pasted by the user)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<ImageBlock>,
}

// ── Desktop-only commands ───────────────────────────────────────────

#[cfg(not(mobile))]
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg(not(mobile))]
#[tauri::command]
async fn get_sessions() -> Result<Vec<Session>, String> {
    polling::detect_and_enrich_sessions().map(|(sessions, _)| sessions)
}

/// Core logic for getting conversation data (shared by Tauri command and WS handler)
#[cfg(not(mobile))]
pub fn get_conversation_data(session_id: &str) -> Result<Conversation, String> {
    let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    let claude_projects_dir = home_dir.join(".claude").join("projects");

    let entries = std::fs::read_dir(&claude_projects_dir)
        .map_err(|e| format!("Failed to read projects directory: {}", e))?;

    let session_filename = format!("{}.jsonl", session_id);

    for entry in entries.flatten() {
        let project_path = entry.path();
        if !project_path.is_dir() {
            continue;
        }

        let session_file = project_path.join(&session_filename);
        if session_file.exists() {
            let entries = parse_all_entries(&session_file)
                .map_err(|e| format!("Failed to parse session file: {}", e))?;

            let messages = extract_messages(&entries);

            let conversation_messages: Vec<ConversationMessage> = messages
                .into_iter()
                .map(|(timestamp, msg_type, content, images)| ConversationMessage {
                    timestamp,
                    message_type: msg_type,
                    content,
                    images,
                })
                .collect();

            return Ok(Conversation {
                session_id: session_id.to_string(),
                messages: conversation_messages,
            });
        }
    }

    Err(format!(
        "Session {} not found in any project directory",
        session_id
    ))
}

#[cfg(not(mobile))]
#[tauri::command]
async fn get_conversation(session_id: String) -> Result<Conversation, String> {
    get_conversation_data(&session_id)
}

#[cfg(not(mobile))]
#[tauri::command]
async fn get_session_history() -> Result<Vec<session::HistoryEntry>, String> {
    session::get_history()
}

#[cfg(not(mobile))]
#[tauri::command]
async fn deep_search_sessions(query: String) -> Result<Vec<session::DeepSearchHit>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }
    session::deep_search(&query)
}

#[cfg(not(mobile))]
#[tauri::command]
async fn get_cost_data() -> Result<session::CostData, String> {
    session::get_cost_data()
}

#[cfg(not(mobile))]
#[tauri::command]
async fn get_memory_files() -> Result<Vec<session::ProjectMemory>, String> {
    session::get_memory_files()
}

/// Save base64-encoded PNG data to a temp file and return the path.
/// Used by the token distance visualizer to share canvas screenshots.
#[cfg(not(mobile))]
#[tauri::command]
async fn save_temp_image(data: String) -> Result<String, String> {
    use std::fs;

    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("c9watch-token-journey.png");

    // data is base64-encoded PNG (no data URL prefix)
    let bytes = data
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(&data);

    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(bytes)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    fs::write(&file_path, decoded).map_err(|e| format!("Failed to write temp file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// Open a directory in the system file manager (Finder on macOS)
#[cfg(not(mobile))]
#[tauri::command]
async fn reveal_in_file_manager(path: String) -> Result<(), String> {
    std::process::Command::new("open")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open directory: {}", e))?;
    Ok(())
}

#[cfg(not(mobile))]
#[tauri::command]
async fn stop_session(app: AppHandle, pid: u32) -> Result<(), String> {
    stop_session_action(pid)?;
    std::thread::sleep(Duration::from_millis(300));

    if let Ok((sessions, _)) = detect_and_enrich_sessions() {
        let _ = app.emit("sessions-updated", &sessions);
    }
    Ok(())
}

#[cfg(not(mobile))]
#[tauri::command]
async fn open_session(pid: u32, project_path: String) -> Result<(), String> {
    open_session_action(pid, project_path)
}

#[cfg(not(mobile))]
#[tauri::command]
async fn rename_session(
    app: AppHandle,
    session_id: String,
    new_name: String,
) -> Result<(), String> {
    // Write to Claude Code's native JSONL format (primary)
    write_native_custom_title(&session_id, &new_name);

    // Also write to c9watch's own custom titles (fallback for history view)
    let mut custom_titles = session::CustomTitles::load();
    custom_titles.set(session_id, new_name);
    custom_titles.save()?;

    if let Ok((sessions, _)) = detect_and_enrich_sessions() {
        let _ = app.emit("sessions-updated", &sessions);
    }
    Ok(())
}

/// Append a `custom-title` entry to the session's JSONL file in Claude Code's native format.
/// This makes the rename visible to Claude Code itself (and persists across c9watch reinstalls).
pub fn write_native_custom_title(session_id: &str, title: &str) {
    use std::io::Write;

    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return,
    };
    let projects_dir = home.join(".claude").join("projects");

    // Search all project directories for the session JSONL
    let entries = match std::fs::read_dir(&projects_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let jsonl_path = path.join(format!("{}.jsonl", session_id));
        if jsonl_path.exists() {
            let entry = serde_json::json!({
                "type": "custom-title",
                "customTitle": title,
                "sessionId": session_id,
            });
            match std::fs::OpenOptions::new().append(true).open(&jsonl_path) {
                Ok(mut file) => {
                    if let Err(e) = writeln!(file, "{}", entry) {
                        debug_log::log_warn(&format!(
                            "Failed to write native custom-title for {}: {}",
                            session_id, e
                        ));
                    }
                }
                Err(e) => {
                    debug_log::log_warn(&format!(
                        "Failed to open JSONL for native custom-title {}: {}",
                        session_id, e
                    ));
                }
            }
            return;
        }
    }

    debug_log::log_warn(&format!(
        "Could not find JSONL file for session {} to write native custom-title",
        session_id
    ));
}

/// Get the terminal title for a session (iTerm2 only, macOS)
#[tauri::command]
async fn get_terminal_title(pid: u32) -> Result<Option<String>, String> {
    #[cfg(target_os = "macos")]
    {
        Ok(actions::get_iterm2_session_title(pid))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = pid;
        Ok(None)
    }
}

/// Show and focus the main application window
#[cfg(not(mobile))]
#[tauri::command]
async fn show_main_window(app: AppHandle) -> Result<(), String> {
    // On macOS the popover panel auto-hides via window_did_resign_key
    // when the main window takes focus. No need to explicitly hide it here.
    // (Calling panel.hide() here would deadlock the panel manager mutex
    // because resign_key fires synchronously and also calls get_webview_panel.)
    #[cfg(not(target_os = "macos"))]
    if let Some(popover) = app.get_webview_window("popover") {
        let _ = popover.hide();
    }

    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Server connection info for the mobile client
#[cfg(not(mobile))]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub token: String,
    pub port: u16,
    pub local_ip: String,
    pub ws_url: String,
}

#[cfg(not(mobile))]
#[tauri::command]
async fn get_server_info(info: tauri::State<'_, ServerInfo>) -> Result<ServerInfo, String> {
    Ok(ServerInfo {
        token: info.token.clone(),
        port: info.port,
        local_ip: info.local_ip.clone(),
        ws_url: info.ws_url.clone(),
    })
}

#[cfg(not(mobile))]
#[tauri::command]
async fn get_debug_logs() -> Result<Vec<debug_log::LogEntry>, String> {
    Ok(debug_log::get_logs())
}

// ── MQTT commands ──────────────────────────────────────────────────

#[cfg(not(mobile))]
#[tauri::command]
async fn get_mqtt_status(
    mqtt: tauri::State<'_, Arc<Mutex<Option<mqtt::client::MqttClient>>>>,
) -> Result<bool, String> {
    let guard = mqtt.lock().await;
    match guard.as_ref() {
        Some(client) => Ok(client.is_connected().await),
        None => Ok(false),
    }
}

#[cfg(not(mobile))]
#[tauri::command]
async fn get_claimed_missions(
    cache: tauri::State<'_, Arc<Mutex<mqtt::state_cache::MissionStateCache>>>,
) -> Result<Vec<String>, String> {
    let c = cache.lock().await;
    Ok(c.get_claimed_mission_ids())
}

#[cfg(not(mobile))]
#[tauri::command]
async fn get_registry(
    state: tauri::State<'_, Arc<Mutex<Option<mqtt::types::WorkspaceRegistryMessage>>>>,
) -> Result<Option<serde_json::Value>, String> {
    let guard = state.lock().await;
    match guard.as_ref() {
        Some(reg) => Ok(Some(serde_json::to_value(reg).map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

#[cfg(not(mobile))]
#[tauri::command]
async fn accept_phase_cmd(
    mqtt_state: tauri::State<'_, Arc<Mutex<Option<mqtt::client::MqttClient>>>>,
    mission_id: String,
    phase_id: String,
) -> Result<(), String> {
    let guard = mqtt_state.lock().await;
    let client = guard.as_ref().ok_or("MQTT not connected")?;
    let config = client.config();
    let topic = format!(
        "brain/{}/accept/{}/{}",
        config.workspace, mission_id, phase_id
    );
    let payload = serde_json::json!({
        "instance_id": config.instance_id,
        "mission_id": mission_id,
        "phase_id": phase_id,
        "published_at": chrono::Utc::now().to_rfc3339(),
    });
    // Fire-and-forget: UI never blocks; state confirmation arrives via DynamoDB Stream
    client.publish_fire_and_forget(topic, &payload)?;
    debug_log::log_info(&format!(
        "[MQTT] Accepted phase {}/{} (fire-and-forget)",
        mission_id, phase_id
    ));
    Ok(())
}

#[cfg(not(mobile))]
#[tauri::command]
async fn get_task_queue(
    handler: tauri::State<'_, Arc<mqtt::assignment::AssignmentHandler>>,
) -> Result<Vec<mqtt::assignment::TaskQueueEntry>, String> {
    Ok(handler.get_queue().await)
}

#[cfg(not(mobile))]
#[tauri::command]
async fn get_mission_state(
    cache: tauri::State<'_, Arc<Mutex<mqtt::state_cache::MissionStateCache>>>,
) -> Result<serde_json::Value, String> {
    let guard = cache.lock().await;
    Ok(guard.snapshot())
}

#[tauri::command]
async fn get_mission_phases(
    mission_id: String,
    cache: tauri::State<'_, Arc<Mutex<mqtt::state_cache::MissionStateCache>>>,
) -> Result<serde_json::Value, String> {
    let guard = cache.lock().await;

    let mut phase_map: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();

    // Plan is the authoritative source for the phase list.
    // State cache (from retained MQTT messages) only enriches status/completion.
    if let Some(plan) = guard.get_plan(&mission_id) {
        for (i, p) in plan.phases.iter().enumerate() {
            let phase_id = p.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if phase_id.is_empty() { continue; }

            let plan_name = p.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let plan_order = p.get("order").and_then(|v| v.as_u64()).unwrap_or(i as u64);

            // Get todos: plan JSON first, then state cache
            let plan_todos = p.get("todos").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let todo_list: Vec<serde_json::Value> = if !plan_todos.is_empty() {
                plan_todos.iter().map(|t| {
                    let tid = t.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    // Prefer cached todo (has latest status from retained messages)
                    if let Some(cached) = guard.get_todo(tid) {
                        serde_json::json!({
                            "todo_id": cached.todo_id,
                            "description": cached.description,
                            "status": cached.status,
                            "role": cached.role,
                        })
                    } else {
                        serde_json::json!({
                            "todo_id": tid,
                            "description": t.get("title").or(t.get("description")).and_then(|v| v.as_str()).unwrap_or(""),
                            "status": t.get("status").and_then(|v| v.as_str()).unwrap_or("pending"),
                            "role": t.get("role").and_then(|v| v.as_str()).unwrap_or(""),
                        })
                    }
                }).collect()
            } else {
                // Plan phase has no embedded todos -- look them up in state cache
                guard.get_todos_for_phase(&mission_id, &phase_id).iter().map(|t| {
                    serde_json::json!({
                        "todo_id": t.todo_id,
                        "description": t.description,
                        "status": t.status,
                        "role": t.role,
                    })
                }).collect()
            };

            // Enrich with state cache status (from retained phase messages)
            let (status, completed_count, todo_count) = if let Some(sp) = guard.get_phase(&mission_id, &phase_id) {
                let tc = if todo_list.is_empty() { sp.todo_count as usize } else { todo_list.len() };
                (sp.status.clone(), sp.completed_count, tc)
            } else {
                let status = p.get("status").and_then(|v| v.as_str()).unwrap_or("blocked").to_string();
                (status, 0, todo_list.len())
            };

            phase_map.insert(phase_id, serde_json::json!({
                "phase_id": p.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                "name": plan_name,
                "order": plan_order,
                "status": status,
                "todo_count": todo_count,
                "completed_count": completed_count,
                "todos": todo_list,
                "blocked_by": p.get("blocked_by").cloned().unwrap_or(serde_json::json!([])),
            }));
        }
    } else {
        // No plan available -- fall back to state cache phases only
        let state_phases = guard.get_phases_for_mission(&mission_id);
        for p in &state_phases {
            let todos = guard.get_todos_for_phase(&mission_id, &p.phase_id);
            let todo_list: Vec<serde_json::Value> = todos.iter().map(|t| {
                serde_json::json!({
                    "todo_id": t.todo_id,
                    "description": t.description,
                    "status": t.status,
                    "role": t.role,
                })
            }).collect();
            phase_map.insert(p.phase_id.clone(), serde_json::json!({
                "phase_id": p.phase_id,
                "name": p.name,
                "order": p.order,
                "status": p.status,
                "todo_count": if todo_list.is_empty() { p.todo_count as usize } else { todo_list.len() },
                "completed_count": p.completed_count,
                "todos": todo_list,
                "blocked_by": p.blocked_by,
            }));
        }
    }

    // Sort by phase_id numerically (P0=0, P1=1, P10=10)
    let mut result: Vec<serde_json::Value> = phase_map.into_values().collect();
    result.sort_by_key(|p| {
        let pid = p.get("phase_id").and_then(|v| v.as_str()).unwrap_or("P999");
        // Parse number from "P0", "P1", "P10" etc.
        pid.trim_start_matches('P')
            .parse::<u64>()
            .unwrap_or(999)
    });

    Ok(serde_json::to_value(&result).unwrap_or_default())
}

/// Kill a mission: purge all MQTT retained messages and clear the in-memory cache.
#[cfg(not(mobile))]
#[tauri::command]
async fn kill_mission(
    mission_id: String,
    cache: tauri::State<'_, Arc<Mutex<mqtt::state_cache::MissionStateCache>>>,
    mqtt_state: tauri::State<'_, Arc<Mutex<Option<mqtt::client::MqttClient>>>>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    // Resolve full mission ID and purge cache
    let (full_mid, purge) = {
        let mut c = cache.lock().await;
        let fmid = c.resolve_mission_id(&mission_id)
            .unwrap_or_else(|| mission_id.clone());
        let purge = c.purge_mission(&fmid);
        (fmid, purge)
    };

    // Clear retained messages on the MQTT broker
    let mqtt_guard = mqtt_state.lock().await;
    if let Some(client) = mqtt_guard.as_ref() {
        let ws = &client.config().workspace;

        // Clear plan
        let _ = client.clear_retained(&format!("weaver/{}/state/{}/plan", ws, full_mid)).await;

        // Clear each phase
        for pid in &purge.phase_ids {
            let _ = client.clear_retained(&format!("weaver/{}/state/{}/phase/{}", ws, full_mid, pid)).await;
        }

        // Clear each todo
        for tid in &purge.todo_ids {
            let _ = client.clear_retained(&format!("weaver/{}/state/{}/todo/{}", ws, full_mid, tid)).await;
        }

        // Fire-and-forget release notification to Brain
        let _ = client.publish_fire_and_forget(
            format!("brain/{}/release/{}", ws, full_mid),
            &serde_json::json!({
                "mission_id": full_mid,
                "instance_id": client.config().instance_id,
                "reason": "manual_kill",
                "published_at": chrono::Utc::now().to_rfc3339(),
            }),
        );
    }

    // Also notify Brain via HTTP API (reliable, wakes Lambda)
    let saved = settings::load_settings();
    let url = format!(
        "{}/v1/mission/{}/weaver-plan/control",
        saved.brain_api_url.trim_end_matches('/'),
        full_mid
    );
    let http_client = reqwest::Client::new();
    let _ = http_client
        .post(&url)
        .header("X-Workspace", &saved.workspace)
        .json(&serde_json::json!({
            "action": "kill",
            "reason": "Killed from Weaver UI",
            "reset_statuses": false,
        }))
        .send()
        .await;

    // Refresh frontend
    let _ = app.emit("mission-phases-updated", serde_json::json!({ "mission_id": full_mid }));
    let _ = app.emit("mqtt-registry", serde_json::json!({}));

    Ok(format!("Killed mission {}: cleared {} phases, {} todos",
        &full_mid[..8.min(full_mid.len())], purge.phase_ids.len(), purge.todo_ids.len()))
}

/// Pause a running mission. Sends control message to Brain.
#[cfg(not(mobile))]
#[tauri::command]
async fn pause_mission(
    mission_id: String,
    cache: tauri::State<'_, Arc<Mutex<mqtt::state_cache::MissionStateCache>>>,
    mqtt_state: tauri::State<'_, Arc<Mutex<Option<mqtt::client::MqttClient>>>>,
) -> Result<String, String> {
    let full_mid = {
        let c = cache.lock().await;
        c.resolve_mission_id(&mission_id)
            .unwrap_or_else(|| mission_id.clone())
    };

    // Fire-and-forget pause control
    let mqtt_guard = mqtt_state.lock().await;
    if let Some(client) = mqtt_guard.as_ref() {
        let ws = &client.config().workspace;
        let _ = client.publish_fire_and_forget(
            format!("weaver/{}/control/{}", ws, full_mid),
            &serde_json::json!({
                "action": "pause",
                "mission_id": full_mid,
                "published_at": chrono::Utc::now().to_rfc3339(),
            }),
        );
    }

    // Also notify via HTTP API
    let saved = settings::load_settings();
    let url = format!(
        "{}/v1/mission/{}/weaver-plan/control",
        saved.brain_api_url.trim_end_matches('/'),
        full_mid
    );
    let http_client = reqwest::Client::new();
    let _ = http_client
        .post(&url)
        .header("X-Workspace", &saved.workspace)
        .json(&serde_json::json!({ "action": "pause" }))
        .send()
        .await;

    debug_log::log_info(&format!("[API] Paused mission {}", &full_mid[..8.min(full_mid.len())]));
    Ok(format!("Paused mission {}", &full_mid[..8.min(full_mid.len())]))
}

/// Resume a paused mission. Sends control message to Brain.
#[cfg(not(mobile))]
#[tauri::command]
async fn resume_mission(
    mission_id: String,
    cache: tauri::State<'_, Arc<Mutex<mqtt::state_cache::MissionStateCache>>>,
    mqtt_state: tauri::State<'_, Arc<Mutex<Option<mqtt::client::MqttClient>>>>,
) -> Result<String, String> {
    let full_mid = {
        let c = cache.lock().await;
        c.resolve_mission_id(&mission_id)
            .unwrap_or_else(|| mission_id.clone())
    };

    // Fire-and-forget resume control
    let mqtt_guard = mqtt_state.lock().await;
    if let Some(client) = mqtt_guard.as_ref() {
        let ws = &client.config().workspace;
        let _ = client.publish_fire_and_forget(
            format!("weaver/{}/control/{}", ws, full_mid),
            &serde_json::json!({
                "action": "resume",
                "mission_id": full_mid,
                "published_at": chrono::Utc::now().to_rfc3339(),
            }),
        );
    }

    // Also notify via HTTP API
    let saved = settings::load_settings();
    let url = format!(
        "{}/v1/mission/{}/weaver-plan/control",
        saved.brain_api_url.trim_end_matches('/'),
        full_mid
    );
    let http_client = reqwest::Client::new();
    let _ = http_client
        .post(&url)
        .header("X-Workspace", &saved.workspace)
        .json(&serde_json::json!({ "action": "resume" }))
        .send()
        .await;

    debug_log::log_info(&format!("[API] Resumed mission {}", &full_mid[..8.min(full_mid.len())]));
    Ok(format!("Resumed mission {}", &full_mid[..8.min(full_mid.len())]))
}

/// Take/claim a mission for this instance. Publishes claim to Brain.
#[cfg(not(mobile))]
#[tauri::command]
async fn take_mission(
    mission_id: String,
    cache: tauri::State<'_, Arc<Mutex<mqtt::state_cache::MissionStateCache>>>,
    mqtt_state: tauri::State<'_, Arc<Mutex<Option<mqtt::client::MqttClient>>>>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let hostname = gethostname::gethostname().to_string_lossy().to_string();

    let full_mid = {
        let mut c = cache.lock().await;
        let fmid = c.resolve_mission_id(&mission_id)
            .unwrap_or_else(|| mission_id.clone());
        // Store claim locally
        let mqtt_guard = mqtt_state.lock().await;
        let iid = mqtt_guard.as_ref()
            .map(|c| c.config().instance_id.clone())
            .unwrap_or_default();
        c.claim_mission(&fmid, &iid);
        fmid
    };

    // Fire-and-forget claim to Brain: never blocks UI
    let mqtt_guard = mqtt_state.lock().await;
    if let Some(client) = mqtt_guard.as_ref() {
        let ws = &client.config().workspace;
        let iid = &client.config().instance_id;

        let _ = client.publish_fire_and_forget(
            format!("brain/{}/claim/{}", ws, full_mid),
            &serde_json::json!({
                "mission_id": full_mid,
                "instance_id": iid,
                "hostname": hostname,
                "workspace": ws,
                "claimed_at": chrono::Utc::now().to_rfc3339(),
            }),
        );
    }

    let _ = app.emit("mission-phases-updated", serde_json::json!({ "mission_id": full_mid }));

    Ok(format!("Claimed mission {} for {}", &full_mid[..8.min(full_mid.len())], hostname))
}

#[cfg(not(mobile))]
#[tauri::command]
async fn start_mission_execution(
    mission_id: String,
    cache: tauri::State<'_, Arc<Mutex<mqtt::state_cache::MissionStateCache>>>,
) -> Result<String, String> {
    let full_mid = {
        let c = cache.lock().await;
        c.resolve_mission_id(&mission_id)
            .unwrap_or_else(|| mission_id.clone())
    };

    // Call Brain HTTP API directly (not MQTT) to trigger execution.
    // This wakes Lambda and avoids the frozen-subscriber problem.
    let saved = settings::load_settings();
    let url = format!(
        "{}/v1/mission/{}/weaver-plan",
        saved.brain_api_url.trim_end_matches('/'),
        full_mid
    );

    let client = reqwest::Client::new();
    let resp = client
        .put(&url)
        .header("X-Workspace", &saved.workspace)
        .json(&serde_json::json!({ "status": "executing" }))
        .send()
        .await
        .map_err(|e| format!("Brain API call failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Brain API returned {}: {}", status, body));
    }

    debug_log::log_info(&format!(
        "[API] Execute mission {} via Brain API: {}",
        &full_mid[..8.min(full_mid.len())], url
    ));
    Ok(format!("Execution triggered for mission {}", &full_mid[..8.min(full_mid.len())]))
}

/// Manually start execution of a phase by spawning Claude Code in tmux.
/// Used for demos and testing when Brain hasn't published a PhaseAssignment.
#[cfg(not(mobile))]
#[tauri::command]
async fn start_phase_manually(
    mission_id: String,
    phase_id: String,
    cache: tauri::State<'_, Arc<Mutex<mqtt::state_cache::MissionStateCache>>>,
    spawner: tauri::State<'_, Arc<executor::spawner::ClaudeCodeSpawner>>,
) -> Result<String, String> {
    let mount = default_workspace_mount();
    let short_mid = if mission_id.len() > 8 { &mission_id[..8] } else { &mission_id };
    let weaver_cwd = mount.join(".worktrees").join(short_mid).join("weaver");

    if !weaver_cwd.exists() {
        return Err(format!("No weaver/ workspace at {}. Run autopilot first.", weaver_cwd.display()));
    }

    // Find plugin directory
    let plugin_dir = mount.parent().unwrap_or(&mount)
        .join("contexthub-weaver")
        .join("weaver-plugin");
    let plugin_dir = if plugin_dir.exists() {
        plugin_dir
    } else {
        let home = dirs::home_dir().unwrap_or_default();
        let alt = home.join("Sonic-Web-Dev").join("contexthub").join("contexthub-weaver").join("weaver-plugin");
        if alt.exists() { alt } else { return Err("Weaver plugin directory not found".to_string()); }
    };

    // Get phase and todo info from cache
    let phase_name = {
        let guard = cache.lock().await;
        guard.get_phase(&mission_id, &phase_id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| phase_id.clone())
    };

    crate::debug_log::log_info(&format!(
        "[ManualStart] Starting phase '{}' ({}) for mission {}",
        phase_name, phase_id, short_mid
    ));

    // Spawn Claude Code in tmux
    let session = spawner.spawn_session(&mission_id, &weaver_cwd, &plugin_dir).await?;

    // Wait for channel server to start
    let port = executor::spawner::ClaudeCodeSpawner::wait_for_channel_port(&weaver_cwd).await?;

    // Build assignment payload from cache
    let assignment_json = {
        let guard = cache.lock().await;
        let todos = guard.get_todos_for_phase(&mission_id, &phase_id);
        let todo_list: Vec<serde_json::Value> = todos.iter().map(|t| {
            serde_json::json!({
                "todo_id": t.todo_id,
                "description": t.description,
                "role": t.role,
            })
        }).collect();
        serde_json::json!({
            "type": "assignment",
            "mission_id": mission_id,
            "phase_id": phase_id,
            "phase_name": phase_name,
            "todos": todo_list,
            "content": format!("Execute phase: {}. Read CLAUDE.md for context, then work through each todo.", phase_name),
        })
    };

    // POST assignment to channel
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/", port);
    match client.post(&url)
        .json(&assignment_json)
        .send()
        .await
    {
        Ok(resp) => {
            crate::debug_log::log_info(&format!(
                "[ManualStart] Assignment posted to channel port {}, status: {}",
                port, resp.status()
            ));
        }
        Err(e) => {
            crate::debug_log::log_info(&format!(
                "[ManualStart] Failed to POST assignment: {}", e
            ));
        }
    }

    Ok(session.session_name)
}

/// Regenerate weaver/ context (CLAUDE.md + .claude/ + .weaver/) for a mission.
/// Called when a phase transitions or when the developer wants to refresh context.
#[cfg(not(mobile))]
#[tauri::command]
async fn regenerate_workspace_context(
    mission_id: String,
    cache: tauri::State<'_, Arc<Mutex<mqtt::state_cache::MissionStateCache>>>,
) -> Result<bool, String> {
    let mount = default_workspace_mount();
    let guard = cache.lock().await;

    let short_mid = if mission_id.len() > 8 {
        &mission_id[..8]
    } else {
        &mission_id
    };
    let worktrees_dir = mount.join(".worktrees").join(short_mid);
    let weaver_dir = worktrees_dir.join("weaver");

    if !weaver_dir.exists() {
        return Err(format!("No weaver/ directory for mission {}", mission_id));
    }

    // Collect sibling repo worktree names
    let repo_ids: Vec<String> = std::fs::read_dir(&worktrees_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            let name = e.file_name();
            p.is_dir()
                && name != "weaver"
                && (p.join(".git").exists() || p.join(".git").is_file())
        })
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect();

    match workspace::claude_md::write_workspace_context(
        &guard,
        &mission_id,
        &weaver_dir,
        &repo_ids,
        None,
    ) {
        Ok(written) => {
            if written {
                debug_log::log_info(&format!(
                    "[Context] Regenerated weaver/ for mission {}",
                    mission_id
                ));
            }
            Ok(written)
        }
        Err(e) => {
            debug_log::log_error(&format!("[Context] Regen error: {}", e));
            Err(e)
        }
    }
}

/// List active weaver tmux sessions.
#[cfg(not(mobile))]
#[tauri::command]
async fn list_weaver_sessions() -> Result<Vec<serde_json::Value>, String> {
    let output = tokio::process::Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}\t#{session_created}\t#{session_activity}"])
        .output()
        .await
        .map_err(|e| format!("tmux: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let sessions: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|l| l.starts_with("weaver-"))
        .map(|l| {
            let parts: Vec<&str> = l.split('\t').collect();
            serde_json::json!({
                "name": parts.first().unwrap_or(&""),
                "created": parts.get(1).unwrap_or(&""),
                "activity": parts.get(2).unwrap_or(&""),
            })
        })
        .collect();

    Ok(sessions)
}

/// Open a terminal attached to a weaver tmux session.
#[cfg(not(mobile))]
#[tauri::command]
async fn attach_weaver_session(session_name: String) -> Result<(), String> {
    // Open iTerm or Terminal.app with tmux attach
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            r#"tell application "Terminal"
                activate
                do script "tmux attach -t {}"
            end tell"#,
            session_name
        );
        tokio::process::Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        tokio::process::Command::new("tmux")
            .args(["attach", "-t", &session_name])
            .spawn()
            .map_err(|e| format!("Failed to attach: {}", e))?;
    }

    Ok(())
}

/// Find the active channel port for a mission workspace.
#[cfg(not(mobile))]
#[tauri::command]
async fn get_channel_port(mission_id: Option<String>) -> Result<Option<u16>, String> {
    let mount = default_workspace_mount();
    let worktrees = mount.join(".worktrees");

    // If mission_id given, check that specific workspace
    if let Some(mid) = &mission_id {
        let short = if mid.len() > 8 { &mid[..8] } else { mid.as_str() };
        let port_file = worktrees.join(short).join("weaver").join(".weaver").join("channel-port");
        if port_file.exists() {
            let port: u16 = std::fs::read_to_string(&port_file)
                .map_err(|e| e.to_string())?
                .trim()
                .parse()
                .map_err(|e: std::num::ParseIntError| e.to_string())?;
            return Ok(Some(port));
        }
        return Ok(None);
    }

    // Otherwise scan all worktrees for any active channel
    if let Ok(entries) = std::fs::read_dir(&worktrees) {
        for entry in entries.flatten() {
            let port_file = entry.path().join("weaver").join(".weaver").join("channel-port");
            if port_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&port_file) {
                    if let Ok(port) = content.trim().parse::<u16>() {
                        return Ok(Some(port));
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Send text to a tmux session as keystrokes.
#[cfg(not(mobile))]
#[tauri::command]
async fn send_to_weaver_session(session_name: String, text: String) -> Result<(), String> {
    let output = tokio::process::Command::new("tmux")
        .args(["send-keys", "-t", &session_name, &text, "Enter"])
        .output()
        .await
        .map_err(|e| format!("tmux send-keys failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Send failed: {}", stderr));
    }
    Ok(())
}

/// Read observations from claude-mem's SQLite database.
/// Gives Weavy access to the persistent memory across all Claude Code sessions.
#[cfg(not(mobile))]
#[tauri::command]
async fn get_claude_mem_observations(
    project: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<serde_json::Value>, String> {
    let home = dirs::home_dir().ok_or("No home dir")?;
    let db_path = home.join(".claude-mem").join("claude-mem.db");

    if !db_path.exists() {
        return Ok(vec![]);
    }

    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|e| format!("Failed to open claude-mem DB: {}", e))?;

    let limit = limit.unwrap_or(50);

    let (query, param): (String, Option<String>) = if let Some(proj) = project {
        (
            format!(
                "SELECT id, type, title, narrative, project, memory_session_id, created_at_epoch \
                 FROM observations WHERE project = ?1 \
                 ORDER BY created_at_epoch DESC LIMIT {}",
                limit
            ),
            Some(proj),
        )
    } else {
        (
            format!(
                "SELECT id, type, title, narrative, project, memory_session_id, created_at_epoch \
                 FROM observations \
                 ORDER BY created_at_epoch DESC LIMIT {}",
                limit
            ),
            None,
        )
    };

    let mut stmt = conn.prepare(&query).map_err(|e| format!("SQL error: {}", e))?;

    let map_row = |row: &rusqlite::Row| -> rusqlite::Result<serde_json::Value> {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "type": row.get::<_, String>(1).unwrap_or_default(),
            "title": row.get::<_, String>(2).unwrap_or_default(),
            "narrative": row.get::<_, String>(3).unwrap_or_default(),
            "project": row.get::<_, String>(4).unwrap_or_default(),
            "session_id": row.get::<_, String>(5).unwrap_or_default(),
            "timestamp": row.get::<_, i64>(6)?,
        }))
    };

    let results: Vec<serde_json::Value> = if let Some(p) = &param {
        stmt.query_map(rusqlite::params![p], map_row)
            .map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok())
            .collect()
    } else {
        stmt.query_map([], map_row)
            .map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok())
            .collect()
    };

    Ok(results)
}

/// Search claude-mem observations by keyword.
#[cfg(not(mobile))]
#[tauri::command]
async fn search_claude_mem(
    query: String,
    limit: Option<u32>,
) -> Result<Vec<serde_json::Value>, String> {
    let home = dirs::home_dir().ok_or("No home dir")?;
    let db_path = home.join(".claude-mem").join("claude-mem.db");

    if !db_path.exists() {
        return Ok(vec![]);
    }

    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|e| format!("Failed to open claude-mem DB: {}", e))?;

    let limit = limit.unwrap_or(20);
    let sql = format!(
        "SELECT o.id, o.type, o.title, o.narrative, o.project, o.created_at_epoch \
         FROM observations_fts f JOIN observations o ON o.id = f.rowid \
         WHERE observations_fts MATCH ?1 \
         ORDER BY o.created_at_epoch DESC LIMIT {}",
        limit
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("SQL error: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![query], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "type": row.get::<_, String>(1).unwrap_or_default(),
                "title": row.get::<_, String>(2).unwrap_or_default(),
                "narrative": row.get::<_, String>(3).unwrap_or_default(),
                "project": row.get::<_, String>(4).unwrap_or_default(),
                "timestamp": row.get::<_, i64>(5)?,
            }))
        })
        .map_err(|e| format!("Query error: {}", e))?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Read the current content of a weaver tmux session.
#[cfg(not(mobile))]
#[tauri::command]
async fn read_weaver_session(session_name: String, lines: Option<u32>) -> Result<String, String> {
    let scroll = lines.unwrap_or(50);
    let output = tokio::process::Command::new("tmux")
        .args([
            "capture-pane",
            "-t",
            &session_name,
            "-p",
            "-S",
            &format!("-{}", scroll),
        ])
        .output()
        .await
        .map_err(|e| format!("tmux capture-pane failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Session '{}' not found: {}", session_name, stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Load a WeaverPlan from a fixture file or raw JSON into the MissionStateCache.
/// Used for dev/testing without MQTT. Also works as a general "import plan" command.
#[cfg(not(mobile))]
#[tauri::command]
async fn load_fixture(
    cache: tauri::State<'_, Arc<Mutex<mqtt::state_cache::MissionStateCache>>>,
    path: Option<String>,
) -> Result<mqtt::state_cache::LoadResult, String> {
    let file_path = path.unwrap_or_else(|| {
        // Default to the bundled test fixture
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));
        if let Some(dir) = exe_dir {
            let fixture = dir.join("tests/fixtures/test-weaver-plan.json");
            if fixture.exists() {
                return fixture.to_string_lossy().to_string();
            }
        }
        // Fallback to source tree path (dev mode)
        "tests/fixtures/test-weaver-plan.json".to_string()
    });

    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read fixture file {}: {}", file_path, e))?;

    let plan_json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse fixture JSON: {}", e))?;

    let mut guard = cache.lock().await;
    let result = guard.load_weaver_plan(&plan_json)?;

    debug_log::log_info(&format!(
        "[Fixture] Loaded plan '{}': {} phases, {} todos",
        result.title, result.phases, result.todos
    ));

    Ok(result)
}

#[cfg(not(mobile))]
#[tauri::command]
async fn connect_mqtt(
    app: AppHandle,
    mqtt_state: tauri::State<'_, Arc<Mutex<Option<mqtt::client::MqttClient>>>>,
    assignment_handler: tauri::State<'_, Arc<mqtt::assignment::AssignmentHandler>>,
    control_handler: tauri::State<'_, Arc<mqtt::control::ControlHandler>>,
    spawner: tauri::State<'_, Arc<executor::spawner::ClaudeCodeSpawner>>,
    state_cache: tauri::State<'_, Arc<Mutex<mqtt::state_cache::MissionStateCache>>>,
    supervisor_agent: tauri::State<'_, Arc<supervisor::SupervisorAgent>>,
    host: String,
    port: u16,
    username: String,
    password: String,
    instance_id: String,
    workspace: String,
) -> Result<(), String> {
    let config = mqtt::types::MqttConfig {
        host,
        port,
        username,
        password,
        instance_id: instance_id.clone(),
        workspace: workspace.clone(),
    };

    let client = mqtt::client::MqttClient::new(config).await;

    // Store client in managed state first
    {
        let mut state = mqtt_state.lock().await;
        *state = Some(client);
    }

    // Start heartbeat using managed state
    mqtt::heartbeat::start_heartbeat(
        mqtt_state.inner().clone(),
        state_cache.inner().clone(),
        instance_id.clone(),
        workspace,
        2,
        std::time::Instant::now(),
    );

    // Get broadcast receivers (4th for supervisor)
    let (rx1, rx2, rx3, rx4) = {
        let g = mqtt_state.lock().await;
        let c = g.as_ref().unwrap();
        (
            c.subscribe_incoming(),
            c.subscribe_incoming(),
            c.subscribe_incoming(),
            c.subscribe_incoming(),
        )
    };

    // Start handlers
    assignment_handler.start(rx1, app.clone());
    control_handler.start(rx2, app.clone());
    assignment_handler.start_auto_execute(
        rx3,
        mqtt_state.inner().clone(),
        spawner.inner().clone(),
        control_handler.inner().clone(),
        state_cache.inner().clone(),
        instance_id,
        default_workspace_mount(),
        app,
    );

    // Attach MQTT feed to supervisor (already running from setup)
    supervisor_agent.attach_mqtt(rx4);

    debug_log::log_info("[MQTT] Connected and all handlers started");
    Ok(())
}

// ── Settings commands ───────────────────────────────────────────────

#[cfg(not(mobile))]
#[tauri::command]
async fn get_settings() -> Result<settings::WeaverSettings, String> {
    Ok(settings::load_settings())
}

#[cfg(not(mobile))]
#[tauri::command]
async fn save_settings_cmd(s: settings::WeaverSettings) -> Result<(), String> {
    settings::save_settings(&s)
}

// ── Supervisor commands ────────────────────────────────────────────

#[cfg(not(mobile))]
#[tauri::command]
async fn get_supervisor_observations(
    agent: tauri::State<'_, Arc<supervisor::SupervisorAgent>>,
) -> Result<Vec<supervisor::agent::ObservationRecord>, String> {
    Ok(agent.get_observations().await)
}

#[cfg(not(mobile))]
#[tauri::command]
async fn get_supervisor_interventions(
    agent: tauri::State<'_, Arc<supervisor::SupervisorAgent>>,
) -> Result<Vec<supervisor::agent::InterventionRecord>, String> {
    Ok(agent.get_interventions().await)
}

#[cfg(not(mobile))]
#[tauri::command]
async fn get_supervisor_rules(
    agent: tauri::State<'_, Arc<supervisor::SupervisorAgent>>,
) -> Result<supervisor::SupervisorRules, String> {
    Ok(agent.get_rules().await)
}

#[cfg(not(mobile))]
#[tauri::command]
async fn update_supervisor_rules(
    agent: tauri::State<'_, Arc<supervisor::SupervisorAgent>>,
    rules: supervisor::SupervisorRules,
) -> Result<(), String> {
    agent.update_rules(rules).await;
    Ok(())
}

// ── Weavy conductor commands ───────────────────────────────────────

#[cfg(not(mobile))]
#[tauri::command]
async fn get_conductor_decisions(
    agent: tauri::State<'_, Arc<conductor::ConductorAgent>>,
) -> Result<Vec<conductor::DecisionRecord>, String> {
    Ok(agent.get_decisions().await)
}

#[cfg(not(mobile))]
#[tauri::command]
async fn get_conductor_config(
    agent: tauri::State<'_, Arc<conductor::ConductorAgent>>,
) -> Result<conductor::ConductorConfig, String> {
    Ok(agent.get_config().await)
}

#[cfg(not(mobile))]
#[tauri::command]
async fn update_conductor_config(
    agent: tauri::State<'_, Arc<conductor::ConductorAgent>>,
    config: conductor::ConductorConfig,
) -> Result<(), String> {
    agent.update_config(config).await;
    Ok(())
}

/// Weavy chat: free-form AI conversation powered by Bedrock.
/// Returns a natural language response informed by mission state.
#[cfg(not(mobile))]
#[tauri::command]
async fn weavy_chat(
    message: String,
    agent: tauri::State<'_, Arc<conductor::ConductorAgent>>,
    cache: tauri::State<'_, Arc<Mutex<mqtt::state_cache::MissionStateCache>>>,
) -> Result<String, String> {
    let config = agent.get_config().await;

    if !config.enabled {
        return Err("not_enabled".to_string());
    }

    // Use the conductor engine's chat() method for free-form text
    let engine = conductor::engine::ConductorEngine::new(config);
    let cache_guard = cache.lock().await;

    match engine.chat(&message, &*cache_guard).await {
        Ok(reply) => Ok(reply),
        Err(e) => Err(format!("bedrock_error: {}", e)),
    }
}

// ── Workspace commands ─────────────────────────────────────────────

#[cfg(not(mobile))]
#[tauri::command]
async fn get_workspace_status() -> Result<workspace::scanner::WorkspaceStatus, String> {
    let mount = default_workspace_mount();
    Ok(workspace::scanner::scan_workspace(&mount))
}

#[cfg(not(mobile))]
#[tauri::command]
async fn create_worktree_cmd(
    repo_path: String,
    mission_id: String,
    phase_id: String,
) -> Result<String, String> {
    let repo = std::path::PathBuf::from(&repo_path);
    let branch = workspace::git::mission_branch_name(&mission_id, &phase_id);
    let worktree_path = repo
        .parent()
        .unwrap_or(&repo)
        .join(".worktrees")
        .join(&mission_id[..8.min(mission_id.len())])
        .join(&phase_id);
    let path = workspace::git::create_worktree(&repo, &worktree_path, &branch)?;
    Ok(path.to_string_lossy().to_string())
}

#[cfg(not(mobile))]
#[tauri::command]
async fn setup_mission_cmd(
    mission_id: String,
    repos: Vec<String>,
    cache: tauri::State<'_, Arc<Mutex<mqtt::state_cache::MissionStateCache>>>,
) -> Result<workspace::git::MissionWorkspaceResult, String> {
    let mount = default_workspace_mount();
    let guard = cache.lock().await;
    Ok(workspace::git::setup_mission_worktrees(
        &mount,
        &mission_id,
        &repos,
        Some(&guard),
    ))
}

#[cfg(not(mobile))]
#[tauri::command]
async fn open_workspace_cmd(path: String) -> Result<(), String> {
    workspace::git::open_vscode_workspace(&std::path::PathBuf::from(&path))
}

#[cfg(not(mobile))]
#[tauri::command]
async fn clone_repo_cmd(url: String, branch: Option<String>) -> Result<String, String> {
    let mount = default_workspace_mount();
    let repo_name = url
        .rsplit('/')
        .next()
        .unwrap_or("repo")
        .trim_end_matches(".git");
    let target = mount.join(repo_name);
    let path = workspace::git::clone_repo(&url, &target, branch.as_deref())?;
    Ok(path.to_string_lossy().to_string())
}

#[cfg(not(mobile))]
fn default_workspace_mount() -> std::path::PathBuf {
    let saved = settings::load_settings();
    if !saved.workspace_mount.is_empty() {
        std::path::PathBuf::from(&saved.workspace_mount)
    } else {
        dirs::home_dir()
            .unwrap_or_default()
            .join("Workspace")
    }
}

// ── NSPanel definition for macOS popover ────────────────────────────
#[cfg(target_os = "macos")]
tauri_panel! {
    panel!(PopoverPanel {
        config: {
            can_become_key_window: true,
            is_floating_panel: true
        }
    })

    panel_event!(PopoverEventHandler {
        window_did_resign_key(notification: &NSNotification) -> ()
    })
}

// ── App entry point ─────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());

    // MCP Bridge: expose Weaver to AI agents via hypothesi/mcp-server-tauri (debug only)
    #[cfg(debug_assertions)]
    let builder = builder.plugin(tauri_plugin_mcp_bridge::init());

    // Desktop: full setup with all plugins and commands
    #[cfg(not(mobile))]
    let builder = builder
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_sharekit::init());

    // macOS: NSPanel plugin for popover (must appear above fullscreen apps)
    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_nspanel::init());

    #[cfg(not(mobile))]
    let builder = builder
        .setup(|app| {
            // ── WebSocket server ────────────────────────────────
            let token = auth::generate_token();
            let local_ip = auth::get_local_ip();
            let port = web_server::WS_PORT;

            let ws_url = format!("ws://{}:{}/ws?token={}", local_ip, port, token);
            let http_url = format!("http://{}:{}/?token={}", local_ip, port, token);

            debug_log::log_info(&format!("Mobile connection ready — URL: {}", http_url));
            qr2term::print_qr(&http_url).ok();
            eprintln!();

            let (sessions_tx, _rx) = tokio::sync::broadcast::channel::<String>(16);
            let (notifications_tx, _nrx) = tokio::sync::broadcast::channel::<String>(16);

            let server_info = ServerInfo {
                token: token.clone(),
                port,
                local_ip: local_ip.clone(),
                ws_url,
            };
            app.manage(server_info);

            // Create conductor event channel early so WsState can hold a sender
            let (conductor_tx_for_ws, conductor_rx_for_agent) =
                tokio::sync::mpsc::channel::<conductor::ConductorEvent>(256);

            let ws_state = Arc::new(web_server::WsState {
                auth_token: token,
                sessions_tx: sessions_tx.clone(),
                notifications_tx: notifications_tx.clone(),
                app_handle: Some(app.handle().clone()),
                conductor_tx: Some(conductor_tx_for_ws.clone()),
            });
            tauri::async_runtime::spawn(web_server::start_server(ws_state));

            // ── Polling loop ────────────────────────────────────
            start_polling(app.handle().clone(), sessions_tx, notifications_tx);

            // ── MQTT client (starts disconnected, connected via settings or env) ──
            let mqtt_state: Arc<Mutex<Option<mqtt::client::MqttClient>>> =
                Arc::new(Mutex::new(None));
            app.manage(mqtt_state.clone());

            // ── Assignment handler + control handler + spawner + state cache ──
            let assignment_handler = Arc::new(mqtt::assignment::AssignmentHandler::new());
            app.manage(assignment_handler.clone());
            let control_handler = Arc::new(mqtt::control::ControlHandler::new());
            app.manage(control_handler.clone());
            let spawner = Arc::new(executor::spawner::ClaudeCodeSpawner::new());
            app.manage(spawner.clone());
            let registry_state: Arc<Mutex<Option<mqtt::types::WorkspaceRegistryMessage>>> =
                Arc::new(Mutex::new(None));
            app.manage(registry_state);
            let state_cache: Arc<Mutex<mqtt::state_cache::MissionStateCache>> =
                Arc::new(Mutex::new(mqtt::state_cache::MissionStateCache::new()));
            app.manage(state_cache.clone());

            // ── Supervisor agent ──
            let supervisor_agent = Arc::new(supervisor::SupervisorAgent::new(
                supervisor::SupervisorRules::default(),
            ));
            app.manage(supervisor_agent.clone());
            // Hook event channel for forwarding web_server hook events to supervisor
            let (hook_tx, hook_rx) = tokio::sync::mpsc::channel::<supervisor::agent::HookEvent>(128);
            app.manage(hook_tx);
            // Start supervisor immediately (it listens for hook events and MQTT via attach_mqtt)
            supervisor_agent.start(app.handle().clone(), hook_rx);

            // ── Weavy conductor agent ──
            let conductor_config = conductor::ConductorConfig {
                enabled: std::env::var("CONDUCTOR_ENABLED")
                    .unwrap_or("1".to_string())
                    != "0",
                ..conductor::ConductorConfig::default()
            };
            let conductor_agent = Arc::new(conductor::ConductorAgent::new(conductor_config));
            app.manage(conductor_agent.clone());
            app.manage(conductor_tx_for_ws);
            // Start conductor with the receiver created alongside WsState
            conductor_agent.start(
                conductor_rx_for_agent,
                mqtt_state.clone(),
                spawner.clone(),
                state_cache.clone(),
                app.handle().clone(),
            );
            debug_log::log_info(&format!(
                "[Weavy] Conductor initialized (enabled={})",
                std::env::var("CONDUCTOR_ENABLED").unwrap_or("1".to_string()) != "0"
            ));

            // ── Auto-connect MQTT from env vars or saved settings ──
            {
                let mqtt_s = mqtt_state.clone();
                let ah = assignment_handler.clone();
                let ch = control_handler.clone();
                let sp = spawner.clone();
                let sc = state_cache.clone();
                let app_h = app.handle().clone();
                let sv = supervisor_agent.clone();

                tauri::async_runtime::spawn(async move {
                    // Try env vars first, then saved settings
                    let saved = settings::load_settings();
                    let host = std::env::var("MQTT_HOST").unwrap_or(saved.mqtt_host);
                    let port = std::env::var("MQTT_PORT")
                        .ok()
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(saved.mqtt_port);
                    let username = std::env::var("MQTT_USERNAME").unwrap_or(saved.mqtt_username);
                    let password = std::env::var("MQTT_PASSWORD").unwrap_or(saved.mqtt_password);
                    let instance_id = std::env::var("INSTANCE_ID").unwrap_or(saved.instance_id);
                    let workspace = std::env::var("WORKSPACE").unwrap_or(saved.workspace);

                    // Only auto-connect if we have non-default credentials
                    if username.is_empty() || password.is_empty() {
                        debug_log::log_info("[MQTT] No credentials found, skipping auto-connect");
                        return;
                    }

                    eprintln!("[MQTT] Auto-connecting as {} to {}:{}", username, host, port);
                    // Debug: write to file for troubleshooting
                    let _ = std::fs::write("/tmp/weaver-mqtt-debug.log", format!(
                        "Auto-connect: host={} port={} user={} pass_len={} instance={} ws={}\n",
                        host, port, username, password.len(), instance_id, workspace
                    ));
                    debug_log::log_info(&format!(
                        "[MQTT] Auto-connecting as {} to {}:{}",
                        username, host, port
                    ));

                    let config = mqtt::types::MqttConfig {
                        host,
                        port,
                        username,
                        password,
                        instance_id: instance_id.clone(),
                        workspace: workspace.clone(),
                    };

                    let client = mqtt::client::MqttClient::new(config).await;

                    // Store client in managed state first
                    {
                        let mut guard = mqtt_s.lock().await;
                        *guard = Some(client);
                    }

                    // Start heartbeat (uses managed state)
                    mqtt::heartbeat::start_heartbeat(
                        mqtt_s.clone(),
                        sc.clone(),
                        instance_id.clone(),
                        workspace.clone(),
                        2,
                        std::time::Instant::now(),
                    );

                    // Get broadcast receivers from the client (4th for supervisor)
                    let (rx1, rx2, rx3, rx4) = {
                        let g = mqtt_s.lock().await;
                        let c = g.as_ref().unwrap();
                        (
                            c.subscribe_incoming(),
                            c.subscribe_incoming(),
                            c.subscribe_incoming(),
                            c.subscribe_incoming(),
                        )
                    };

                    // Start handlers
                    ah.start(rx1, app_h.clone());
                    ch.start(rx2, app_h.clone());

                    ah.start_auto_execute(
                        rx3,
                        mqtt_s.clone(),
                        sp,
                        ch,
                        sc,
                        instance_id,
                        default_workspace_mount(),
                        app_h,
                    );

                    // Attach MQTT feed to supervisor (already running)
                    sv.attach_mqtt(rx4);

                    eprintln!("[MQTT] Auto-connect complete, all handlers started");
                    debug_log::log_info("[MQTT] Auto-connect complete, all handlers started");
                });
            }

            // ── Main window: hide on close instead of destroying ──────────────
            // This allows "Open Dashboard" from the popover to re-show it.
            // Without this, closing the window destroys it and show() is a no-op.
            #[cfg(not(mobile))]
            if let Some(main_win) = app.get_webview_window("main") {
                let main_win_clone = main_win.clone();
                main_win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = main_win_clone.hide();
                    }
                });
            }

            // ── Popover panel: convert NSWindow to NSPanel for fullscreen support ──
            // NSPanel can appear above fullscreen apps, unlike regular NSWindow.
            #[cfg(target_os = "macos")]
            if let Some(popover) = app.get_webview_window("popover") {
                match popover.to_panel::<PopoverPanel>() {
                    Err(e) => {
                        debug_log::log_warn(&format!("Failed to convert popover to NSPanel: {e}. Fullscreen support unavailable."));
                        // Do not return early — tray icon setup must still proceed below.
                    }
                    Ok(panel) => {
                        // Status level (25) = same as macOS menu bar
                        panel.set_level(PanelLevel::Status.value());

                        // NonactivatingPanel: won't steal focus from the fullscreen app
                        panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());

                        // Allow in all Spaces including fullscreen
                        panel.set_collection_behavior(
                            CollectionBehavior::new()
                                .full_screen_auxiliary()
                                .can_join_all_spaces()
                                .stationary()
                                .into(),
                        );

                        // Don't hide when app is deactivated (when fullscreen app is active)
                        panel.set_hides_on_deactivate(false);

                        // Rounded corners at the native window level
                        panel.set_corner_radius(10.0);

                        // Click-outside dismiss: hide panel when it loses key window status
                        let handler = PopoverEventHandler::new();
                        let handle = app.handle().clone();
                        handler.window_did_resign_key(move |_notification| {
                            if let Ok(p) = handle.get_webview_panel("popover") {
                                p.hide();
                            }
                        });
                        panel.set_event_handler(Some(handler.as_ref()));
                    }
                }
            }

            // ── Tray icon ───────────────────────────────────────
            let app_handle = app.handle().clone();
            TrayIconBuilder::new()
                .icon(tauri::include_image!("icons/tray-icon.png"))
                .icon_as_template(true)
                .tooltip("c9watch")
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        rect,
                        ..
                    } = event
                    {
                        // Use NSPanel via tauri-nspanel on macOS for fullscreen support
                        #[cfg(target_os = "macos")]
                        {
                            if let Ok(panel) = app_handle.get_webview_panel("popover") {
                                if panel.is_visible() {
                                    panel.hide();
                                } else {
                                    // Position below the tray icon, centered horizontally
                                    if let Some(popover) = app_handle.get_webview_window("popover")
                                    {
                                        let scale = popover
                                            .current_monitor()
                                            .ok()
                                            .flatten()
                                            .map(|m| m.scale_factor())
                                            .unwrap_or(1.0);

                                        let pos = rect.position.to_physical::<f64>(scale);
                                        let size = rect.size.to_physical::<f64>(scale);

                                        // Align panel left edge with tray icon left edge
                                        let x = pos.x;
                                        let y = pos.y + size.height + 4.0;

                                        let _ = popover.set_position(PhysicalPosition::new(
                                            x.round() as i32,
                                            y.round() as i32,
                                        ));
                                    }
                                    panel.show_and_make_key();
                                }
                            }
                        }

                        // Non-macOS: use regular window
                        #[cfg(not(target_os = "macos"))]
                        {
                            if let Some(popover) = app_handle.get_webview_window("popover") {
                                if popover.is_visible().unwrap_or(false) {
                                    let _ = popover.hide();
                                } else {
                                    let scale = popover
                                        .current_monitor()
                                        .ok()
                                        .flatten()
                                        .map(|m| m.scale_factor())
                                        .unwrap_or(1.0);
                                    let pos = rect.position.to_physical::<f64>(scale);
                                    let size = rect.size.to_physical::<f64>(scale);
                                    let popover_physical_width = popover
                                        .outer_size()
                                        .map(|s| s.width as f64)
                                        .unwrap_or(320.0);

                                    let x =
                                        pos.x + (size.width / 2.0) - (popover_physical_width / 2.0);
                                    let y = pos.y + size.height + 4.0;

                                    let _ = popover.set_position(PhysicalPosition::new(
                                        x.round() as i32,
                                        y.round() as i32,
                                    ));
                                    let _ = popover.show();
                                    let _ = popover.set_focus();
                                }
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_sessions,
            get_conversation,
            get_session_history,
            deep_search_sessions,
            get_cost_data,
            get_memory_files,
            save_temp_image,
            reveal_in_file_manager,
            stop_session,
            open_session,
            rename_session,
            get_terminal_title,
            show_main_window,
            get_server_info,
            get_debug_logs,
            get_mqtt_status,
            get_registry,
            get_claimed_missions,
            accept_phase_cmd,
            get_task_queue,
            get_mission_state,
            get_mission_phases,
            kill_mission,
            pause_mission,
            resume_mission,
            take_mission,
            start_mission_execution,
            start_phase_manually,
            load_fixture,
            regenerate_workspace_context,
            list_weaver_sessions,
            attach_weaver_session,
            read_weaver_session,
            send_to_weaver_session,
            get_channel_port,
            get_claude_mem_observations,
            search_claude_mem,
            get_workspace_status,
            clone_repo_cmd,
            create_worktree_cmd,
            setup_mission_cmd,
            open_workspace_cmd,
            connect_mqtt,
            get_settings,
            save_settings_cmd,
            get_supervisor_observations,
            get_supervisor_interventions,
            get_supervisor_rules,
            update_supervisor_rules,
            get_conductor_decisions,
            get_conductor_config,
            update_conductor_config,
            weavy_chat
        ]);

    // Mobile: minimal shell (all communication via WebSocket from the frontend)
    #[cfg(mobile)]
    let builder = builder.setup(|_app| Ok(()));

    builder
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            // Prevent the app from exiting when all windows are closed.
            // This is essential for tray/menu bar apps — the app stays alive
            // in the background with the tray icon even when no windows are visible.
            // Guard for desktop only: on mobile the OS controls the app lifecycle.
            #[cfg(not(mobile))]
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
