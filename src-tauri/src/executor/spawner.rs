use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnedProcess {
    pub todo_id: String,
    pub pid: u32,
    pub status: String, // running | completed | failed | killed
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

    pub async fn spawn(
        &self,
        todo_id: &str,
        prompt: &str,
        cwd: &Path,
        model: Option<&str>,
        allowed_tools: &[String],
        mcp_servers: &[String],
        system_prompt: Option<&str>,
    ) -> Result<Child, String> {
        let mut cmd = Command::new("claude");
        cmd.current_dir(cwd);
        cmd.arg("--print"); // Non-interactive mode

        // Model
        if let Some(m) = model {
            cmd.arg("--model").arg(m);
        }

        // Allowed tools
        for tool in allowed_tools {
            cmd.arg("--allowedTools").arg(tool);
        }

        // MCP servers (passed as --mcp-server flags)
        for mcp in mcp_servers {
            cmd.arg("--mcp-server").arg(mcp);
        }

        // System prompt
        if let Some(sp) = system_prompt {
            cmd.arg("--system-prompt").arg(sp);
        }

        // The prompt/task description
        cmd.arg("--").arg(prompt);

        // Capture stdout/stderr
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Don't inherit stdin (non-interactive)
        cmd.stdin(Stdio::null());

        let child = cmd.spawn().map_err(|e| format!("Failed to spawn claude: {}", e))?;

        let pid = child.id().unwrap_or(0);
        crate::debug_log::log_info(&format!(
            "[Executor] Spawned Claude Code for todo {} (pid {})",
            todo_id, pid
        ));

        self.processes.lock().await.insert(
            todo_id.to_string(),
            SpawnedProcess {
                todo_id: todo_id.to_string(),
                pid,
                status: "running".into(),
            },
        );

        Ok(child)
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

/// Build the system prompt from a TodoSpec JSON value.
pub fn build_system_prompt(
    phase_name: &str,
    todo_id: &str,
    description: &str,
    spec: Option<&serde_json::Value>,
) -> String {
    let mut prompt = format!(
        "You are executing a weaver plan task.\n\n\
         Phase: {}\n\
         Todo: {}\n\
         Description: {}\n",
        phase_name, todo_id, description
    );

    if let Some(spec) = spec {
        if let Some(behavior) = spec.get("behavior").and_then(|b| b.as_array()) {
            prompt.push_str("\nBehavior steps:\n");
            for (i, step) in behavior.iter().enumerate() {
                if let Some(s) = step.as_str() {
                    prompt.push_str(&format!("{}. {}\n", i + 1, s));
                }
            }
        }
        if let Some(constraints) = spec.get("constraints").and_then(|c| c.as_array()) {
            prompt.push_str("\nConstraints:\n");
            for c in constraints {
                if let Some(s) = c.as_str() {
                    prompt.push_str(&format!("- {}\n", s));
                }
            }
        }
        if let Some(edge_cases) = spec.get("edge_cases").and_then(|e| e.as_array()) {
            prompt.push_str("\nEdge cases to handle:\n");
            for e in edge_cases {
                if let Some(s) = e.as_str() {
                    prompt.push_str(&format!("- {}\n", s));
                }
            }
        }
    }

    prompt.push_str("\nComplete this task. Report progress as you go. When done, the todo will be marked complete automatically.\n");
    prompt
}
