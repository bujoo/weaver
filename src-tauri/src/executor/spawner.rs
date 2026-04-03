use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnedProcess {
    pub todo_id: String,
    pub session_name: String,
    pub status: String, // running | completed | failed | killed
}

/// Returned by spawn() -- represents a running tmux session.
pub struct TmuxSession {
    pub session_name: String,
    pub log_path: PathBuf,
    pub todo_id: String,
}

pub struct ClaudeCodeSpawner {
    processes: Arc<Mutex<HashMap<String, SpawnedProcess>>>,
}

impl ClaudeCodeSpawner {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawn Claude Code inside a named tmux session.
    /// The developer can `tmux attach -t {session_name}` to watch.
    pub async fn spawn(
        &self,
        todo_id: &str,
        prompt: &str,
        cwd: &Path,
        model: Option<&str>,
        allowed_tools: &[String],
        mcp_servers: &[String],
        system_prompt: Option<&str>,
    ) -> Result<TmuxSession, String> {
        let session_name = format!("weaver-{}", todo_id.replace('.', "-"));
        let log_path = std::env::temp_dir().join(format!("weaver-{}.log", todo_id));

        // Clean up stale log file
        let _ = std::fs::remove_file(&log_path);

        // Build the claude command string
        let mut claude_args: Vec<String> = vec!["--print".to_string()];

        // Permission mode: autonomous execution
        claude_args.push("--permission-mode".to_string());
        claude_args.push("bypassPermissions".to_string());

        if let Some(m) = model {
            claude_args.push("--model".to_string());
            claude_args.push(m.to_string());
        }

        for tool in allowed_tools {
            claude_args.push("--allowedTools".to_string());
            claude_args.push(tool.clone());
        }

        for mcp in mcp_servers {
            claude_args.push("--mcp-server".to_string());
            claude_args.push(mcp.clone());
        }

        if let Some(sp) = system_prompt {
            claude_args.push("--system-prompt".to_string());
            claude_args.push(sp.to_string());
        }

        claude_args.push("--".to_string());
        claude_args.push(prompt.to_string());

        // Build shell command: set env vars + run claude with args
        // Using shell form so env vars work in tmux
        let escaped_args: Vec<String> = claude_args
            .iter()
            .map(|a| shell_escape(a))
            .collect();
        let shell_cmd = format!(
            "export CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1; claude {}",
            escaped_args.join(" ")
        );

        // Kill existing session with same name (idempotent)
        let _ = Command::new("tmux")
            .args(["kill-session", "-t", &session_name])
            .output()
            .await;

        // Create tmux session
        let output = Command::new("tmux")
            .args([
                "new-session",
                "-d",
                "-s",
                &session_name,
                "-c",
            ])
            .arg(cwd)
            .arg(&shell_cmd)
            .output()
            .await
            .map_err(|e| format!("Failed to create tmux session: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("tmux new-session failed: {}", stderr));
        }

        // Set up output capture via pipe-pane
        let _ = Command::new("tmux")
            .args(["pipe-pane", "-t", &session_name])
            .arg(format!("cat >> {}", log_path.display()))
            .output()
            .await;

        crate::debug_log::log_info(&format!(
            "[Executor] Spawned tmux session '{}' for todo {} in {}",
            session_name,
            todo_id,
            cwd.display()
        ));

        self.processes.lock().await.insert(
            todo_id.to_string(),
            SpawnedProcess {
                todo_id: todo_id.to_string(),
                session_name: session_name.clone(),
                status: "running".into(),
            },
        );

        Ok(TmuxSession {
            session_name,
            log_path,
            todo_id: todo_id.to_string(),
        })
    }

    pub async fn mark_completed(&self, todo_id: &str) {
        if let Some(proc) = self.processes.lock().await.get_mut(todo_id) {
            proc.status = "completed".into();
        }
    }

    pub async fn mark_failed(&self, todo_id: &str) {
        if let Some(proc) = self.processes.lock().await.get_mut(todo_id) {
            proc.status = "failed".into();
        }
    }

    pub async fn get_processes(&self) -> Vec<SpawnedProcess> {
        self.processes.lock().await.values().cloned().collect()
    }

    pub async fn active_count(&self) -> usize {
        self.processes
            .lock()
            .await
            .values()
            .filter(|p| p.status == "running")
            .count()
    }
}

/// Escape a string for safe use in a shell command.
fn shell_escape(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    // If it contains no special chars, return as-is
    if s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/' || c == ':') {
        return s.to_string();
    }
    // Wrap in single quotes, escaping existing single quotes
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Build the system prompt from a TodoSpec JSON value.
pub fn build_system_prompt(
    phase_name: &str,
    todo_id: &str,
    description: &str,
    spec: Option<&serde_json::Value>,
    file_paths: &[String],
    phase_system_prompt: Option<&str>,
) -> String {
    let mut prompt = String::new();

    // Phase-level system prompt override (prepended if present)
    if let Some(psp) = phase_system_prompt {
        prompt.push_str(psp);
        prompt.push_str("\n\n---\n\n");
    }

    prompt.push_str(&format!(
        "You are executing a weaver plan task.\n\n\
         Phase: {}\n\
         Todo: {}\n\
         Description: {}\n",
        phase_name, todo_id, description
    ));

    if let Some(spec) = spec {
        // Component identity
        let name = spec.get("name").and_then(|v| v.as_str());
        let location = spec.get("location").and_then(|v| v.as_str());
        if let (Some(n), Some(l)) = (name, location) {
            prompt.push_str(&format!("\nComponent: {} @ {}\n", n, l));
        }

        if let Some(summary) = spec.get("summary").and_then(|v| v.as_str()) {
            prompt.push_str(&format!("\nSummary: {}\n", summary));
        }

        render_prompt_array(&mut prompt, "Inputs", spec.get("inputs"));
        render_prompt_array(&mut prompt, "Outputs", spec.get("outputs"));

        if let Some(behavior) = spec.get("behavior").and_then(|b| b.as_array()) {
            prompt.push_str("\nBehavior steps:\n");
            for (i, step) in behavior.iter().enumerate() {
                if let Some(s) = step.as_str() {
                    prompt.push_str(&format!("{}. {}\n", i + 1, s));
                }
            }
        }

        render_prompt_array(&mut prompt, "Constraints", spec.get("constraints"));
        render_prompt_array(&mut prompt, "Edge cases to handle", spec.get("edge_cases"));

        if let Some(refs) = spec.get("references").and_then(|r| r.as_array()) {
            if !refs.is_empty() {
                prompt.push_str("\nReference materials:\n");
                for r in refs {
                    let label = r.get("label").and_then(|v| v.as_str()).unwrap_or("ref");
                    let target = r.get("target").and_then(|v| v.as_str()).unwrap_or("");
                    let desc = r.get("description").and_then(|v| v.as_str());
                    if let Some(d) = desc {
                        prompt.push_str(&format!("- {} ({}): {}\n", label, target, d));
                    } else {
                        prompt.push_str(&format!("- {} ({})\n", label, target));
                    }
                }
            }
        }
    }

    if !file_paths.is_empty() {
        prompt.push_str("\nFiles to modify:\n");
        for fp in file_paths {
            prompt.push_str(&format!("- {}\n", fp));
        }
    }

    prompt.push_str("\nComplete this task. Report progress as you go. When done, the todo will be marked complete automatically.\n");
    prompt
}

fn render_prompt_array(prompt: &mut String, heading: &str, value: Option<&serde_json::Value>) {
    if let Some(arr) = value.and_then(|v| v.as_array()) {
        if !arr.is_empty() {
            prompt.push_str(&format!("\n{}:\n", heading));
            for item in arr {
                if let Some(s) = item.as_str() {
                    prompt.push_str(&format!("- {}\n", s));
                }
            }
        }
    }
}
