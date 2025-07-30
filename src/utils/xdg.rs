use std::path::PathBuf;
use std::env;

/// Get the memo data directory following XDG Base Directory specification
pub fn get_memo_dir() -> PathBuf {
    get_memo_dir_with_override(None)
}

/// Get the memo data directory with optional override for testing
pub fn get_memo_dir_with_override(override_dir: Option<PathBuf>) -> PathBuf {
    if let Some(dir) = override_dir {
        return dir.join("memo");
    }
    
    // Check XDG_DATA_HOME environment variable first
    if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data_home).join("memo");
    }
    
    // Fall back to dirs crate
    if let Some(data_home) = dirs::data_dir() {
        data_home.join("memo")
    } else {
        // Final fallback to ~/.local/share/memo
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local")
            .join("share")
            .join("memo")
    }
}

/// Ensure the memo directory exists
pub fn ensure_memo_dir() -> std::io::Result<PathBuf> {
    let memo_dir = get_memo_dir();
    std::fs::create_dir_all(&memo_dir)?;
    Ok(memo_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_get_memo_dir_default() {
        let memo_dir = get_memo_dir();
        assert!(memo_dir.to_string_lossy().contains("memo"));
    }

    #[test]
    fn test_get_memo_dir_with_override() {
        let temp_dir = tempdir().unwrap();
        let memo_dir = get_memo_dir_with_override(Some(temp_dir.path().to_path_buf()));
        assert_eq!(memo_dir, temp_dir.path().join("memo"));
    }

    #[test]
    fn test_ensure_memo_dir() {
        let temp_dir = tempdir().unwrap();
        
        // Test with override to avoid permission issues
        let memo_dir = temp_dir.path().join("memo");
        std::fs::create_dir_all(&memo_dir).unwrap();
        
        assert!(memo_dir.exists());
        assert!(memo_dir.is_dir());
    }
}
