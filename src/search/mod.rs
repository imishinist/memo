pub mod index;
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
    pub score: f32,
}

/// 検索機能の統合インターface
pub struct SearchManager {
    data_dir: PathBuf,
}

impl SearchManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// 現在のインデックスを取得
    pub fn get_current_index(&self) -> Result<Option<SearchIndex>, MemoError> {
        let version_file = self.data_dir.join(".indexversion");
        if !version_file.exists() {
            return Ok(None);
        }

        let version = std::fs::read_to_string(&version_file)
            .map_err(|e| MemoError::Io(e))?
            .trim()
            .to_string();

        let index_dir = self.data_dir.join(".index").join(&version);
        if !index_dir.exists() {
            return Ok(None);
        }

        Ok(Some(SearchIndex::open(index_dir)?))
    }

    /// 新しいインデックスを作成
    pub fn create_new_index(&self) -> Result<SearchIndex, MemoError> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let index_dir = self.data_dir.join(".index").join(&timestamp);
        
        std::fs::create_dir_all(&index_dir)
            .map_err(|e| MemoError::Io(e))?;

        let index = SearchIndex::create(index_dir.clone())?;

        // バージョンファイルを更新
        let version_file = self.data_dir.join(".indexversion");
        std::fs::write(&version_file, &timestamp)
            .map_err(|e| MemoError::Io(e))?;

        Ok(index)
    }

    /// インデックスにメモを追加
    pub fn add_memo(&self, memo: &MemoDocument) -> Result<(), MemoError> {
        if let Some(mut index) = self.get_current_index()? {
            let _lock = IndexLock::acquire(&index.index_dir)?;
            index.add_memo(memo)?;
            index.commit()?;
        }
        Ok(())
    }

    /// インデックスからメモを削除
    pub fn remove_memo(&self, path: &str) -> Result<(), MemoError> {
        if let Some(mut index) = self.get_current_index()? {
            let _lock = IndexLock::acquire(&index.index_dir)?;
            index.remove_memo(path)?;
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
