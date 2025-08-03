use crate::memo::MemoFile;
use serde_yaml::Value;

pub struct MemoDisplayFormatter;

impl MemoDisplayFormatter {
    pub fn display_memo(memo: &MemoFile) {
        println!("id: {}", memo.id);
        println!("modified: {}", memo.modified.format("%Y-%m-%d %H:%M:%S"));

        // メタデータエラーがある場合は表示
        if let Some(error) = &memo.metadata_error {
            println!("metadata error: {}", error);
        } else if let Some(metadata) = &memo.metadata {
            if !metadata.is_empty() {
                println!("metadata:");
                for (key, value) in metadata {
                    println!("  {}: {}", key, Self::format_yaml_value(value));
                }
            }
        }

        let preview = memo.preview(100);
        if !preview.is_empty() {
            println!("preview: {}", preview);
        }
        println!("---");
    }

    pub fn display_memo_list(memos: &[MemoFile], title: &str) {
        if memos.is_empty() {
            println!("No memos found. Use 'memo add' to create your first memo.");
            return;
        }

        println!("{}:", title);
        println!();
        for memo in memos.iter() {
            Self::display_memo(memo);
        }
    }

    fn format_yaml_value(value: &Value) -> String {
        // NOTE: object not supported in YAML, so we return a placeholder
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Sequence(seq) => {
                let items: Vec<String> = seq.iter().map(Self::format_yaml_value).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Mapping(_) => "[object]".to_string(),
            Value::Null => "null".to_string(),
            _ => "unknown".to_string(),
        }
    }
}
