---
name: phase-workflow
description: Execute a mission phase using Weaver orchestration. Triggered when a phase assignment arrives through the weaver channel.
---

# Phase Workflow

When you receive a phase assignment through the weaver channel:

1. Read CLAUDE.md for mission context (objectives, acceptance criteria, plan overview)
2. Read the todo specs in `.weaver/specs/` for each todo listed in the assignment
3. Execute todos sequentially, respecting `blocked_by` dependencies
4. For each todo:
   - Read `.weaver/specs/{todo_id}.md` for the full spec
   - Follow the behavior steps exactly
   - Respect all constraints
   - Handle edge cases
   - Work within the listed file paths (in sibling repo worktrees: `../repo-name/`)
   - Call `weaver_complete_todo` when done
5. After all todos complete, call `weaver_phase_complete`

## Verification

Before marking a todo complete, run the verification tools listed in the phase config:
- Check for lint errors
- Run relevant tests
- Verify the acceptance criteria

## Communication

- Use `weaver_reply` with type "progress" for status updates during long tasks
- Use `weaver_reply` with type "help" if you're blocked and need human input
- Use `weaver_complete_todo` after each todo
- Use `weaver_phase_complete` after all todos in the phase

## File Paths

Code lives in sibling repo worktrees, not in the weaver/ folder:
- `../contexthub-brain/` -- backend API code
- `../contexthub-obsidian/` -- Obsidian plugin code
- Check CLAUDE.md for the full list of repositories
