use chrono::{DateTime, Local};
use serde::{Serialize, Serializer};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::frontmatter::parse_memo_content;
use crate::utils::xdg::get_memo_dir;

#[derive(Debug, Serialize)]
pub struct MemoFile {
    pub id: String,
    #[serde(serialize_with = "serialize_datetime")]
    pub modified: DateTime<Local>,
    pub preview: String,
    pub metadata: Option<HashMap<String, Value>>,
    pub metadata_error: Option<String>,
}

fn serialize_datetime<S>(dt: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&dt.to_rfc3339())
}

pub fn run(json_output: bool) {
    let memo_dir = get_memo_dir();

    if !memo_dir.exists() {
        if json_output {
            return; // No output for JSON when no memos exist
        } else {
            println!("No memos found. Use 'memo add' to create your first memo.");
            return;
        }
    }

    let mut memos = collect_memos(&memo_dir);

    if memos.is_empty() {
        if json_output {
            return; // No output for JSON when no memos exist
        } else {
            println!("No memos found. Use 'memo add' to create your first memo.");
            return;
        }
    }

    // Sort by modification time (newest first)
    memos.sort_by(|a, b| b.modified.cmp(&a.modified));

    if json_output {
        // Output in JSONL format
        for memo in memos.iter().take(20) {
            if let Ok(json) = serde_json::to_string(memo) {
                println!("{}", json);
            }
        }
    } else {
        // Original text output
        println!("Recent memos:");
        println!();

        for memo in memos.iter().take(20) {
            // Show latest 20 memos
            println!("ID: {}", memo.id);
            println!("Modified: {}", memo.modified.format("%Y-%m-%d %H:%M:%S"));

            // Display metadata information if available
            if let Some(error) = &memo.metadata_error {
                println!("Metadata Error: {}", error);
            } else if let Some(metadata) = &memo.metadata {
                if !metadata.is_empty() {
                    println!("Metadata:");
                    for (key, value) in metadata {
                        println!("  {}: {}", key, format_yaml_value(value));
                    }
                }
            }

            if !memo.preview.is_empty() {
                println!("Preview: {}", memo.preview);
            }
            println!("---");
        }

        if memos.len() > 20 {
            println!("... and {} more memos", memos.len() - 20);
        }
    }
}

pub fn collect_memos(memo_dir: &PathBuf) -> Vec<MemoFile> {
    collect_memos_with_memo_dir(memo_dir, memo_dir)
}

pub fn collect_memos_with_memo_dir(memo_dir: &PathBuf, base_memo_dir: &PathBuf) -> Vec<MemoFile> {
    let mut memos = Vec::new();

    if let Ok(year_months) = fs::read_dir(memo_dir) {
        for year_month in year_months.flatten() {
            if year_month
                .file_type()
                .map(|ft| ft.is_dir())
                .unwrap_or(false)
            {
                collect_from_month_dir(&year_month.path(), &mut memos, base_memo_dir);
            }
        }
    }

    memos
}

fn collect_from_month_dir(month_dir: &PathBuf, memos: &mut Vec<MemoFile>, base_memo_dir: &PathBuf) {
    if let Ok(days) = fs::read_dir(month_dir) {
        for day in days.flatten() {
            if day.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                collect_from_day_dir(&day.path(), memos, base_memo_dir);
            }
        }
    }
}

fn collect_from_day_dir(day_dir: &PathBuf, memos: &mut Vec<MemoFile>, base_memo_dir: &PathBuf) {
    if let Ok(files) = fs::read_dir(day_dir) {
        for file in files.flatten() {
            if let Some(file_name) = file.file_name().to_str() {
                if file_name.ends_with(".md") {
                    if let Some(memo) = create_memo_file(&file.path(), base_memo_dir) {
                        memos.push(memo);
                    }
                }
            }
        }
    }
}

fn create_memo_file(file_path: &PathBuf, memo_dir: &PathBuf) -> Option<MemoFile> {
    // Extract ID from path
    let relative_path = file_path.strip_prefix(memo_dir).ok()?;
    let id = relative_path.to_str()?.trim_end_matches(".md").to_string();

    // Get modification time
    let metadata = fs::metadata(file_path).ok()?;
    let modified = DateTime::from(metadata.modified().ok()?);

    // Read file content and parse frontmatter
    let content = fs::read_to_string(file_path).ok()?;
    let parsed = parse_memo_content(&content);

    // Get preview from the main content (not frontmatter)
    let preview = get_preview_from_content(&parsed.content);

    Some(MemoFile {
        id,
        modified,
        preview,
        metadata: parsed.frontmatter,
        metadata_error: parsed.frontmatter_error,
    })
}

fn get_preview_from_content(content: &str) -> String {
    let lines: Vec<&str> = content.lines().take(3).collect();
    let preview = lines.join(" ");
    if preview.chars().count() > 100 {
        let truncated: String = preview.chars().take(97).collect();
        format!("{}...", truncated)
    } else {
        preview
    }
}

fn format_yaml_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Sequence(seq) => {
            let items: Vec<String> = seq.iter().map(format_yaml_value).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Mapping(_) => "[object]".to_string(),
        Value::Null => "null".to_string(),
        _ => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_test_memos() -> (tempfile::TempDir, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let memo_dir = temp_dir.path().join("memo");

        // Create test directory structure
        let test_date_dir = memo_dir.join("2025-01/30");
        fs::create_dir_all(&test_date_dir).unwrap();

        // Create test memo files
        fs::write(
            test_date_dir.join("143022.md"),
            "---\ntitle: Test Memo\ntags: [\"@tag1\", \"@meeting\"]\n---\n\n# Test Memo\n\nThis is a test memo with @tag1 @meeting",
        )
        .unwrap();
        fs::write(
            test_date_dir.join("151545.md"),
            "# Another Memo\n\nAnother test memo with @tag2",
        )
        .unwrap();

        (temp_dir, memo_dir)
    }

    #[test]
    fn test_collect_memos() {
        let (_temp_dir, memo_dir) = setup_test_memos();

        let memos = collect_memos_with_memo_dir(&memo_dir, &memo_dir);
        assert_eq!(memos.len(), 2);

        // Check that memos have correct IDs
        let ids: Vec<&String> = memos.iter().map(|m| &m.id).collect();
        assert!(ids.contains(&&"2025-01/30/143022".to_string()));
        assert!(ids.contains(&&"2025-01/30/151545".to_string()));
    }

    #[test]
    fn test_get_preview_from_content() {
        let content = "# Title\n\nFirst line\nSecond line\nThird line\nFourth line";
        let preview = get_preview_from_content(content);
        // The preview takes first 3 lines: "# Title", "", "First line"
        assert_eq!(preview, "# Title  First line");
    }

    #[test]
    fn test_get_preview_from_content_long() {
        let long_content = "a".repeat(150);
        let preview = get_preview_from_content(&long_content);
        assert!(preview.ends_with("..."));
        assert!(preview.len() <= 100);
    }

    #[test]
    fn test_create_memo_file() {
        let (_temp_dir, memo_dir) = setup_test_memos();

        let test_file = memo_dir.join("2025-01/30/143022.md");

        // Use the test version that accepts memo_dir parameter
        if let Some(memo_file) = create_memo_file(&test_file, &memo_dir) {
            assert_eq!(memo_file.id, "2025-01/30/143022");
            assert!(memo_file.preview.contains("Test Memo"));

            // Check metadata
            assert!(memo_file.metadata.is_some());
            assert!(memo_file.metadata_error.is_none());
            let metadata = memo_file.metadata.unwrap();
            assert_eq!(
                metadata.get("title").unwrap().as_str().unwrap(),
                "Test Memo"
            );
        } else {
            panic!("Failed to create memo file");
        }
    }

    #[test]
    fn test_memo_file_serialization() {
        let (_temp_dir, memo_dir) = setup_test_memos();
        let memos = collect_memos_with_memo_dir(&memo_dir, &memo_dir);

        assert!(!memos.is_empty());

        // Test JSON serialization
        for memo in &memos {
            let json_result = serde_json::to_string(memo);
            assert!(json_result.is_ok());

            let json = json_result.unwrap();
            assert!(json.contains(&memo.id));
            assert!(json.contains("metadata"));
            assert!(!json.contains("frontmatter")); // Ensure old field name is not present
        }
    }
}
