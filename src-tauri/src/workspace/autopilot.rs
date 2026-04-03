use crate::mqtt::state_cache::MissionStateCache;
use crate::mqtt::types::WorkspaceRegistryMessage;
use crate::workspace::git;
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionSetupResult {
    pub mission_id: String,
    pub title: String,
    pub repos_cloned: Vec<String>,
    pub workspace_file: Option<String>,
    pub worktrees_created: u32,
    pub errors: Vec<String>,
}

/// React to a WorkspaceRegistryMessage: for each mission, clone missing repos
/// and create worktrees + VS Code workspace file.
pub async fn setup_mission_workspaces(
    registry: &WorkspaceRegistryMessage,
    workspace_mount: &Path,
    app: &tauri::AppHandle,
) -> Vec<MissionSetupResult> {
    let mut results = Vec::new();

    for mission in &registry.missions {
        // Only set up missions that are in an active state
        let dominated = matches!(
            mission.status.as_str(),
            "approved" | "executing" | "review" | "ready"
        );
        if !dominated {
            continue;
        }

        let mut repos_cloned = Vec::new();
        let mut errors = Vec::new();
        let mut available_repo_ids = Vec::new();

        // Clone missing repos
        for repo in &mission.repos {
            let repo_name = repo
                .repo_id
                .rsplit('/')
                .next()
                .unwrap_or(&repo.repo_id);
            let target = workspace_mount.join(repo_name);

            if target.join(".git").exists() {
                available_repo_ids.push(repo_name.to_string());
                continue;
            }

            if let Some(url) = &repo.repo_url {
                match git::clone_repo(url, &target, repo.branch.as_deref()) {
                    Ok(_) => {
                        repos_cloned.push(repo_name.to_string());
                        available_repo_ids.push(repo_name.to_string());
                        crate::debug_log::log_info(&format!(
                            "[Autopilot] Cloned {} for mission {}",
                            repo_name, mission.title
                        ));
                    }
                    Err(e) => {
                        errors.push(format!("Clone {} failed: {}", repo_name, e));
                        crate::debug_log::log_error(&format!(
                            "[Autopilot] Clone {} failed: {}",
                            repo_name, e
                        ));
                    }
                }
            }
        }

        // Create worktrees + VS Code workspace
        let mut workspace_file = None;
        let mut worktrees_created = 0;

        if !available_repo_ids.is_empty() {
            let result = git::setup_mission_worktrees(
                workspace_mount,
                &mission.mission_id,
                &available_repo_ids,
            );
            worktrees_created = result.worktrees_created;
            if !result.workspace_file.is_empty() {
                workspace_file = Some(result.workspace_file.clone());
            }
            for err in &result.errors {
                errors.push(err.clone());
            }
        }

        let setup_result = MissionSetupResult {
            mission_id: mission.mission_id.clone(),
            title: mission.title.clone(),
            repos_cloned: repos_cloned.clone(),
            workspace_file: workspace_file.clone(),
            worktrees_created,
            errors: errors.clone(),
        };

        // Emit to frontend
        if worktrees_created > 0 || !repos_cloned.is_empty() {
            use tauri::Emitter;
            let _ = app.emit("autopilot-workspace-ready", &setup_result);
            crate::debug_log::log_info(&format!(
                "[Autopilot] Workspace ready for '{}': {} repos cloned, {} worktrees",
                mission.title,
                repos_cloned.len(),
                worktrees_created
            ));
        }

        results.push(setup_result);
    }

    results
}

/// Check cached phases for human-targeted work and send notifications.
pub async fn check_human_phases(
    cache: &MissionStateCache,
    app: &tauri::AppHandle,
    notified: &mut HashSet<String>,
) {
    for phase in cache.all_phases() {
        // Check execution_target.kind == "person"
        let is_human = phase
            .execution_target
            .as_ref()
            .and_then(|et| et.get("kind"))
            .and_then(|k| k.as_str())
            == Some("person");

        if !is_human {
            continue;
        }

        // Only notify for phases that need attention
        let needs_attention = matches!(phase.status.as_str(), "ready" | "running" | "pending");
        if !needs_attention {
            continue;
        }

        let key = format!("{}:{}", phase.mission_id, phase.phase_id);
        if notified.contains(&key) {
            continue;
        }

        // Look up mission title from plan
        let mission_title = cache
            .get_plan(&phase.mission_id)
            .map(|p| p.title.clone())
            .unwrap_or_else(|| phase.mission_id.clone());

        // Derive workspace file path
        let short_mid = if phase.mission_id.len() > 8 {
            &phase.mission_id[..8]
        } else {
            &phase.mission_id
        };

        // Send desktop notification
        {
            use tauri_plugin_notification::NotificationExt;
            let _ = app
                .notification()
                .builder()
                .title(format!("Weaver: {} needs you", phase.name))
                .body(format!("Mission: {}", mission_title))
                .show();
        }

        // Emit to frontend
        use tauri::Emitter;
        let _ = app.emit(
            "autopilot-human-needed",
            serde_json::json!({
                "missionId": phase.mission_id,
                "missionTitle": mission_title,
                "phaseId": phase.phase_id,
                "phaseName": phase.name,
                "workspaceDir": format!(".worktrees/{}", short_mid),
            }),
        );

        crate::debug_log::log_info(&format!(
            "[Autopilot] Human phase detected: {} ({}) for mission '{}'",
            phase.name, phase.phase_id, mission_title
        ));

        notified.insert(key);
    }
}
