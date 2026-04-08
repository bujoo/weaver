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
    client.publish_json(&topic, &payload).await?;
    debug_log::log_info(&format!(
        "[MQTT] Accepted phase {}/{} -> {}",
        mission_id, phase_id, topic
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

    // Build phase map from individual PhaseState messages (have latest status)
    let state_phases = guard.get_phases_for_mission(&mission_id);
    let mut phase_map: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();

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
        }));
    }

    // Merge with plan JSON (has ALL phases including those not yet published individually)
    if let Some(plan) = guard.get_plan(&mission_id) {
        for (i, p) in plan.phases.iter().enumerate() {
            let phase_id = p.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if phase_id.is_empty() { continue; }

            // Skip if we already have this phase from individual state messages
            if phase_map.contains_key(&phase_id) { continue; }

            // Get todos: first try plan JSON, then fall back to state cache
            let plan_todos = p.get("todos").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let todo_list: Vec<serde_json::Value> = if !plan_todos.is_empty() {
                plan_todos.iter().map(|t| {
                    let tid = t.get("id").and_then(|v| v.as_str()).unwrap_or("");
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
                // Plan phases don't embed todos -- look them up in state cache
                guard.get_todos_for_phase(&mission_id, &phase_id).iter().map(|t| {
                    serde_json::json!({
                        "todo_id": t.todo_id,
                        "description": t.description,
                        "status": t.status,
                        "role": t.role,
                    })
                }).collect()
            };

            phase_map.insert(phase_id, serde_json::json!({
                "phase_id": p.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                "name": p.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                "order": p.get("order").and_then(|v| v.as_u64()).unwrap_or(i as u64),
                "status": p.get("status").and_then(|v| v.as_str()).unwrap_or("blocked"),
                "todo_count": todo_list.len(),
                "completed_count": 0,
                "todos": todo_list,
            }));
        }
    }

    // Sort by order and return
    let mut result: Vec<serde_json::Value> = phase_map.into_values().collect();
    result.sort_by_key(|p| p.get("order").and_then(|v| v.as_u64()).unwrap_or(999));

    Ok(serde_json::to_value(&result).unwrap_or_default())
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

/// Weavy chat: send a user message through the conductor for an AI response.
/// When the conductor is enabled, this calls Bedrock to generate a contextual reply.
/// When disabled, returns a simple status-based response.
#[cfg(not(mobile))]
#[tauri::command]
async fn weavy_chat(
    message: String,
    agent: tauri::State<'_, Arc<conductor::ConductorAgent>>,
    cache: tauri::State<'_, Arc<Mutex<mqtt::state_cache::MissionStateCache>>>,
) -> Result<String, String> {
    let config = agent.get_config().await;

    if !config.enabled {
        return Ok("Weavy conductor is not enabled. Set CONDUCTOR_ENABLED=1 and ensure AWS SSO is logged in.".to_string());
    }

    // Build a context-aware prompt from the user message + mission state
    let cache_guard = cache.lock().await;
    let snapshot = cache_guard.snapshot();
    let phases_info: Vec<String> = snapshot
        .get("plan_ids")
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|id| id.as_str())
        .flat_map(|mid| {
            cache_guard
                .get_phases_for_mission(mid)
                .iter()
                .map(|p| format!("{} ({}): {}", p.name, p.phase_id, p.status))
                .collect::<Vec<_>>()
        })
        .collect();

    let system = format!(
        "You are Weavy, a friendly AI dev sidekick managing Claude Code sessions for ContextHub.\n\
         Answer concisely. You have access to mission state.\n\n\
         Current state: {} plans, {} phases, {} todos cached.\n\
         Phases: {}",
        snapshot.get("plans").and_then(|v| v.as_u64()).unwrap_or(0),
        snapshot.get("phases").and_then(|v| v.as_u64()).unwrap_or(0),
        snapshot.get("todos").and_then(|v| v.as_u64()).unwrap_or(0),
        if phases_info.is_empty() {
            "none loaded".to_string()
        } else {
            phases_info.join(", ")
        }
    );
    drop(cache_guard);

    // Call Bedrock via the conductor engine
    let engine = conductor::engine::ConductorEngine::new(config);
    let events = vec![conductor::ConductorEvent::ChannelReply {
        mission_id: "chat".to_string(),
        reply_type: "user_message".to_string(),
        message: message.clone(),
    }];

    // Use Haiku for fast chat responses
    match engine
        .decide(&events, &*cache.lock().await, conductor::types::ModelTier::Haiku)
        .await
    {
        Ok((decision, _input_tok, _output_tok)) => {
            // Extract the reason field as the chat response
            let response = match &decision {
                conductor::ConductorDecision::NoAction { reason } => reason.clone(),
                conductor::ConductorDecision::PushNextPhase { reason, .. } => {
                    format!("I'll push the next phase. {}", reason)
                }
                conductor::ConductorDecision::Escalate { reason, .. } => {
                    format!("I need to escalate: {}", reason)
                }
                conductor::ConductorDecision::InjectContext { message, .. } => {
                    format!("Injecting context: {}", message)
                }
                other => format!("{:?}", other),
            };
            Ok(response)
        }
        Err(e) => Ok(format!("Bedrock error: {}. Is AWS SSO logged in? (aws sso login --profile wds_dev)", e)),
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
                    .unwrap_or_default()
                    == "1",
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
                std::env::var("CONDUCTOR_ENABLED").unwrap_or_default() == "1"
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
            accept_phase_cmd,
            get_task_queue,
            get_mission_state,
            get_mission_phases,
            start_phase_manually,
            load_fixture,
            regenerate_workspace_context,
            list_weaver_sessions,
            attach_weaver_session,
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
