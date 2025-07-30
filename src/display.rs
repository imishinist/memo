use crate::memo::MemoFile;
use serde_yaml::Value;

/// メモの表示形式を統一するための構造体
pub struct MemoDisplayFormatter;

impl MemoDisplayFormatter {
    /// メモを標準形式で表示
    pub fn display_memo(memo: &MemoFile, show_score: Option<f64>) {
        println!("ID: {}", memo.id);
        println!("Modified: {}", memo.modified.format("%Y-%m-%d %H:%M:%S"));

        // スコアがある場合は表示（検索結果用）
        if let Some(score) = show_score {
            println!("Score: {:.2}", score);
        }

        // メタデータエラーがある場合は表示
        if let Some(error) = &memo.frontmatter_error {
            println!("Metadata Error: {}", error);
        } else if let Some(metadata) = &memo.frontmatter {
            if !metadata.is_empty() {
                println!("Metadata:");
                for (key, value) in metadata {
                    println!("  {}: {}", key, Self::format_yaml_value(value));
                }
            }
        }

        // プレビューを表示
        let preview = memo.preview(100);
        if !preview.is_empty() {
            println!("Preview: {}", preview);
        }
        println!("---");
    }

    /// メモのリストを表示
    pub fn display_memo_list(memos: &[MemoFile], title: &str, show_scores: Option<&[f64]>) {
        if memos.is_empty() {
            println!("No memos found. Use 'memo add' to create your first memo.");
            return;
        }

        println!("{}:", title);
        println!();

        for (i, memo) in memos.iter().enumerate() {
            let score = show_scores.and_then(|scores| scores.get(i).copied());
            Self::display_memo(memo, score);
        }
    }

    /// YAML値を文字列形式に変換
    fn format_yaml_value(value: &Value) -> String {
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
