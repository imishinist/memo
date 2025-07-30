use crate::context::MemoContext;
use crate::error::MemoResult;
use crate::repository::MemoRepository;
use chrono::{DateTime, Local};
use serde::{Serialize, Serializer};
use serde_yaml::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct MemoListItem {
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

pub fn run(context: &MemoContext, json_output: bool) -> MemoResult<()> {
    let repo = MemoRepository::new(context.clone());
    let memos = repo.list_recent_memos(20)?;

    if memos.is_empty() {
        if !json_output {
            println!("No memos found. Use 'memo add' to create your first memo.");
        }
        return Ok(());
    }

    if json_output {
        for memo in &memos {
            let list_item = MemoListItem {
                id: memo.id.clone(),
                modified: get_modified_time(&memo.path)?,
                preview: memo.preview(100),
                metadata: memo.frontmatter.clone(),
                metadata_error: memo.frontmatter_error.clone(),
            };

            if let Ok(json) = serde_json::to_string(&list_item) {
                println!("{}", json);
            }
        }
    } else {
        println!("Recent memos:");
        println!();

        for memo in &memos {
            println!("ID: {}", memo.id);

            let modified = get_modified_time(&memo.path)?;
            println!("Modified: {}", modified.format("%Y-%m-%d %H:%M:%S"));

            // Display metadata information if available
            if let Some(error) = &memo.frontmatter_error {
                println!("Metadata Error: {}", error);
            } else if let Some(metadata) = &memo.frontmatter {
                if !metadata.is_empty() {
                    println!("Metadata:");
                    for (key, value) in metadata {
                        println!("  {}: {}", key, format_yaml_value(value));
                    }
                }
            }

            let preview = memo.preview(100);
            if !preview.is_empty() {
                println!("Preview: {}", preview);
            }
            println!("---");
        }

        if memos.len() == 20 {
            let total_count = repo.list_all_memos()?.len();
            if total_count > 20 {
                println!("... and {} more memos", total_count - 20);
            }
        }
    }

    Ok(())
}

fn get_modified_time(path: &std::path::Path) -> MemoResult<DateTime<Local>> {
    let metadata = std::fs::metadata(path)?;
    let modified = metadata.modified()?;
    Ok(DateTime::from(modified))
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
    use crate::context::MemoContext;
    use crate::repository::MemoRepository;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_context() -> (TempDir, MemoContext) {
        let temp_dir = TempDir::new().unwrap();
        let memo_dir = temp_dir.path().join("memo");
        fs::create_dir_all(&memo_dir).unwrap();

        let context = MemoContext {
            memo_dir,
            editor: "echo".to_string(),
        };

        (temp_dir, context)
    }

    #[test]
    fn test_list_empty() {
        let (_temp_dir, context) = create_test_context();
        let result = run(&context, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_with_memos() {
        let (_temp_dir, context) = create_test_context();
        let repo = MemoRepository::new(context.clone());

        // テストメモを作成
        repo.create_memo("2025-01/30/143022.md", "Test memo 1".to_string())
            .unwrap();
        repo.create_memo("2025-01/30/151545.md", "Test memo 2".to_string())
            .unwrap();

        let result = run(&context, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_json_output() {
        let (_temp_dir, context) = create_test_context();
        let repo = MemoRepository::new(context.clone());

        // テストメモを作成
        repo.create_memo("2025-01/30/143022.md", "Test memo".to_string())
            .unwrap();

        let result = run(&context, true);
        assert!(result.is_ok());
    }
}
