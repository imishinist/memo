use crate::context::MemoContext;
use crate::display::MemoDisplayFormatter;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
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

    if json_output {
        if memos.is_empty() {
            return Ok(());
        }

        for memo in &memos {
            let list_item = MemoListItem {
                id: memo.id.as_str(),
                modified: memo.modified,
                preview: memo.preview(100),
                content: Some(memo.content.clone()), // JSON出力時は全文を含める
                metadata: memo.frontmatter.clone(),
                metadata_error: memo.frontmatter_error.clone(),
            };

            if let Ok(json) = serde_json::to_string(&list_item) {
                println!("{}", json);
            }
        }
    } else {
        // 共通の表示機能を使用
        MemoDisplayFormatter::display_memo_list(&memos, "Recent memos", None);

        if memos.len() == 20 {
            let total_count = repo.list_all_memos()?.len();
            if total_count > 20 {
                println!("... and {} more memos", total_count - 20);
            }
        }
    }

    Ok(())
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
