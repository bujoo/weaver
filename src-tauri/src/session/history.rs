use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A raw line from ~/.claude/history.jsonl
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawHistoryLine {
    #[serde(default)]
    display: String,
    #[serde(default)]
    timestamp: u64,
    #[serde(default)]
    project: String,
    #[serde(default)]
    session_id: String,
}

/// A deduplicated, enriched history entry returned to the frontend
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub session_id: String,
    pub display: String,
    pub timestamp: u64,
    pub project: String,
    pub project_name: String,
}

/// Parse JSONL text into deduplicated HistoryEntry vec, sorted newest first.
/// Keeps the entry with the highest timestamp for each sessionId.
pub fn parse_history_jsonl(content: &str) -> Vec<HistoryEntry> {
    let mut by_session: HashMap<String, RawHistoryLine> = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(raw) = serde_json::from_str::<RawHistoryLine>(line) {
            if raw.session_id.is_empty() {
                continue;
            }
            let existing = by_session.get(&raw.session_id);
            if existing.map_or(true, |e| raw.timestamp > e.timestamp) {
                by_session.insert(raw.session_id.clone(), raw);
            }
        }
    }

    let mut entries: Vec<HistoryEntry> = by_session
        .into_values()
        .map(|raw| {
            let project_name = PathBuf::from(&raw.project)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            HistoryEntry {
                session_id: raw.session_id,
                display: raw.display,
                timestamp: raw.timestamp,
                project: raw.project,
                project_name,
            }
        })
        .collect();

    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    entries
}

/// Read ~/.claude/history.jsonl and return deduplicated entries sorted newest first.
pub fn get_history() -> Result<Vec<HistoryEntry>, String> {
    let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    let path = home_dir.join(".claude").join("history.jsonl");

    if !path.exists() {
        return Ok(vec![]);
    }

    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read history.jsonl: {e}"))?;

    Ok(parse_history_jsonl(&content))
}

/// Search all session JSONL files under ~/.claude/projects/ for a query string.
/// Returns session IDs of files containing the query (case-insensitive).
/// Runs file reads concurrently using threads.
pub fn deep_search(query: &str) -> Result<Vec<String>, String> {
    let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    let projects_dir = home_dir.join(".claude").join("projects");

    if !projects_dir.exists() {
        return Ok(vec![]);
    }

    let query_lower = query.to_lowercase();

    // Collect all candidate JSONL file paths
    let mut candidates: Vec<(String, std::path::PathBuf)> = Vec::new();
    if let Ok(project_entries) = std::fs::read_dir(&projects_dir) {
        for project_entry in project_entries.flatten() {
            let project_path = project_entry.path();
            if !project_path.is_dir() {
                continue;
            }
            if let Ok(files) = std::fs::read_dir(&project_path) {
                for file_entry in files.flatten() {
                    let file_path = file_entry.path();
                    if file_path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                        if let Some(stem) = file_path.file_stem().and_then(|s| s.to_str()) {
                            // Skip agent-* sidechains and non-UUID files
                            if !stem.starts_with("agent-") && stem.contains('-') {
                                candidates.push((stem.to_string(), file_path));
                            }
                        }
                    }
                }
            }
        }
    }

    // Search files concurrently using threads
    use std::sync::{Arc, Mutex};
    let matched: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let query_lower = Arc::new(query_lower);

    let handles: Vec<_> = candidates
        .into_iter()
        .map(|(session_id, path)| {
            let matched = Arc::clone(&matched);
            let query_lower = Arc::clone(&query_lower);
            std::thread::spawn(move || {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.to_lowercase().contains(query_lower.as_str()) {
                        let mut guard = matched.lock().unwrap();
                        guard.push(session_id);
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        let _ = handle.join();
    }

    let result = Arc::try_unwrap(matched)
        .map_err(|_| "Arc unwrap failed")?
        .into_inner()
        .map_err(|e| format!("Mutex poisoned: {e}"))?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_history_jsonl_empty() {
        let result = parse_history_jsonl("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_history_jsonl_single_entry() {
        let jsonl = r#"{"display":"Hello world","timestamp":1000,"project":"/Users/you/myproject","sessionId":"abc-123"}"#;
        let result = parse_history_jsonl(jsonl);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].session_id, "abc-123");
        assert_eq!(result[0].display, "Hello world");
        assert_eq!(result[0].project_name, "myproject");
        assert_eq!(result[0].timestamp, 1000);
    }

    #[test]
    fn test_parse_history_jsonl_deduplicates_by_session_id() {
        let jsonl = concat!(
            r#"{"display":"First prompt","timestamp":1000,"project":"/p/proj","sessionId":"abc"}"#,
            "\n",
            r#"{"display":"Second prompt","timestamp":2000,"project":"/p/proj","sessionId":"abc"}"#,
        );
        let result = parse_history_jsonl(jsonl);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].display, "Second prompt");
        assert_eq!(result[0].timestamp, 2000);
    }

    #[test]
    fn test_parse_history_jsonl_sorted_newest_first() {
        let jsonl = concat!(
            r#"{"display":"Old","timestamp":1000,"project":"/p/a","sessionId":"aaa"}"#,
            "\n",
            r#"{"display":"New","timestamp":3000,"project":"/p/b","sessionId":"bbb"}"#,
            "\n",
            r#"{"display":"Mid","timestamp":2000,"project":"/p/c","sessionId":"ccc"}"#,
        );
        let result = parse_history_jsonl(jsonl);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].timestamp, 3000);
        assert_eq!(result[1].timestamp, 2000);
        assert_eq!(result[2].timestamp, 1000);
    }

    #[test]
    fn test_parse_history_jsonl_skips_empty_session_id() {
        let jsonl = concat!(
            r#"{"display":"No session","timestamp":1000,"project":"/p/a","sessionId":""}"#,
            "\n",
            r#"{"display":"Has session","timestamp":2000,"project":"/p/b","sessionId":"valid-id"}"#,
        );
        let result = parse_history_jsonl(jsonl);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].session_id, "valid-id");
    }

    #[test]
    fn test_parse_history_jsonl_skips_malformed_lines() {
        let jsonl = "not json at all\n{\"display\":\"ok\",\"timestamp\":1,\"project\":\"/p\",\"sessionId\":\"s1\"}";
        let result = parse_history_jsonl(jsonl);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_project_name_derived_from_path() {
        let jsonl = r#"{"display":"x","timestamp":1,"project":"/Users/you/Documents/GitHub/c9watch","sessionId":"s1"}"#;
        let result = parse_history_jsonl(jsonl);
        assert_eq!(result[0].project_name, "c9watch");
    }
}
