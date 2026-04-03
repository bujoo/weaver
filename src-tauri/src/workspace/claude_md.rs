use crate::mqtt::state_cache::MissionStateCache;
use crate::mqtt::types::TodoStateMessage;
use std::collections::HashSet;
use std::path::Path;

/// Render a lean CLAUDE.md (<100 lines) for a mission worktree.
/// Returns None if the plan is not yet cached.
pub fn render_claude_md(cache: &MissionStateCache, mission_id: &str) -> Option<String> {
    let plan = cache.get_plan(mission_id)?;
    let phases = cache.get_phases_for_mission(mission_id);
    let active = cache.get_active_phase(mission_id);

    let mut md = String::with_capacity(4096);

    // Header
    md.push_str(&format!("# {}\n\n", plan.title));
    md.push_str(&format!(
        "Mission: `{}` | Scope: {} | Status: {}\n\n",
        plan.mission_id, plan.scope, plan.status
    ));

    // Objectives (extracted from mission_context -- trim to essentials)
    if let Some(ctx) = &plan.mission_context {
        md.push_str("## Objectives\n\n");
        // Extract objectives and acceptance criteria sections
        let trimmed = extract_objectives(ctx);
        md.push_str(&trimmed);
        md.push_str("\n\n");
    }

    // Phase overview table
    md.push_str("## Plan\n\n");
    for phase in &phases {
        let marker = if active
            .map(|a| a.phase_id == phase.phase_id)
            .unwrap_or(false)
        {
            ">>"
        } else {
            "  "
        };
        let status_icon = match phase.status.as_str() {
            "completed" => "[x]",
            "running" | "executing" => "[~]",
            "pending" => "[ ]",
            _ => "[ ]",
        };
        md.push_str(&format!(
            "{} {} {} ({}) - {} todos\n",
            marker, status_icon, phase.name, phase.phase_id, phase.todo_count
        ));
    }
    md.push('\n');

    // Active phase detail
    if let Some(ap) = active {
        md.push_str(&format!("## Active Phase: {} ({})\n\n", ap.name, ap.phase_id));

        let todos = cache.get_todos_for_phase(mission_id, &ap.phase_id);

        // Deduplicated file paths
        let file_paths: Vec<&str> = todos
            .iter()
            .flat_map(|t| t.file_paths.iter().map(|s| s.as_str()))
            .collect::<HashSet<&str>>()
            .into_iter()
            .collect();

        if !file_paths.is_empty() {
            md.push_str("Files to modify:\n");
            for fp in &file_paths {
                md.push_str(&format!("- `{}`\n", fp));
            }
            md.push('\n');
        }

        // Active phase constraints (from first todo's spec that has them)
        let mut constraints_shown = false;
        for todo in &todos {
            if constraints_shown {
                break;
            }
            if let Some(spec) = &todo.spec {
                if let Some(constraints) = spec.get("constraints").and_then(|c| c.as_array()) {
                    if !constraints.is_empty() {
                        md.push_str("Key constraints:\n");
                        for c in constraints.iter().take(5) {
                            if let Some(s) = c.as_str() {
                                md.push_str(&format!("- {}\n", s));
                            }
                        }
                        md.push('\n');
                        constraints_shown = true;
                    }
                }
            }
        }

        // Todo list (brief)
        md.push_str("Todos:\n");
        for todo in &todos {
            let check = match todo.status.as_str() {
                "completed" => "[x]",
                "running" => "[~]",
                _ => "[ ]",
            };
            md.push_str(&format!(
                "- {} **{}** ({}): {}\n",
                check,
                todo.todo_id,
                todo.role,
                truncate(&todo.description, 80)
            ));
        }
        md.push('\n');

        // Spec references
        md.push_str("Detailed specs per todo: see `.weaver/specs/{todo_id}.md`\n\n");
    }

    // Workflow section
    md.push_str("## Workflow\n\n");
    md.push_str("- This worktree is managed by Weaver (ContextHub orchestration)\n");
    md.push_str("- Full mission plan: `.weaver/mission.json`\n");
    md.push_str("- Stay within listed file paths for the active phase\n");
    md.push_str("- Use acceptance criteria above as quality gates\n");
    md.push_str("- When blocked or missing context, document the gap clearly\n");

    Some(md)
}

/// Render a full spec markdown file for a single todo.
pub fn render_todo_spec_md(todo: &TodoStateMessage) -> String {
    let mut md = String::with_capacity(2048);

    md.push_str(&format!("# {} - {}\n\n", todo.todo_id, todo.role));
    md.push_str(&format!("{}\n\n", todo.description));

    if let Some(spec) = &todo.spec {
        if let Some(name) = spec.get("name").and_then(|v| v.as_str()) {
            let location = spec.get("location").and_then(|v| v.as_str()).unwrap_or("");
            md.push_str(&format!("**Component:** {} @ `{}`\n\n", name, location));
        }

        if let Some(summary) = spec.get("summary").and_then(|v| v.as_str()) {
            md.push_str(&format!("{}\n\n", summary));
        }

        // Inputs
        if let Some(inputs) = spec.get("inputs").and_then(|v| v.as_array()) {
            if !inputs.is_empty() {
                md.push_str("## Inputs\n\n");
                for inp in inputs {
                    if let Some(s) = inp.as_str() {
                        md.push_str(&format!("- {}\n", s));
                    }
                }
                md.push('\n');
            }
        }

        // Outputs
        if let Some(outputs) = spec.get("outputs").and_then(|v| v.as_array()) {
            if !outputs.is_empty() {
                md.push_str("## Outputs\n\n");
                for out in outputs {
                    if let Some(s) = out.as_str() {
                        md.push_str(&format!("- {}\n", s));
                    }
                }
                md.push('\n');
            }
        }

        // Behavior steps
        if let Some(behavior) = spec.get("behavior").and_then(|v| v.as_array()) {
            md.push_str("## Behavior\n\n");
            for (i, step) in behavior.iter().enumerate() {
                if let Some(s) = step.as_str() {
                    md.push_str(&format!("{}. {}\n", i + 1, s));
                }
            }
            md.push('\n');
        }

        // Constraints
        if let Some(constraints) = spec.get("constraints").and_then(|v| v.as_array()) {
            md.push_str("## Constraints\n\n");
            for c in constraints {
                if let Some(s) = c.as_str() {
                    md.push_str(&format!("- {}\n", s));
                }
            }
            md.push('\n');
        }

        // Edge cases
        if let Some(edge_cases) = spec.get("edge_cases").and_then(|v| v.as_array()) {
            md.push_str("## Edge Cases\n\n");
            for e in edge_cases {
                if let Some(s) = e.as_str() {
                    md.push_str(&format!("- {}\n", s));
                }
            }
            md.push('\n');
        }

        // References
        if let Some(refs) = spec.get("references").and_then(|v| v.as_array()) {
            if !refs.is_empty() {
                md.push_str("## References\n\n");
                for r in refs {
                    let label = r.get("label").and_then(|v| v.as_str()).unwrap_or("ref");
                    let target = r.get("target").and_then(|v| v.as_str()).unwrap_or("");
                    let desc = r.get("description").and_then(|v| v.as_str());
                    if let Some(d) = desc {
                        md.push_str(&format!("- **{}** (`{}`): {}\n", label, target, d));
                    } else {
                        md.push_str(&format!("- **{}** (`{}`)\n", label, target));
                    }
                }
                md.push('\n');
            }
        }
    }

    // File paths
    if !todo.file_paths.is_empty() {
        md.push_str("## Files\n\n");
        for fp in &todo.file_paths {
            md.push_str(&format!("- `{}`\n", fp));
        }
    }

    md
}

/// Write CLAUDE.md + .weaver/ directory into a worktree.
/// Returns Ok(true) if written, Ok(false) if plan not cached.
pub fn write_mission_context(
    cache: &MissionStateCache,
    mission_id: &str,
    worktree_path: &Path,
    plan_json: Option<&serde_json::Value>,
) -> Result<bool, String> {
    // Render CLAUDE.md
    let claude_md = match render_claude_md(cache, mission_id) {
        Some(content) => content,
        None => {
            crate::debug_log::log_info(&format!(
                "[ClaudeMD] Plan not cached for {}, skipping",
                mission_id
            ));
            return Ok(false);
        }
    };

    // Write CLAUDE.md
    let claude_md_path = worktree_path.join("CLAUDE.md");
    std::fs::write(&claude_md_path, &claude_md)
        .map_err(|e| format!("Failed to write CLAUDE.md: {}", e))?;

    // Create .weaver/ directory
    let weaver_dir = worktree_path.join(".weaver");
    let specs_dir = weaver_dir.join("specs");
    std::fs::create_dir_all(&specs_dir)
        .map_err(|e| format!("Failed to create .weaver/specs/: {}", e))?;

    // Write mission.json (full plan dump if available)
    if let Some(plan) = plan_json {
        let mission_json_path = weaver_dir.join("mission.json");
        let json_str = serde_json::to_string_pretty(plan)
            .map_err(|e| format!("Failed to serialize mission.json: {}", e))?;
        std::fs::write(&mission_json_path, json_str)
            .map_err(|e| format!("Failed to write mission.json: {}", e))?;
    }

    // Write per-todo spec files
    let phases = cache.get_phases_for_mission(mission_id);
    let mut spec_count = 0u32;
    for phase in &phases {
        let todos = cache.get_todos_for_phase(mission_id, &phase.phase_id);
        for todo in &todos {
            let spec_md = render_todo_spec_md(todo);
            let spec_path = specs_dir.join(format!("{}.md", todo.todo_id));
            std::fs::write(&spec_path, spec_md)
                .map_err(|e| format!("Failed to write spec {}: {}", todo.todo_id, e))?;
            spec_count += 1;
        }
    }

    let line_count = claude_md.lines().count();
    crate::debug_log::log_info(&format!(
        "[ClaudeMD] Wrote CLAUDE.md ({} lines) + {} spec files to {}",
        line_count,
        spec_count,
        worktree_path.display()
    ));

    Ok(true)
}

/// Extract objectives and acceptance criteria from mission_context markdown.
fn extract_objectives(ctx: &str) -> String {
    let mut result = String::new();
    let mut in_section = false;

    for line in ctx.lines() {
        // Start capturing at "Objectives" or "Acceptance Criteria" headers
        if line.starts_with("###") && (line.contains("Objective") || line.contains("Acceptance")) {
            in_section = true;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Stop at next same-level or higher header
        if in_section && line.starts_with("###") && !line.contains("Objective") && !line.contains("Acceptance") {
            // Check if this is a different section at same depth
            if line.starts_with("### ") {
                in_section = false;
                continue;
            }
        }

        if in_section {
            result.push_str(line);
            result.push('\n');
        }
    }

    // If no structured sections found, take first ~500 chars as fallback
    if result.trim().is_empty() {
        let truncated: String = ctx.chars().take(500).collect();
        if truncated.len() < ctx.len() {
            return format!("{}...", truncated);
        }
        return truncated;
    }

    result.trim().to_string()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
