use crate::executor::monitor::monitor_tmux_session;
use crate::executor::spawner::{build_system_prompt, ClaudeCodeSpawner};
use crate::mqtt::client::MqttClient;
use crate::mqtt::state_cache::MissionStateCache;
use crate::mqtt::types::PhaseAssignment;
use crate::workspace::git;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

/// Execute a full phase assignment: create worktree, run todos sequentially in tmux sessions.
pub async fn execute_assignment(
    assignment: PhaseAssignment,
    workspace_mount: &Path,
    spawner: Arc<ClaudeCodeSpawner>,
    mqtt: Arc<Mutex<Option<MqttClient>>>,
    state_cache: Arc<Mutex<MissionStateCache>>,
    instance_id: String,
    abort_rx: watch::Receiver<bool>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let repo_path = find_repo_for_assignment(&assignment, workspace_mount)?;
    let branch_name = git::mission_branch_name(&assignment.mission_id, &assignment.phase_id);

    // Create worktree for phase isolation
    let worktree_path = workspace_mount
        .join(".worktrees")
        .join(&assignment.mission_id)
        .join(&assignment.phase_id);

    let _cwd = git::create_worktree(&repo_path, &worktree_path, &branch_name)?;

    // Use weaver/ as Claude Code CWD (picks up CLAUDE.md + .claude/ config)
    let mission_short = if assignment.mission_id.len() > 8 {
        &assignment.mission_id[..8]
    } else {
        &assignment.mission_id
    };
    let weaver_cwd = workspace_mount
        .join(".worktrees")
        .join(mission_short)
        .join("weaver");
    let effective_cwd = if weaver_cwd.exists() {
        weaver_cwd
    } else {
        worktree_path.clone()
    };

    crate::debug_log::log_info(&format!(
        "[Pipeline] Executing phase {} in CWD {}",
        assignment.phase_name,
        effective_cwd.display()
    ));

    // Extract phase config
    let model = assignment
        .config
        .get("model")
        .and_then(|v| v.as_str())
        .map(String::from);
    let allowed_tools: Vec<String> = assignment
        .config
        .get("allowedTools")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Execute todos sequentially
    for todo_id in &assignment.todos {
        // Check abort signal
        if *abort_rx.borrow() {
            crate::debug_log::log_info(&format!(
                "[Pipeline] Aborted before todo {}",
                todo_id
            ));
            break;
        }

        crate::debug_log::log_info(&format!(
            "[Pipeline] Executing todo {} in phase {}",
            todo_id, assignment.phase_name
        ));

        // Look up todo spec + phase config from retained MQTT state cache
        let (description, spec, file_paths, phase_system_prompt) = {
            let cache = state_cache.lock().await;
            let (desc, sp, fps) = match cache.get_todo(todo_id) {
                Some(todo_state) => {
                    let desc = if todo_state.description.is_empty() {
                        format!("Execute todo {}", todo_id)
                    } else {
                        todo_state.description.clone()
                    };
                    (desc, todo_state.spec.clone(), todo_state.file_paths.clone())
                }
                None => {
                    crate::debug_log::log_warn(&format!(
                        "[Pipeline] No cached state for todo {}, using fallback prompt",
                        todo_id
                    ));
                    (format!("Execute todo {}", todo_id), None, vec![])
                }
            };
            let psp = cache
                .get_phase(&assignment.mission_id, &assignment.phase_id)
                .and_then(|p| p.config.get("system_prompt"))
                .and_then(|v| v.as_str())
                .map(String::from);
            (desc, sp, fps, psp)
        };

        let prompt = build_system_prompt(
            &assignment.phase_name,
            todo_id,
            &description,
            spec.as_ref(),
            &file_paths,
            phase_system_prompt.as_deref(),
        );

        // Spawn Claude Code in tmux session
        let session = spawner
            .spawn(
                todo_id,
                &prompt,
                &effective_cwd,
                model.as_deref(),
                &allowed_tools,
                &[], // MCP servers TODO
                phase_system_prompt.as_deref(),
            )
            .await?;

        // Monitor tmux session until completion
        let result = monitor_tmux_session(
            session,
            todo_id.clone(),
            assignment.mission_id.clone(),
            assignment.phase_id.clone(),
            instance_id.clone(),
            mqtt.clone(),
            assignment.workspace.clone(),
            app.clone(),
        )
        .await;

        match result {
            Ok(()) => {
                spawner.mark_completed(todo_id).await;
            }
            Err(e) => {
                crate::debug_log::log_error(&format!(
                    "[Pipeline] Todo {} failed: {}",
                    todo_id, e
                ));
                spawner.mark_failed(todo_id).await;
                break;
            }
        }
    }

    crate::debug_log::log_info(&format!(
        "[Pipeline] Phase {} execution complete",
        assignment.phase_name
    ));

    Ok(())
}

/// Find the repo path for an assignment.
fn find_repo_for_assignment(
    assignment: &PhaseAssignment,
    workspace_mount: &Path,
) -> Result<PathBuf, String> {
    if let Some(repo_id) = assignment.config.get("repo_id").and_then(|v| v.as_str()) {
        let repo_path = workspace_mount.join(repo_id);
        if repo_path.join(".git").exists() {
            return Ok(repo_path);
        }
    }

    if let Ok(entries) = std::fs::read_dir(workspace_mount) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join(".git").exists() {
                return Ok(path);
            }
        }
    }

    Err(format!(
        "No git repository found in workspace mount: {}",
        workspace_mount.display()
    ))
}
