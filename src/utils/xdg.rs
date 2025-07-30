use crate::error::{MemoError, MemoResult};
use std::env;
use std::path::PathBuf;

/// Get the memo data directory following XDG Base Directory specification
pub fn get_memo_dir() -> MemoResult<PathBuf> {
    get_memo_dir_with_override(None)
}

pub fn get_memo_dir_with_override(override_dir: Option<PathBuf>) -> MemoResult<PathBuf> {
    if let Some(dir) = override_dir {
        return Ok(dir.join("memo"));
    }

    if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
        return Ok(PathBuf::from(xdg_data_home).join("memo"));
    }

    if let Some(data_home) = dirs::data_dir() {
        Ok(data_home.join("memo"))
    } else {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            MemoError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine home directory",
            ))
        })?;

        Ok(home_dir.join(".local").join("share").join("memo"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_memo_dir_default() {
        let memo_dir = get_memo_dir().unwrap();
        assert!(memo_dir.to_string_lossy().contains("memo"));
    }

    #[test]
    fn test_get_memo_dir_with_override() {
        let temp_dir = TempDir::new().unwrap();
        let memo_dir = get_memo_dir_with_override(Some(temp_dir.path().to_path_buf())).unwrap();
        assert_eq!(memo_dir, temp_dir.path().join("memo"));
    }
}
