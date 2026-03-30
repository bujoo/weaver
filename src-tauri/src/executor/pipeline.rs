use crate::executor::monitor::monitor_process;
use crate::executor::spawner::{build_system_prompt, ClaudeCodeSpawner};
use crate::mqtt::client::MqttClient;
use crate::mqtt::types::PhaseAssignment;
use crate::workspace::git;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

/// Execute a full phase assignment: create worktree, run todos sequentially.
pub async fn execute_assignment(
    assignment: PhaseAssignment,
    workspace_mount: &Path,
    spawner: Arc<ClaudeCodeSpawner>,
    mqtt: Arc<Mutex<Option<MqttClient>>>,
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

    let cwd = git::create_worktree(&repo_path, &worktree_path, &branch_name)?;

    crate::debug_log::log_info(&format!(
        "[Pipeline] Worktree created at {} for phase {}",
        cwd.display(),
        assignment.phase_name
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

        // Build prompt from todo description
        let prompt = build_system_prompt(
            &assignment.phase_name,
            todo_id,
            &format!("Execute todo {}", todo_id),
            None, // TODO: get spec from retained MQTT state
        );

        // Spawn Claude Code
        let child = spawner
            .spawn(
                todo_id,
                &prompt,
                &cwd,
                model.as_deref(),
                &allowed_tools,
                &[], // MCP servers TODO
                None,
            )
            .await?;

        // Monitor until completion
        let result = monitor_process(
            child,
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
                // Continue to next todo or break depending on strategy
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
/// Uses workspace_mount to locate repos by name from the assignment config.
fn find_repo_for_assignment(
    assignment: &PhaseAssignment,
    workspace_mount: &Path,
) -> Result<PathBuf, String> {
    // Try to get repo from config
    if let Some(repo_id) = assignment.config.get("repo_id").and_then(|v| v.as_str()) {
        let repo_path = workspace_mount.join(repo_id);
        if repo_path.join(".git").exists() {
            return Ok(repo_path);
        }
    }

    // Fallback: find first repo in workspace_mount
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
