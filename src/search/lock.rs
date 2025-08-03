use crate::error::MemoError;
use fs2::FileExt;
use std::fs::File;
use std::path::{Path, PathBuf};

/// IndexLock manages a file-based lock for the search index directory.
pub struct IndexLock {
    _file: File,
    lock_path: PathBuf,
}

impl IndexLock {
    pub fn acquire<P: AsRef<Path>>(index_dir: P) -> Result<Self, MemoError> {
        let lock_path = index_dir.as_ref().join("lock");
        let file = File::create(&lock_path).map_err(|e| MemoError::Io(e))?;
        file.lock_exclusive().map_err(|e| MemoError::Io(e))?;

        Ok(Self {
            _file: file,
            lock_path,
        })
    }
}

impl Drop for IndexLock {
    fn drop(&mut self) {
        // delete the lock file when the lock is dropped
        let _ = std::fs::remove_file(&self.lock_path);
    }
}
