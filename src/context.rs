use crate::error::MemoResult;
use crate::utils::xdg;
use std::path::PathBuf;

/// アプリケーションのコンテキスト情報を管理
#[derive(Debug, Clone)]
pub struct MemoContext {
    pub memo_dir: PathBuf,
    pub editor: String,
}

impl MemoContext {
    pub fn new() -> MemoResult<Self> {
        let memo_dir = xdg::get_memo_dir()?;
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

        Ok(MemoContext { memo_dir, editor })
    }

    pub fn ensure_memo_dir(&self) -> MemoResult<()> {
        if !self.memo_dir.exists() {
            std::fs::create_dir_all(&self.memo_dir)?;
        }
        Ok(())
    }

    pub fn archive_dir(&self) -> PathBuf {
        self.memo_dir.join(".archive")
    }

    pub fn ignore_file(&self) -> PathBuf {
        self.memo_dir.join(".ignore")
    }

    pub fn index_dir(&self) -> PathBuf {
        self.memo_dir.join(".index")
    }
}
