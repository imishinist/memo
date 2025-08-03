use crate::error::{MemoError, MemoResult};
use crate::memo_id::MemoId;
use std::path::{Path, PathBuf};

/// MemoIDを使ってファイルパスを解決（14桁の完全IDのみサポート）
pub fn resolve_memo_id<P: AsRef<Path>>(memo_dir: P, id: &str) -> MemoResult<PathBuf> {
    let memo_dir = memo_dir.as_ref();

    // 14桁の完全IDのみサポート
    let memo_id = MemoId::from_str(id)?;
    let file_path = memo_id.to_file_path(memo_dir);

    if file_path.exists() {
        Ok(file_path)
    } else {
        Err(MemoError::MemoNotFound(id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_memo_structure() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let memo_dir = temp_dir.path().join("memo");

        // Create test directory structure
        let test_date_dir = memo_dir.join("2025-01/30");
        fs::create_dir_all(&test_date_dir).unwrap();

        // Create test memo files
        fs::write(test_date_dir.join("143022.md"), "Test memo content").unwrap();
        fs::write(test_date_dir.join("151545.md"), "Another memo").unwrap();

        (temp_dir, memo_dir)
    }

    #[test]
    fn test_resolve_full_id() {
        let (_temp_dir, memo_dir) = setup_test_memo_structure();

        let result = resolve_memo_id(&memo_dir, "20250130143022");
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("143022.md"));
    }

    #[test]
    fn test_resolve_nonexistent_id() {
        let (_temp_dir, memo_dir) = setup_test_memo_structure();

        let result = resolve_memo_id(&memo_dir, "20250130999999");
        assert!(result.is_err());

        // MemoNotFoundまたはInvalidIdエラーのいずれかを期待
        match result {
            Err(MemoError::MemoNotFound(id)) => {
                assert_eq!(id, "20250130999999");
            }
            Err(MemoError::InvalidId(_)) => {
                // 無効な時刻（99時99分99秒）なのでInvalidIdエラーも許容
            }
            _ => panic!("Expected MemoNotFound or InvalidId error"),
        }
    }

    #[test]
    fn test_resolve_invalid_id_format() {
        let (_temp_dir, memo_dir) = setup_test_memo_structure();

        // 短縮IDは無効
        let result = resolve_memo_id(&memo_dir, "143022");
        assert!(result.is_err());

        // 無効な形式
        let result = resolve_memo_id(&memo_dir, "invalid");
        assert!(result.is_err());
    }
}
