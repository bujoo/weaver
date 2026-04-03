use crate::executor::monitor::monitor_tmux_session;
use crate::executor::spawner::ClaudeCodeSpawner;
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
    _state_cache: Arc<Mutex<MissionStateCache>>,
    instance_id: String,
    abort_rx: watch::Receiver<bool>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mission_short = if assignment.mission_id.len() > 8 {
        &assignment.mission_id[..8]
    } else {
        &assignment.mission_id
    };

    // Check if autopilot already set up the workspace (worktrees at .worktrees/{mid_short}/)
    let weaver_cwd = workspace_mount
        .join(".worktrees")
        .join(mission_short)
        .join("weaver");

    let effective_cwd = if weaver_cwd.exists() {
        // Autopilot already created worktrees + weaver/ folder -- use it
        crate::debug_log::log_info(&format!(
            "[Pipeline] Using existing weaver/ workspace at {}",
            weaver_cwd.display()
        ));
        weaver_cwd
    } else {
        // Fallback: create worktree on the fly (no autopilot ran)
        let repo_path = find_repo_for_assignment(&assignment, workspace_mount)?;
        let branch_name = git::mission_branch_name(&assignment.mission_id, &assignment.phase_id);
        let worktree_path = workspace_mount
            .join(".worktrees")
            .join(mission_short)
            .join(&assignment.phase_id);
        let cwd = git::create_worktree(&repo_path, &worktree_path, &branch_name)?;
        cwd
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

        // Short prompt -- Claude Code reads CLAUDE.md + .weaver/specs/{todo_id}.md
        // No need for a massive system prompt; the weaver/ folder has everything
        let prompt = format!(
            "Execute todo {}. Read .weaver/specs/{}.md for the full spec. \
             Follow the behavior steps, constraints, and edge cases defined there.",
            todo_id, todo_id
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
                None,
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
