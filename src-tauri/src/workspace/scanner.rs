use crate::workspace::git;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStatus {
    pub mount_path: String,
    pub repos: Vec<git::RepoStatus>,
    pub tools: Vec<ToolInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
}

/// Scan a workspace directory for git repos and installed tools.
pub fn scan_workspace(mount_path: &Path) -> WorkspaceStatus {
    let repos = scan_repos(mount_path);
    let tools = scan_tools();

    WorkspaceStatus {
        mount_path: mount_path.to_string_lossy().to_string(),
        repos,
        tools,
    }
}

fn scan_repos(mount_path: &Path) -> Vec<git::RepoStatus> {
    let mut repos = Vec::new();
    let entries = match std::fs::read_dir(mount_path) {
        Ok(e) => e,
        Err(_) => return repos,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join(".git").exists() {
            if let Ok(status) = git::get_repo_status(&path) {
                repos.push(status);
            }
        }
    }

    repos.sort_by(|a, b| a.name.cmp(&b.name));
    repos
}

fn scan_tools() -> Vec<ToolInfo> {
    let tool_checks = vec![
        ("git", "git --version"),
        ("node", "node --version"),
        ("bun", "bun --version"),
        ("cargo", "cargo --version"),
        ("python3", "python3 --version"),
        ("claude", "claude --version"),
        ("npm", "npm --version"),
        ("pip", "pip3 --version"),
    ];

    tool_checks
        .into_iter()
        .map(|(name, cmd)| {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            let result = std::process::Command::new(parts[0])
                .args(&parts[1..])
                .output();
            match result {
                Ok(output) if output.status.success() => {
                    let version = String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .to_string();
                    ToolInfo {
                        name: name.into(),
                        installed: true,
                        version: Some(version),
                    }
                }
                _ => ToolInfo {
                    name: name.into(),
                    installed: false,
                    version: None,
                },
            }
        })
        .collect()
}
