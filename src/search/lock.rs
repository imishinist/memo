use crate::error::MemoError;
use fs2::FileExt;
use std::fs::File;
use std::path::{Path, PathBuf};

/// インデックスディレクトリのロック管理
pub struct IndexLock {
    _file: File,
    lock_path: PathBuf,
}

impl IndexLock {
    /// ロックを取得
    pub fn acquire<P: AsRef<Path>>(index_dir: P) -> Result<Self, MemoError> {
        let lock_path = index_dir.as_ref().join("lock");
        
        // ロックファイルを作成または開く
        let file = File::create(&lock_path)
            .map_err(|e| MemoError::Io(e))?;

        // 排他ロックを取得（ブロッキング）
        file.lock_exclusive()
            .map_err(|e| MemoError::Io(e))?;

        Ok(Self {
            _file: file,
            lock_path,
        })
    }

    /// ロックを試行（非ブロッキング）
    pub fn try_acquire<P: AsRef<Path>>(index_dir: P) -> Result<Option<Self>, MemoError> {
        let lock_path = index_dir.as_ref().join("lock");
        
        let file = File::create(&lock_path)
            .map_err(|e| MemoError::Io(e))?;

        match file.try_lock_exclusive() {
            Ok(()) => Ok(Some(Self {
                _file: file,
                lock_path,
            })),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(MemoError::Io(e)),
        }
    }
}

impl Drop for IndexLock {
    fn drop(&mut self) {
        // ロックファイルを削除（ベストエフォート）
        let _ = std::fs::remove_file(&self.lock_path);
    }
}
