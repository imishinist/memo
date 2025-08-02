use crate::error::MemoResult;
use crate::frontmatter::parse_memo_content;
use crate::memo_id::MemoId;
use chrono::{DateTime, Local};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 検索機能で使用するメモドキュメント構造体
#[derive(Debug, Clone)]
pub struct MemoDocument {
    pub content: String,
    pub path: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub frontmatter: Option<serde_json::Value>,
}

impl MemoDocument {
    /// MemoFileからMemoDocumentに変換
    pub fn from_memo_file(memo_file: &MemoFile) -> Self {
        // ファイルの作成日時を取得（ファイル名から推測）
        let created_at =
            Self::extract_datetime_from_path(&memo_file.path).unwrap_or_else(|| chrono::Utc::now());

        // frontmatterをHashMapからserde_json::Valueに変換
        let frontmatter = memo_file.frontmatter.as_ref().map(|fm| {
            // serde_yaml::Value から serde_json::Value に変換
            yaml_to_json_value(fm)
        });

        Self {
            content: memo_file.content.clone(),
            path: memo_file.path.to_string_lossy().to_string(),
            created_at,
            frontmatter,
        }
    }

    /// ファイルパスから日時を抽出（YYYY-MM/DD/HHMMSS.md の形式）
    fn extract_datetime_from_path(path: &Path) -> Option<chrono::DateTime<chrono::Utc>> {
        let components: Vec<_> = path
            .components()
            .rev()
            .take(3)
            .map(|c| c.as_os_str().to_string_lossy())
            .collect();

        if components.len() >= 3 {
            let filename = &components[0];
            let day = &components[1];
            let year_month = &components[2];

            let stem = Path::new(filename.as_ref()).file_stem()?.to_string_lossy();

            // YYYY-MM/DD/HHMMSS の形式をパース
            let datetime_str = format!("{}/{}/{}", year_month, day, stem);

            // 2025-01/30/143022 -> 2025-01-30 14:30:22
            let parts: Vec<&str> = datetime_str.split('/').collect();
            if parts.len() == 3 {
                let year_month = parts[0];
                let day = parts[1];
                let time = parts[2];

                if time.len() == 6 {
                    let hour = &time[0..2];
                    let minute = &time[2..4];
                    let second = &time[4..6];

                    let full_datetime =
                        format!("{}-{} {}:{}:{}", year_month, day, hour, minute, second);

                    return chrono::NaiveDateTime::parse_from_str(
                        &full_datetime,
                        "%Y-%m-%d %H:%M:%S",
                    )
                    .ok()
                    .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc));
                }
            }
        }

        None
    }
}

/// serde_yaml::Value から serde_json::Value に変換
fn yaml_to_json_value(fm: &HashMap<String, serde_yaml::Value>) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (k, v) in fm {
        map.insert(k.clone(), yaml_value_to_json(v));
    }
    serde_json::Value::Object(map)
}

/// serde_yaml::Value を serde_json::Value に変換
fn yaml_value_to_json(value: &serde_yaml::Value) -> serde_json::Value {
    match value {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(serde_json::Number::from(i))
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            }
        }
        serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            let arr: Vec<serde_json::Value> = seq.iter().map(yaml_value_to_json).collect();
            serde_json::Value::Array(arr)
        }
        serde_yaml::Value::Mapping(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                if let Some(key_str) = k.as_str() {
                    obj.insert(key_str.to_string(), yaml_value_to_json(v));
                }
            }
            serde_json::Value::Object(obj)
        }
        serde_yaml::Value::Tagged(_) => serde_json::Value::Null,
    }
}

/// メモファイルを表現する構造体
#[derive(Debug, Clone)]
pub struct MemoFile {
    pub path: PathBuf,
    pub id: MemoId,
    pub content: String,
    pub frontmatter: Option<HashMap<String, Value>>,
    pub frontmatter_error: Option<String>,
    pub modified: DateTime<Local>,
}

impl MemoFile {
    pub fn from_path<P: AsRef<Path>>(path: P) -> MemoResult<Self> {
        let path = path.as_ref().to_path_buf();
        let content = fs::read_to_string(&path)?;
        let parsed = parse_memo_content(&content);

        let id = MemoId::from_path(&path)?;
        let modified = Self::get_modified_time(&path)?;

        Ok(MemoFile {
            path,
            id,
            content: parsed.content,
            frontmatter: parsed.frontmatter,
            frontmatter_error: parsed.frontmatter_error,
            modified,
        })
    }

    pub fn create<P: AsRef<Path>>(path: P, content: String) -> MemoResult<Self> {
        let path = path.as_ref().to_path_buf();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&path, &content)?;

        let parsed = parse_memo_content(&content);
        let id = MemoId::from_path(&path)?;
        let modified = Self::get_modified_time(&path)?;

        Ok(MemoFile {
            path,
            id,
            content: parsed.content,
            frontmatter: parsed.frontmatter,
            frontmatter_error: parsed.frontmatter_error,
            modified,
        })
    }

    pub fn move_to<P: AsRef<Path>>(&self, new_path: P) -> MemoResult<MemoFile> {
        let new_path = new_path.as_ref().to_path_buf();

        if let Some(parent) = new_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::rename(&self.path, &new_path)?;

        let new_id = MemoId::from_path(&new_path)?;
        let modified = Self::get_modified_time(&new_path)?;
        Ok(MemoFile {
            path: new_path,
            id: new_id,
            content: self.content.clone(),
            frontmatter: self.frontmatter.clone(),
            frontmatter_error: self.frontmatter_error.clone(),
            modified,
        })
    }

    /// ファイルの更新日時を取得
    fn get_modified_time(path: &Path) -> MemoResult<DateTime<Local>> {
        let metadata = fs::metadata(path)?;
        let modified = metadata.modified()?;
        Ok(DateTime::from(modified))
    }

    pub fn preview(&self, max_length: usize) -> String {
        let content = self.content.trim();
        if content.chars().count() <= max_length {
            content.to_string()
        } else {
            let truncated: String = content.chars().take(max_length).collect();
            format!("{}...", truncated)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_with_japanese_text() {
        let memo = MemoFile {
            path: std::path::PathBuf::from("test.md"),
            id: MemoId::from_str("20250130143022").unwrap(),
            content: "これは日本語のテストです。長いテキストをテストします。".to_string(),
            frontmatter: None,
            frontmatter_error: None,
            modified: chrono::Local::now(),
        };

        let preview = memo.preview(10);
        assert_eq!(preview, "これは日本語のテスト...");

        // 文字数が正確にカウントされることを確認
        let preview_chars: Vec<char> = preview.chars().collect();
        let expected_chars = 10 + 3; // 10文字 + "..."
        assert_eq!(preview_chars.len(), expected_chars);
    }

    #[test]
    fn test_preview_short_japanese_text() {
        let memo = MemoFile {
            path: std::path::PathBuf::from("test.md"),
            id: MemoId::from_str("20250130143022").unwrap(),
            content: "短いテスト".to_string(),
            frontmatter: None,
            frontmatter_error: None,
            modified: chrono::Local::now(),
        };

        let preview = memo.preview(10);
        assert_eq!(preview, "短いテスト");
    }
}
