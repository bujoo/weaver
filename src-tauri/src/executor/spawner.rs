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

        // Summary
        if let Some(summary) = spec.get("summary").and_then(|v| v.as_str()) {
            prompt.push_str(&format!("\nSummary: {}\n", summary));
        }

        // Inputs
        if let Some(inputs) = spec.get("inputs").and_then(|v| v.as_array()) {
            if !inputs.is_empty() {
                prompt.push_str("\nInputs:\n");
                for inp in inputs {
                    if let Some(s) = inp.as_str() {
                        prompt.push_str(&format!("- {}\n", s));
                    }
                }
            }
        }

        // Outputs
        if let Some(outputs) = spec.get("outputs").and_then(|v| v.as_array()) {
            if !outputs.is_empty() {
                prompt.push_str("\nOutputs:\n");
                for out in outputs {
                    if let Some(s) = out.as_str() {
                        prompt.push_str(&format!("- {}\n", s));
                    }
                }
            }
        }

        // Behavior steps
        if let Some(behavior) = spec.get("behavior").and_then(|b| b.as_array()) {
            prompt.push_str("\nBehavior steps:\n");
            for (i, step) in behavior.iter().enumerate() {
                if let Some(s) = step.as_str() {
                    prompt.push_str(&format!("{}. {}\n", i + 1, s));
                }
            }
        }

        // Constraints
        if let Some(constraints) = spec.get("constraints").and_then(|c| c.as_array()) {
            prompt.push_str("\nConstraints:\n");
            for c in constraints {
                if let Some(s) = c.as_str() {
                    prompt.push_str(&format!("- {}\n", s));
                }
            }
        }

        // Edge cases
        if let Some(edge_cases) = spec.get("edge_cases").and_then(|e| e.as_array()) {
            prompt.push_str("\nEdge cases to handle:\n");
            for e in edge_cases {
                if let Some(s) = e.as_str() {
                    prompt.push_str(&format!("- {}\n", s));
                }
            }
        }

        // References
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

    // File paths
    if !file_paths.is_empty() {
        prompt.push_str("\nFiles to modify:\n");
        for fp in file_paths {
            prompt.push_str(&format!("- {}\n", fp));
        }
    }

    prompt.push_str("\nComplete this task. Report progress as you go. When done, the todo will be marked complete automatically.\n");
    prompt
}
