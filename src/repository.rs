use crate::context::MemoContext;
use crate::error::{MemoError, MemoResult};
use crate::memo::{MemoDocument, MemoFile};
use crate::utils::id_resolver::resolve_memo_id;
use std::fs;
use std::path::Path;

/// MemoRepository is responsible for managing memo files.
pub struct MemoRepository {
    context: MemoContext,
}

impl MemoRepository {
    pub fn new(context: MemoContext) -> Self {
        MemoRepository { context }
    }

    pub fn memo_dir(&self) -> &Path {
        &self.context.memo_dir
    }

    pub fn list_all_memos(&self) -> MemoResult<Vec<MemoFile>> {
        let mut memos = Vec::new();
        self.collect_memos_recursive(&self.context.memo_dir, &mut memos)?;

        // sort by modified time in descending order
        memos.sort_by(|a, b| b.path.cmp(&a.path));

        Ok(memos)
    }

    pub fn list_all_memo_documents(&self) -> MemoResult<Vec<MemoDocument>> {
        let memo_files = self.list_all_memos()?;
        Ok(memo_files
            .iter()
            .map(MemoDocument::from_memo_file)
            .collect())
    }

    pub fn find_memo_by_id(&self, id: &str) -> MemoResult<MemoFile> {
        let resolved_path = resolve_memo_id(&self.context.memo_dir, id)
            .map_err(|_| MemoError::MemoNotFound(id.to_string()))?;

        MemoFile::from_path(resolved_path)
    }

    pub fn create_memo<P: AsRef<Path>>(
        &self,
        relative_path: P,
        content: String,
    ) -> MemoResult<MemoFile> {
        let full_path = self.context.memo_dir.join(relative_path);
        MemoFile::create(full_path, content)
    }

    // archive a single memo file
    pub fn archive_memo(&self, memo: &MemoFile) -> MemoResult<MemoFile> {
        let archive_dir = self.context.archive_dir();
        let relative_path = memo.id.to_relative_path();
        let archive_path = archive_dir.join(relative_path);

        if let Some(parent) = archive_path.parent() {
            fs::create_dir_all(parent)?;
        }
        self.ensure_archive_ignored()?;

        memo.move_to(archive_path)
    }

    // archive multiple memo files
    pub fn archive_memos(&self, memos: Vec<MemoFile>) -> MemoResult<Vec<MemoFile>> {
        let mut archived = Vec::new();

        for memo in memos {
            let archived_memo = self.archive_memo(&memo)?;
            archived.push(archived_memo);
        }

        Ok(archived)
    }

    /// archive all memo files in a directory
    pub fn archive_directory(&self, dir_path: &str) -> MemoResult<Vec<MemoFile>> {
        let full_dir_path = self.context.memo_dir.join(dir_path);

        if !full_dir_path.exists() || !full_dir_path.is_dir() {
            return Err(MemoError::MemoNotFound(dir_path.to_string()));
        }

        let mut memos = Vec::new();
        self.collect_memos_recursive(&full_dir_path, &mut memos)?;

        self.archive_memos(memos)
    }

    /// add ".archive" to .ignore file
    fn ensure_archive_ignored(&self) -> MemoResult<()> {
        let ignore_file = self.context.ignore_file();

        let mut content = if ignore_file.exists() {
            fs::read_to_string(&ignore_file)?
        } else {
            String::new()
        };

        if !content.contains(".archive") {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(".archive\n");
            fs::write(&ignore_file, content)?;
        }

        Ok(())
    }

    /// connects to the memo directory and recursively collects all memo files
    fn collect_memos_recursive(&self, dir: &Path, memos: &mut Vec<MemoFile>) -> MemoResult<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if path.file_name().and_then(|n| n.to_str()) == Some(".archive") {
                    continue;
                }
                self.collect_memos_recursive(&path, memos)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                match MemoFile::from_path(&path) {
                    Ok(memo) => memos.push(memo),
                    Err(_) => continue,
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_context() -> (TempDir, MemoContext) {
        let temp_dir = TempDir::new().unwrap();
        let memo_dir = temp_dir.path().join("memo");
        fs::create_dir_all(&memo_dir).unwrap();

        let context = MemoContext {
            memo_dir,
            editor: "echo".to_string(),
        };

        (temp_dir, context)
    }

    #[test]
    fn test_memo_repository_dir() {
        let (_temp_dir, context) = create_test_context();
        let repo = MemoRepository::new(context.clone());

        assert_eq!(repo.memo_dir(), context.memo_dir);
        assert!(repo.memo_dir().exists());
    }

    #[test]
    fn test_create_memo() {
        let (_temp_dir, context) = create_test_context();
        let repo = MemoRepository::new(context);

        let memo = repo
            .create_memo("2025-01/30/143022.md", "Test content".to_string())
            .unwrap();

        assert_eq!(memo.content, "Test content");
        assert_eq!(memo.id.as_str(), "20250130143022");
        assert!(memo.path.exists());
    }

    #[test]
    fn test_list_memos() {
        let (_temp_dir, context) = create_test_context();
        let repo = MemoRepository::new(context);

        repo.create_memo("2025-01/30/143022.md", "Memo 1".to_string())
            .unwrap();
        repo.create_memo("2025-01/30/151545.md", "Memo 2".to_string())
            .unwrap();

        let memos = repo.list_all_memos().unwrap();
        assert_eq!(memos.len(), 2);
        assert_eq!(memos[0].content, "Memo 2");
        assert_eq!(memos[1].content, "Memo 1");
    }

    #[test]
    fn test_find_memo_by_id() {
        let (_temp_dir, context) = create_test_context();
        let repo = MemoRepository::new(context);

        repo.create_memo("2025-01/30/143022.md", "Test memo".to_string())
            .unwrap();

        let memo = repo.find_memo_by_id("20250130143022").unwrap();
        assert_eq!(memo.content, "Test memo");
        matches!(
            repo.find_memo_by_id("invalid_id").unwrap_err(),
            MemoError::MemoNotFound(_)
        );
    }

    #[test]
    fn test_archive_memo() {
        let (_temp_dir, context) = create_test_context();
        let repo = MemoRepository::new(context);

        let memo = repo
            .create_memo("2025-01/30/143022.md", "Test memo".to_string())
            .unwrap();
        let archived = repo.archive_memo(&memo).unwrap();

        assert!(!memo.path.exists());
        assert!(archived.path.exists());
        assert!(archived.path.to_string_lossy().contains(".archive"));
    }

    #[test]
    fn test_archive_memos() {
        let (_temp_dir, context) = create_test_context();
        let repo = MemoRepository::new(context);

        let memo1 = repo
            .create_memo("2025-01/30/143022.md", "Memo 1".to_string())
            .unwrap();
        let memo2 = repo
            .create_memo("2025-01/30/151545.md", "Memo 2".to_string())
            .unwrap();

        let archived = repo.archive_memos(vec![memo1, memo2]).unwrap();

        assert_eq!(archived.len(), 2);
        for memo in archived {
            assert!(memo.path.exists());
            assert!(memo.path.to_string_lossy().contains(".archive"));
        }
    }

    #[test]
    fn test_archive_directory() {
        let (_temp_dir, context) = create_test_context();
        let repo = MemoRepository::new(context);

        repo.create_memo("2025-01/30/143022.md", "Memo 1".to_string())
            .unwrap();
        repo.create_memo("2025-01/30/151545.md", "Memo 2".to_string())
            .unwrap();
        repo.create_memo("2025-02/01/151545.md", "Memo 3".to_string())
            .unwrap();

        let archived = repo.archive_directory("2025-01/30").unwrap();
        assert_eq!(archived.len(), 2);
        for memo in archived {
            assert!(memo.path.exists());
            assert!(memo.path.to_string_lossy().contains(".archive"));
        }
        assert!(repo.memo_dir().join("2025-01/30").exists());
        assert!(!repo.memo_dir().join("2025-01/30/143022.md").exists());
        assert!(!repo.memo_dir().join("2025-01/30/151545.md").exists());
        assert!(repo.memo_dir().join("2025-02/01/151545.md").exists());
        assert!(repo.memo_dir().join("2025-02/01").exists());
    }
}
