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
        if content.len() <= max_length {
            content.to_string()
        } else {
            format!("{}...", &content[..max_length])
        }
    }
}
