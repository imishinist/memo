use crate::error::{MemoError, MemoResult};
use crate::frontmatter::parse_memo_content;
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// メモファイルを表現する構造体
#[derive(Debug, Clone)]
pub struct MemoFile {
    pub path: PathBuf,
    pub id: String,
    pub content: String,
    pub frontmatter: Option<HashMap<String, Value>>,
    pub frontmatter_error: Option<String>,
}

impl MemoFile {
    pub fn from_path<P: AsRef<Path>>(path: P) -> MemoResult<Self> {
        let path = path.as_ref().to_path_buf();
        let content = fs::read_to_string(&path)?;
        let parsed = parse_memo_content(&content);

        let id = Self::extract_id_from_path(&path)?;

        Ok(MemoFile {
            path,
            id,
            content: parsed.content,
            frontmatter: parsed.frontmatter,
            frontmatter_error: parsed.frontmatter_error,
        })
    }

    pub fn create<P: AsRef<Path>>(path: P, content: String) -> MemoResult<Self> {
        let path = path.as_ref().to_path_buf();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&path, &content)?;

        let parsed = parse_memo_content(&content);
        let id = Self::extract_id_from_path(&path)?;

        Ok(MemoFile {
            path,
            id,
            content: parsed.content,
            frontmatter: parsed.frontmatter,
            frontmatter_error: parsed.frontmatter_error,
        })
    }

    /// ファイルパスからIDを抽出（YYYY-MM/DD/HHMMSS.md の形式）
    fn extract_id_from_path(path: &Path) -> MemoResult<String> {
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

            let stem = Path::new(filename.as_ref())
                .file_stem()
                .ok_or_else(|| MemoError::InvalidId(filename.to_string()))?
                .to_string_lossy();

            Ok(format!("{}/{}/{}", year_month, day, stem))
        } else {
            Err(MemoError::InvalidId(path.to_string_lossy().to_string()))
        }
    }

    pub fn move_to<P: AsRef<Path>>(&self, new_path: P) -> MemoResult<MemoFile> {
        let new_path = new_path.as_ref().to_path_buf();

        if let Some(parent) = new_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::rename(&self.path, &new_path)?;

        let new_id = Self::extract_id_from_path(&new_path)?;
        Ok(MemoFile {
            path: new_path,
            id: new_id,
            content: self.content.clone(),
            frontmatter: self.frontmatter.clone(),
            frontmatter_error: self.frontmatter_error.clone(),
        })
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
            id: "test".to_string(),
            content: "これは日本語のテストです。長いテキストをテストします。".to_string(),
            frontmatter: None,
            frontmatter_error: None,
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
            id: "test".to_string(),
            content: "短いテスト".to_string(),
            frontmatter: None,
            frontmatter_error: None,
        };

        let preview = memo.preview(10);
        assert_eq!(preview, "短いテスト");
    }
}
