use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoStatus {
    pub path: String,
    pub name: String,
    pub branch: String,
    pub clean: bool,
    pub worktrees: Vec<WorktreeInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: String,
}

/// Clone a git repository. Skips if target already exists.
pub fn clone_repo(url: &str, target: &Path, branch: Option<&str>) -> Result<PathBuf, String> {
    if target.join(".git").exists() {
        crate::debug_log::log_info(&format!(
            "[Git] Repo already exists at {}",
            target.display()
        ));
        return Ok(target.to_path_buf());
    }

    let mut cmd = Command::new("git");
    cmd.arg("clone");
    if let Some(b) = branch {
        cmd.arg("-b").arg(b);
    }
    cmd.arg("--depth").arg("1");
    cmd.arg(url).arg(target);

    let output = cmd.output().map_err(|e| format!("git clone failed: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git clone failed: {}", stderr));
    }

    crate::debug_log::log_info(&format!("[Git] Cloned {} to {}", url, target.display()));
    Ok(target.to_path_buf())
}

/// Checkout a branch in an existing repo.
pub fn checkout_branch(repo_path: &Path, branch: &str) -> Result<(), String> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["checkout", branch])
        .output()
        .map_err(|e| format!("git checkout failed: {}", e))?;

    if !output.status.success() {
        // Try creating the branch
        let output = Command::new("git")
            .current_dir(repo_path)
            .args(["checkout", "-b", branch])
            .output()
            .map_err(|e| format!("git checkout -b failed: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git checkout -b failed: {}", stderr));
        }
    }

    crate::debug_log::log_info(&format!(
        "[Git] Checked out {} in {}",
        branch,
        repo_path.display()
    ));
    Ok(())
}

/// Create a git worktree for isolated phase execution.
pub fn create_worktree(
    repo_path: &Path,
    worktree_path: &Path,
    branch_name: &str,
) -> Result<PathBuf, String> {
    if worktree_path.exists() {
        crate::debug_log::log_info(&format!(
            "[Git] Worktree already exists at {}",
            worktree_path.display()
        ));
        return Ok(worktree_path.to_path_buf());
    }

    // Create parent directory
    if let Some(parent) = worktree_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create worktree parent dir: {}", e))?;
    }

    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "add", "-b", branch_name])
        .arg(worktree_path)
        .output()
        .map_err(|e| format!("git worktree add failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If branch already exists, try without -b
        if stderr.contains("already exists") {
            let output = Command::new("git")
                .current_dir(repo_path)
                .args(["worktree", "add"])
                .arg(worktree_path)
                .arg(branch_name)
                .output()
                .map_err(|e| format!("git worktree add (existing branch) failed: {}", e))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("git worktree add failed: {}", stderr));
            }
        } else {
            return Err(format!("git worktree add failed: {}", stderr));
        }
    }

    crate::debug_log::log_info(&format!(
        "[Git] Created worktree at {} on branch {}",
        worktree_path.display(),
        branch_name
    ));
    Ok(worktree_path.to_path_buf())
}

/// Create a standardized mission branch name.
/// Uses dash separator to avoid git ref conflicts (slash creates hierarchy).
pub fn mission_branch_name(mission_id: &str, _phase_id: &str) -> String {
    let short_mid = if mission_id.len() > 8 {
        &mission_id[..8]
    } else {
        mission_id
    };
    format!("weaver-{}", short_mid)
}

/// Remove a git worktree.
pub fn remove_worktree(repo_path: &Path, worktree_path: &Path) -> Result<(), String> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "remove", "--force"])
        .arg(worktree_path)
        .output()
        .map_err(|e| format!("git worktree remove failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git worktree remove failed: {}", stderr));
    }
    Ok(())
}

/// Get status of a repo: current branch, clean/dirty.
pub fn get_repo_status(repo_path: &Path) -> Result<RepoStatus, String> {
    let name = repo_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Get current branch
    let branch_output = Command::new("git")
        .current_dir(repo_path)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|e| format!("git rev-parse failed: {}", e))?;
    let branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();

    // Check if clean
    let status_output = Command::new("git")
        .current_dir(repo_path)
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| format!("git status failed: {}", e))?;
    let clean = String::from_utf8_lossy(&status_output.stdout)
        .trim()
        .is_empty();

    // List worktrees
    let wt_output = Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "list", "--porcelain"])
        .output()
        .map_err(|e| format!("git worktree list failed: {}", e))?;
    let worktrees = parse_worktree_list(&String::from_utf8_lossy(&wt_output.stdout));

    Ok(RepoStatus {
        path: repo_path.to_string_lossy().to_string(),
        name,
        branch,
        clean,
        worktrees,
    })
}

fn parse_worktree_list(output: &str) -> Vec<WorktreeInfo> {
    let mut worktrees = Vec::new();
    let mut current_path = String::new();
    let mut current_branch = String::new();

    for line in output.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if !current_path.is_empty() {
                worktrees.push(WorktreeInfo {
                    path: current_path.clone(),
                    branch: current_branch.clone(),
                });
            }
            current_path = path.to_string();
            current_branch = String::new();
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
            current_branch = branch.to_string();
        }
    }
    if !current_path.is_empty() {
        worktrees.push(WorktreeInfo {
            path: current_path,
            branch: current_branch,
        });
    }

    worktrees
}

// ── Mission workspace setup ─────────────────────────────────────────

/// Result of setting up a mission workspace.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionWorkspaceResult {
    pub mission_id: String,
    pub workspace_file: String,
    pub worktrees_created: u32,
    pub errors: Vec<String>,
}

/// Set up worktrees for a mission: one worktree per repo, shared across all phases.
///
/// Structure:
///   {mount}/.worktrees/{mid_short}/
///     mission.code-workspace
///     contexthub-brain/   (worktree, branch: weaver/{mid_short})
///     ocxp-engine/        (worktree, branch: weaver/{mid_short})
pub fn setup_mission_worktrees(
    mount: &Path,
    mission_id: &str,
    repo_ids: &[String],
) -> MissionWorkspaceResult {
    let short_mid = if mission_id.len() > 8 {
        &mission_id[..8]
    } else {
        mission_id
    };

    let worktrees_dir = mount.join(".worktrees").join(short_mid);
    let _ = std::fs::create_dir_all(&worktrees_dir);

    let mut folders: Vec<serde_json::Value> = Vec::new();
    let mut created = 0u32;
    let mut errors = Vec::new();

    let branch = format!("weaver-{}", short_mid);

    for repo_id in repo_ids {
        let repo_path = mount.join(repo_id);
        if !repo_path.join(".git").exists() {
            errors.push(format!("Repo '{}' not found at {}", repo_id, repo_path.display()));
            continue;
        }

        let wt_path = worktrees_dir.join(repo_id);

        match create_worktree(&repo_path, &wt_path, &branch) {
            Ok(_) => {
                created += 1;
                folders.push(serde_json::json!({
                    "path": repo_id,
                    "name": repo_id
                }));
            }
            Err(e) => {
                errors.push(format!("{}: {}", repo_id, e));
            }
        }
    }

    // Generate VS Code workspace file
    let workspace_file = worktrees_dir.join("mission.code-workspace");
    let workspace_json = serde_json::json!({
        "folders": folders,
        "settings": {}
    });
    if let Err(e) = std::fs::write(
        &workspace_file,
        serde_json::to_string_pretty(&workspace_json).unwrap_or_default(),
    ) {
        errors.push(format!("Failed to write workspace file: {}", e));
    }

    crate::debug_log::log_info(&format!(
        "[Git] Mission workspace: {} worktrees created, branch '{}', workspace: {}",
        created, branch, workspace_file.display()
    ));

    MissionWorkspaceResult {
        mission_id: mission_id.to_string(),
        workspace_file: workspace_file.to_string_lossy().to_string(),
        worktrees_created: created,
        errors,
    }
}

/// Open a VS Code workspace file.
pub fn open_vscode_workspace(workspace_file: &Path) -> Result<(), String> {
    // Try `code` in PATH first, then macOS app bundle path
    let result = Command::new("code").arg(workspace_file).spawn();
    if result.is_ok() {
        return Ok(());
    }

    // macOS: VS Code app bundle
    let macos_code = "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code";
    Command::new(macos_code)
        .arg(workspace_file)
        .spawn()
        .map_err(|e| format!("Failed to open VS Code: {}", e))?;
    Ok(())
}
