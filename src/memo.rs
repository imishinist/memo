use crate::error::MemoResult;
use crate::front_matter;
use crate::memo_id::MemoId;

use chrono::{DateTime, Local, Utc};

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// MemoDocument is used for search functionality and represents a memo document with its content,
/// path, creation date, and front matter.
#[derive(Debug, Clone)]
pub struct MemoDocument {
    pub id: MemoId,
    pub content: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

impl MemoDocument {
    pub fn from_memo_file(memo_file: &MemoFile) -> Self {
        let created_at = memo_file.id.get_datetime().to_utc();
        let metadata = memo_file.metadata.as_ref().map(yaml_to_json_value);

        Self {
            id: memo_file.id.clone(),
            content: memo_file.content.clone(),
            path: memo_file.path.to_string_lossy().to_string(),
            created_at,
            metadata,
        }
    }
}

/// Convert serde_yaml::Value to serde_json::Value
fn yaml_to_json_value(fm: &HashMap<String, serde_yaml::Value>) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (k, v) in fm {
        map.insert(k.clone(), yaml_value_to_json(v));
    }
    serde_json::Value::Object(map)
}

/// Convert serde_yaml::Value to serde_json::Value
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

/// MemoFile represents a memo file with its path, content, front matter, and metadata.
#[derive(Debug, Clone)]
pub struct MemoFile {
    pub id: MemoId,
    pub path: PathBuf,

    pub content: String,
    pub metadata: Option<HashMap<String, serde_yaml::Value>>,
    pub metadata_error: Option<String>,

    pub modified: DateTime<Local>,
}

impl MemoFile {
    pub fn from_path<P: AsRef<Path>>(path: P) -> MemoResult<Self> {
        let path = path.as_ref().to_path_buf();

        let id = MemoId::from_path(&path)?;

        let content = fs::read_to_string(&path)?;
        let parsed = front_matter::parse_memo_content(&content);

        let modified = Self::get_modified_time(&path)?;
        Ok(MemoFile {
            id,
            path,
            content: parsed.content,
            metadata: parsed.front_matter,
            metadata_error: parsed.front_matter_error,
            modified,
        })
    }

    pub fn create<P: AsRef<Path>>(path: P, content: String) -> MemoResult<Self> {
        let path = path.as_ref().to_path_buf();
        let id = MemoId::from_path(&path)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, &content)?;

        let parsed = front_matter::parse_memo_content(&content);
        let modified = Self::get_modified_time(&path)?;

        Ok(MemoFile {
            id,
            path,
            content: parsed.content,
            metadata: parsed.front_matter,
            metadata_error: parsed.front_matter_error,
            modified,
        })
    }

    pub fn move_to<P: AsRef<Path>>(&self, new_path: P) -> MemoResult<MemoFile> {
        let new_path = new_path.as_ref().to_path_buf();
        if let Some(parent) = new_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&self.path, &new_path)?;

        let id = self.id.clone();
        let modified = Self::get_modified_time(&new_path)?;
        Ok(MemoFile {
            id,
            path: new_path,
            content: self.content.clone(),
            metadata: self.metadata.clone(),
            metadata_error: self.metadata_error.clone(),
            modified,
        })
    }

    fn get_modified_time(path: &Path) -> MemoResult<DateTime<Local>> {
        let metadata = fs::metadata(path)?;
        let modified = metadata.modified()?;
        Ok(DateTime::from(modified))
    }

    pub fn preview(&self, max_chars: usize) -> String {
        let content = self.content.trim();
        if content.chars().count() <= max_chars {
            return content.to_string();
        }

        let truncated: String = content.chars().take(max_chars).collect();
        format!("{}...", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memo_document_from_memo_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let memo_path = temp_dir.path().join("2025-01/30/143022.md");
        fs::create_dir_all(memo_path.parent().unwrap()).unwrap();
        fs::write(
            &memo_path,
            r#"---
title: "Test Memo"
tags: ["@tag1", "@tag2"]
---
Test content"#,
        )
        .unwrap();

        let memo_file = MemoFile::from_path(&memo_path).unwrap();

        let memo_doc = MemoDocument::from_memo_file(&memo_file);
        assert_eq!(memo_doc.content, "Test content");
        assert_eq!(memo_doc.path, memo_path.to_string_lossy());
        assert_eq!(memo_doc.created_at.timestamp(), 1738215022);
        assert_eq!(
            memo_doc.metadata,
            Some(serde_json::json!({
                "title": "Test Memo",
                "tags": ["@tag1", "@tag2"]
            }))
        );
    }

    #[test]
    fn test_memo_file_from_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let memo_path = temp_dir.path().join("2025-01/30/143022.md");
        fs::create_dir_all(memo_path.parent().unwrap()).unwrap();
        fs::write(&memo_path, "Test content").unwrap();

        let memo_file = MemoFile::from_path(&memo_path).unwrap();
        assert_eq!(memo_file.id.as_str(), "20250130143022");
        assert_eq!(memo_file.content, "Test content");
        assert_eq!(memo_file.path, memo_path);
    }

    #[test]
    fn test_memo_file_create() {
        let temp_dir = tempfile::tempdir().unwrap();
        let memo_path = temp_dir.path().join("2025-01/30/143022.md");

        let memo_file = MemoFile::create(&memo_path, "Test content".to_string()).unwrap();
        assert_eq!(memo_file.id.as_str(), "20250130143022");
        assert_eq!(memo_file.content, "Test content");
        assert_eq!(memo_file.path, memo_path);

        // ファイルが実際に作成されていることを確認
        assert!(memo_path.exists());
    }

    #[test]
    fn test_memo_file_move_to() {
        let temp_dir = tempfile::tempdir().unwrap();
        let old_path = temp_dir.path().join("2025-01/30/143022.md");
        let new_path = temp_dir.path().join(".archive/2025-01/30/143022.md");

        // 旧ファイルを作成
        fs::create_dir_all(old_path.parent().unwrap()).unwrap();
        fs::write(&old_path, "Test content").unwrap();

        let memo_file = MemoFile::from_path(&old_path).unwrap();
        let moved_memo_file = memo_file.move_to(&new_path).unwrap();

        assert_eq!(moved_memo_file.id.as_str(), "20250130143022");
        assert_eq!(moved_memo_file.path, new_path);
        assert!(!old_path.exists());
        assert!(new_path.exists());
    }

    #[test]
    fn test_memo_file_preview() {
        let memo = MemoFile {
            id: MemoId::from_str("20250130143022").unwrap(),
            path: Default::default(),
            content: "a".repeat(200).to_string(),
            metadata: None,
            metadata_error: None,
            modified: Default::default(),
        };
        assert_eq!(memo.preview(100).len(), 103);

        let memo = MemoFile {
            id: MemoId::from_str("20250130143022").unwrap(),
            path: Default::default(),
            content: "あ".repeat(200).to_string(),
            metadata: None,
            metadata_error: None,
            modified: Default::default(),
        };
        assert_eq!(memo.preview(100).chars().count(), 103);
    }
}
