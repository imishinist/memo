use crate::context::MemoContext;
use crate::error::{MemoError, MemoResult};
use crate::memo::{MemoDocument, MemoFile};
use crate::utils::id_resolver::resolve_memo_id;
use std::fs;
use std::path::{Path, PathBuf};

/// メモリポジトリを管理する構造体
pub struct MemoRepository {
    context: MemoContext,
}

impl MemoRepository {
    pub fn new(context: MemoContext) -> Self {
        MemoRepository { context }
    }

    /// data_dirからMemoRepositoryを作成
    pub fn from_data_dir(data_dir: PathBuf) -> Self {
        let memo_dir = data_dir.join("memo");
        let context = MemoContext {
            memo_dir,
            editor: std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string()),
        };
        Self::new(context)
    }

    pub fn memo_dir(&self) -> &Path {
        &self.context.memo_dir
    }

    pub fn list_all_memos(&self) -> MemoResult<Vec<MemoFile>> {
        let mut memos = Vec::new();
        self.collect_memos_recursive(&self.context.memo_dir, &mut memos)?;

        // 作成日時でソート（新しい順）
        memos.sort_by(|a, b| b.path.cmp(&a.path));

        Ok(memos)
    }

    /// 検索機能用：全メモをMemoDocumentとして取得
    pub fn list_all_memo_documents(&self) -> MemoResult<Vec<MemoDocument>> {
        let memo_files = self.list_all_memos()?;
        Ok(memo_files
            .iter()
            .map(MemoDocument::from_memo_file)
            .collect())
    }

    pub fn list_recent_memos(&self, limit: usize) -> MemoResult<Vec<MemoFile>> {
        let all_memos = self.list_all_memos()?;
        Ok(all_memos.into_iter().take(limit).collect())
    }

    pub fn find_memo_by_id(&self, id: &str) -> MemoResult<MemoFile> {
        let resolved_path = resolve_memo_id(&self.context.memo_dir, id)
            .map_err(|_| MemoError::MemoNotFound(id.to_string()))?;

        MemoFile::from_path(resolved_path)
    }

    pub fn create_memo(&self, relative_path: &str, content: String) -> MemoResult<MemoFile> {
        let full_path = self.context.memo_dir.join(relative_path);
        MemoFile::create(full_path, content)
    }

    pub fn archive_memo(&self, memo: &MemoFile) -> MemoResult<MemoFile> {
        let archive_dir = self.context.archive_dir();

        let relative_path = memo
            .path
            .strip_prefix(&self.context.memo_dir)
            .map_err(|_| MemoError::ArchiveError("Invalid memo path".to_string()))?;

        let archive_path = archive_dir.join(relative_path);

        if let Some(parent) = archive_path.parent() {
            fs::create_dir_all(parent)?;
        }

        self.ensure_archive_ignored()?;

        memo.move_to(archive_path)
    }

    pub fn archive_memos(&self, memos: Vec<MemoFile>) -> MemoResult<Vec<MemoFile>> {
        let mut archived = Vec::new();

        for memo in memos {
            let archived_memo = self.archive_memo(&memo)?;
            archived.push(archived_memo);
        }

        Ok(archived)
    }

    pub fn archive_directory(&self, dir_path: &str) -> MemoResult<Vec<MemoFile>> {
        let full_dir_path = self.context.memo_dir.join(dir_path);

        if !full_dir_path.exists() || !full_dir_path.is_dir() {
            return Err(MemoError::MemoNotFound(dir_path.to_string()));
        }

        let mut memos = Vec::new();
        self.collect_memos_recursive(&full_dir_path, &mut memos)?;

        self.archive_memos(memos)
    }

    /// .ignoreファイルに.archiveを追加
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

    /// 再帰的にメモファイルを収集（.archiveディレクトリはスキップ）
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
                    Err(_) => continue, // 無効なファイルはスキップ
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
    fn test_create_memo() {
        let (_temp_dir, context) = create_test_context();
        let repo = MemoRepository::new(context);

        let memo = repo
            .create_memo("2025-01/30/143022.md", "Test content".to_string())
            .unwrap();

        assert_eq!(memo.content, "Test content");
        assert_eq!(memo.id, "2025-01/30/143022");
        assert!(memo.path.exists());
    }

    #[test]
    fn test_list_memos() {
        let (_temp_dir, context) = create_test_context();
        let repo = MemoRepository::new(context);

        // テストメモを作成
        repo.create_memo("2025-01/30/143022.md", "Memo 1".to_string())
            .unwrap();
        repo.create_memo("2025-01/30/151545.md", "Memo 2".to_string())
            .unwrap();

        let memos = repo.list_all_memos().unwrap();
        assert_eq!(memos.len(), 2);
    }

    #[test]
    fn test_find_memo_by_id() {
        let (_temp_dir, context) = create_test_context();
        let repo = MemoRepository::new(context);

        repo.create_memo("2025-01/30/143022.md", "Test memo".to_string())
            .unwrap();

        let memo = repo.find_memo_by_id("2025-01/30/143022").unwrap();
        assert_eq!(memo.content, "Test memo");
    }

    #[test]
    fn test_archive_memo() {
        let (_temp_dir, context) = create_test_context();
        let repo = MemoRepository::new(context);

        let memo = repo
            .create_memo("2025-01/30/143022.md", "Test memo".to_string())
            .unwrap();
        let archived = repo.archive_memo(&memo).unwrap();

        // 元のファイルが存在しないことを確認
        assert!(!memo.path.exists());

        // アーカイブファイルが存在することを確認
        assert!(archived.path.exists());
        assert!(archived.path.to_string_lossy().contains(".archive"));
    }
}
