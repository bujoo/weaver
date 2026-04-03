use crate::mqtt::state_cache::MissionStateCache;
use crate::mqtt::types::TodoStateMessage;
use std::collections::HashSet;
use std::path::Path;

// ── CLAUDE.md Renderer ─────────────────────────────────────────────

/// Render a lean CLAUDE.md (<100 lines) for the weaver/ workspace folder.
pub fn render_claude_md(
    cache: &MissionStateCache,
    mission_id: &str,
    repo_ids: &[String],
) -> Option<String> {
    let plan = cache.get_plan(mission_id)?;
    let phases = cache.get_phases_for_mission(mission_id);
    let active = cache.get_active_phase(mission_id);

    let mut md = String::with_capacity(4096);

    md.push_str(&format!("# {}\n\n", plan.title));
    md.push_str(&format!(
        "Mission: `{}` | Scope: {} | Status: {}\n\n",
        plan.mission_id, plan.scope, plan.status
    ));

    // Objectives
    if let Some(ctx) = &plan.mission_context {
        md.push_str("## Objectives\n\n");
        md.push_str(&extract_objectives(ctx));
        md.push_str("\n\n");
    }

    // Repositories
    if !repo_ids.is_empty() {
        md.push_str("## Repositories\n\n");
        for repo in repo_ids {
            md.push_str(&format!("- `../{}` \n", repo));
        }
        md.push('\n');
    }

    // Phase overview
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
        let icon = match phase.status.as_str() {
            "completed" => "[x]",
            "running" | "executing" => "[~]",
            _ => "[ ]",
        };
        md.push_str(&format!(
            "{} {} {} ({}) - {} todos\n",
            marker, icon, phase.name, phase.phase_id, phase.todo_count
        ));
    }
    md.push('\n');

    // Active phase detail
    if let Some(ap) = active {
        md.push_str(&format!("## Active Phase: {} ({})\n\n", ap.name, ap.phase_id));

        let todos = cache.get_todos_for_phase(mission_id, &ap.phase_id);

        // File paths
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

        // Brief todo list
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
    }

    // Workflow
    md.push_str("## Workflow\n\n");
    md.push_str("- Managed by Weaver (ContextHub orchestration)\n");
    md.push_str("- Detailed specs: `.weaver/specs/{todo_id}.md`\n");
    md.push_str("- Full plan: `.weaver/mission.json`\n");
    md.push_str("- Stay within listed file paths for the active phase\n");
    md.push_str("- Use acceptance criteria above as quality gates\n");
    md.push_str("- When blocked or missing context, document the gap clearly\n");

    Some(md)
}

// ── Todo Spec Renderer ─────────────────────────────────────────────

/// Render a full spec markdown for a single todo.
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
        render_string_array(&mut md, "## Inputs", spec.get("inputs"));
        render_string_array(&mut md, "## Outputs", spec.get("outputs"));
        if let Some(behavior) = spec.get("behavior").and_then(|v| v.as_array()) {
            md.push_str("## Behavior\n\n");
            for (i, step) in behavior.iter().enumerate() {
                if let Some(s) = step.as_str() {
                    md.push_str(&format!("{}. {}\n", i + 1, s));
                }
            }
            md.push('\n');
        }
        render_string_array(&mut md, "## Constraints", spec.get("constraints"));
        render_string_array(&mut md, "## Edge Cases", spec.get("edge_cases"));
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
    if !todo.file_paths.is_empty() {
        md.push_str("## Files\n\n");
        for fp in &todo.file_paths {
            md.push_str(&format!("- `{}`\n", fp));
        }
    }
    md
}

// ── .claude/ Config Generators ─────────────────────────────────────

/// Generate .claude/settings.json with permissions for required_tools.
pub fn render_settings_json(cache: &MissionStateCache, mission_id: &str) -> Option<String> {
    let active = cache.get_active_phase(mission_id)?;
    let tools: Vec<String> = active
        .config
        .get("required_tools")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|t| format!("Bash({}:*)", t))
                .collect()
        })
        .unwrap_or_default();

    let settings = serde_json::json!({
        "permissions": {
            "allow": tools
        }
    });
    Some(serde_json::to_string_pretty(&settings).unwrap_or_default())
}

/// Generate an agent markdown file from a plan role definition.
pub fn render_agent_md(role: &serde_json::Value) -> Option<String> {
    let name = role.get("name")?.as_str()?;
    let model = role
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("sonnet");
    let description = role
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut md = String::with_capacity(512);
    md.push_str(&format!("# Agent: {}\n\n", name));
    md.push_str(&format!("Model: {}\n\n", model));
    md.push_str(&format!("## Role\n\n{}\n\n", description));
    md.push_str("## Guidelines\n\n");

    match name {
        "architect" => {
            md.push_str("- Focus on design investigation and documentation\n");
            md.push_str("- Do NOT modify code unless explicitly required\n");
            md.push_str("- Document findings for downstream implementers\n");
            md.push_str("- Validate assumptions against actual codebase\n");
        }
        "implementer" => {
            md.push_str("- Implement changes within the file paths listed in todo specs\n");
            md.push_str("- Follow existing code patterns and conventions\n");
            md.push_str("- Write minimal, focused changes\n");
            md.push_str("- Run required tools (linters/tests) before completing\n");
        }
        "reviewer" => {
            md.push_str("- Write comprehensive tests covering happy path and edge cases\n");
            md.push_str("- Verify acceptance criteria are met\n");
            md.push_str("- Check for regressions in existing functionality\n");
            md.push_str("- Review code quality and adherence to constraints\n");
        }
        _ => {
            md.push_str(&format!(
                "- Execute tasks assigned to the {} role\n",
                name
            ));
            md.push_str("- Follow constraints and spec requirements\n");
        }
    }

    Some(md)
}

/// Generate .claude/skills/phase-workflow/SKILL.md for the active phase.
pub fn render_skill_md(cache: &MissionStateCache, mission_id: &str) -> Option<String> {
    let active = cache.get_active_phase(mission_id)?;
    let plan = cache.get_plan(mission_id)?;
    let todos = cache.get_todos_for_phase(mission_id, &active.phase_id);

    let mut md = String::with_capacity(2048);
    md.push_str(&format!(
        "# Phase Workflow: {} ({})\n\n",
        active.name, active.phase_id
    ));
    md.push_str(&format!(
        "Execution workflow for phase '{}' of mission '{}'.\n\n",
        active.name, plan.title
    ));

    // Execution order
    md.push_str("## Execution Order\n\n");
    for todo in &todos {
        let icon = match todo.status.as_str() {
            "completed" => "[x]",
            "running" => "[~]",
            _ => "[ ]",
        };
        md.push_str(&format!(
            "- {} **{}** ({}) - {}\n",
            icon,
            todo.todo_id,
            todo.role,
            truncate(&todo.description, 80)
        ));
        if !todo.blocked_by.is_empty() {
            md.push_str(&format!("  Blocked by: {}\n", todo.blocked_by.join(", ")));
        }
    }
    md.push('\n');

    // Verification tools
    if let Some(tools) = active
        .config
        .get("required_tools")
        .and_then(|v| v.as_array())
    {
        md.push_str("## Verification\n\nRun before marking a todo complete:\n\n");
        for tool in tools {
            if let Some(t) = tool.as_str() {
                md.push_str(&format!("- `{}`\n", t));
            }
        }
        md.push('\n');
    }

    md.push_str("## Specs\n\nSee `.weaver/specs/{todo_id}.md` for full specifications.\n");

    Some(md)
}

/// Generate .claude/rules/phase-constraints.md from active phase todos.
pub fn render_phase_constraints_md(
    cache: &MissionStateCache,
    mission_id: &str,
) -> Option<String> {
    let active = cache.get_active_phase(mission_id)?;
    let todos = cache.get_todos_for_phase(mission_id, &active.phase_id);

    let mut md = String::with_capacity(2048);
    md.push_str(&format!("# Phase Constraints: {}\n\n", active.name));

    // Phase-level system prompt (trimmed)
    if let Some(sp) = active
        .config
        .get("system_prompt")
        .and_then(|v| v.as_str())
    {
        md.push_str("## Phase Directives\n\n");
        let trimmed: String = sp.chars().take(500).collect();
        md.push_str(&trimmed);
        if sp.len() > 500 {
            md.push_str("...");
        }
        md.push_str("\n\n");
    }

    // Aggregated constraints from todos
    md.push_str("## Constraints by Todo\n\n");
    for todo in &todos {
        if let Some(spec) = &todo.spec {
            if let Some(constraints) = spec.get("constraints").and_then(|c| c.as_array()) {
                if !constraints.is_empty() {
                    md.push_str(&format!("### {} ({})\n\n", todo.todo_id, todo.role));
                    for c in constraints {
                        if let Some(s) = c.as_str() {
                            md.push_str(&format!("- {}\n", s));
                        }
                    }
                    md.push('\n');
                }
            }
        }
    }

    Some(md)
}

// ── Workspace Writer ───────────────────────────────────────────────

/// Write CLAUDE.md + .claude/ + .weaver/ into the weaver/ workspace folder.
/// This is the single entry point for generating all context files.
pub fn write_workspace_context(
    cache: &MissionStateCache,
    mission_id: &str,
    weaver_path: &Path,
    repo_ids: &[String],
    plan_json: Option<&serde_json::Value>,
) -> Result<bool, String> {
    // Render CLAUDE.md
    let claude_md = match render_claude_md(cache, mission_id, repo_ids) {
        Some(content) => content,
        None => {
            crate::debug_log::log_info(&format!(
                "[Context] Plan not cached for {}, skipping weaver/ generation",
                mission_id
            ));
            return Ok(false);
        }
    };

    // Create directory structure
    let claude_dir = weaver_path.join(".claude");
    let agents_dir = claude_dir.join("agents");
    let skills_dir = claude_dir.join("skills").join("phase-workflow");
    let rules_dir = claude_dir.join("rules");
    let weaver_data = weaver_path.join(".weaver");
    let specs_dir = weaver_data.join("specs");

    for dir in [&agents_dir, &skills_dir, &rules_dir, &specs_dir] {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create {}: {}", dir.display(), e))?;
    }

    // 1. CLAUDE.md
    std::fs::write(weaver_path.join("CLAUDE.md"), &claude_md)
        .map_err(|e| format!("CLAUDE.md: {}", e))?;

    // 2. .claude/settings.json
    if let Some(settings) = render_settings_json(cache, mission_id) {
        std::fs::write(claude_dir.join("settings.json"), settings)
            .map_err(|e| format!("settings.json: {}", e))?;
    }

    // 3. .claude/agents/*.md
    if let Some(plan) = cache.get_plan(mission_id) {
        for role in &plan.roles {
            if let Some(agent_md) = render_agent_md(role) {
                let name = role
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("agent");
                std::fs::write(agents_dir.join(format!("{}.md", name)), agent_md)
                    .map_err(|e| format!("agent {}: {}", name, e))?;
            }
        }
    }

    // 4. .claude/skills/phase-workflow/SKILL.md
    if let Some(skill) = render_skill_md(cache, mission_id) {
        std::fs::write(skills_dir.join("SKILL.md"), skill)
            .map_err(|e| format!("SKILL.md: {}", e))?;
    }

    // 5. .claude/rules/phase-constraints.md
    if let Some(rules) = render_phase_constraints_md(cache, mission_id) {
        std::fs::write(rules_dir.join("phase-constraints.md"), rules)
            .map_err(|e| format!("phase-constraints.md: {}", e))?;
    }

    // 6. .weaver/mission.json
    if let Some(plan) = plan_json {
        let json_str = serde_json::to_string_pretty(plan)
            .map_err(|e| format!("mission.json serialize: {}", e))?;
        std::fs::write(weaver_data.join("mission.json"), json_str)
            .map_err(|e| format!("mission.json: {}", e))?;
    }

    // 7. .weaver/specs/*.md
    let phases = cache.get_phases_for_mission(mission_id);
    let mut spec_count = 0u32;
    for phase in &phases {
        for todo in &cache.get_todos_for_phase(mission_id, &phase.phase_id) {
            std::fs::write(
                specs_dir.join(format!("{}.md", todo.todo_id)),
                render_todo_spec_md(todo),
            )
            .map_err(|e| format!("spec {}: {}", todo.todo_id, e))?;
            spec_count += 1;
        }
    }

    crate::debug_log::log_info(&format!(
        "[Context] Wrote weaver/ ({} lines CLAUDE.md, {} agents, {} specs) to {}",
        claude_md.lines().count(),
        cache
            .get_plan(mission_id)
            .map(|p| p.roles.len())
            .unwrap_or(0),
        spec_count,
        weaver_path.display()
    ));

    Ok(true)
}

// ── Helpers ────────────────────────────────────────────────────────

fn render_string_array(md: &mut String, heading: &str, value: Option<&serde_json::Value>) {
    if let Some(arr) = value.and_then(|v| v.as_array()) {
        if !arr.is_empty() {
            md.push_str(&format!("{}\n\n", heading));
            for item in arr {
                if let Some(s) = item.as_str() {
                    md.push_str(&format!("- {}\n", s));
                }
            }
            md.push('\n');
        }
    }
}

fn extract_objectives(ctx: &str) -> String {
    let mut result = String::new();
    let mut in_section = false;

    for line in ctx.lines() {
        if line.starts_with("###")
            && (line.contains("Objective") || line.contains("Acceptance"))
        {
            in_section = true;
            result.push_str(line);
            result.push('\n');
            continue;
        }
        if in_section
            && line.starts_with("### ")
            && !line.contains("Objective")
            && !line.contains("Acceptance")
        {
            in_section = false;
            continue;
        }
        if in_section {
            result.push_str(line);
            result.push('\n');
        }
    }

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
