pub mod index;
pub mod japanese_tokenizer;
pub mod lock;

pub use index::SearchIndex;
pub use lock::IndexLock;

use crate::error::MemoError;
use crate::memo::MemoDocument;
use std::path::PathBuf;

/// 検索結果
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub memo: MemoDocument,

    #[allow(dead_code)]
    pub score: f32,
}

/// 検索機能の統合 interface
pub struct SearchManager {
    data_dir: PathBuf,
    index_base_dir: PathBuf,
}

impl SearchManager {
    pub fn new(data_dir: PathBuf, index_base_dir: PathBuf) -> Self {
        Self {
            data_dir,
            index_base_dir,
        }
    }

    fn get_version_file(&self) -> PathBuf {
        self.index_base_dir.join("version")
    }

    fn get_version(&self) -> Result<Option<String>, MemoError> {
        let version_file = self.get_version_file();
        if !version_file.exists() {
            return Ok(None);
        }

        let version = std::fs::read_to_string(&version_file)
            .map_err(|e| MemoError::Io(e))
            .map(|s| s.trim().to_string())?;
        Ok(Some(version))
    }

    pub fn get_current_index(&self) -> Result<Option<SearchIndex>, MemoError> {
        let version = match self.get_version()? {
            Some(v) => v,
            None => return Ok(None), // No index version found
        };
        let index_dir = self.index_base_dir.join(&version);
        if !index_dir.exists() {
            return Ok(None);
        }
        Ok(Some(SearchIndex::open(&self.data_dir, &index_dir)?))
    }

    pub fn create_new_index(&self) -> Result<SearchIndex, MemoError> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let index_dir = self.index_base_dir.join(&timestamp);

        std::fs::create_dir_all(&index_dir).map_err(|e| MemoError::Io(e))?;
        let index = SearchIndex::create(self.data_dir.clone(), index_dir)?;

        // update version file
        let version_file = self.get_version_file();
        std::fs::write(&version_file, &timestamp).map_err(|e| MemoError::Io(e))?;

        Ok(index)
    }

    pub fn add_memo(&self, memo: &MemoDocument) -> Result<(), MemoError> {
        let mut index = {
            match self.get_current_index()? {
                Some(index) => index,
                None => {
                    // If no index exists, create a new one
                    self.create_new_index()?
                }
            }
        };

        let _lock = IndexLock::acquire(&index.index_dir)?;
        index.add_memo(memo)?;
        index.commit()?;
        Ok(())
    }

    pub fn remove_memo(&self, memo: &MemoDocument) -> Result<(), MemoError> {
        if let Some(mut index) = self.get_current_index()? {
            let _lock = IndexLock::acquire(&index.index_dir)?;
            index.remove_memo(memo)?;
            index.commit()?;
        }
        Ok(())
    }

    /// 検索実行
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>, MemoError> {
        if let Some(index) = self.get_current_index()? {
            index.search(query)
        } else {
            Ok(vec![])
        }
    }
}
