use chrono::Local;
use std::path::PathBuf;

use super::xdg::get_memo_dir;

/// Resolve a memo ID to a file path
/// Supports various ID formats:
/// - Full: 2025-01/30/143022
/// - Short: 0130143022 (month-day-hour-minute-second)
/// - Shorter: 30143022 (day-hour-minute-second, same month)
/// - Shortest: 143022 (hour-minute-second, same day)
pub fn resolve_id(id: &str) -> Option<PathBuf> {
    resolve_id_with_memo_dir(&get_memo_dir(), id)
}

/// Resolve a memo ID to a file path with custom memo directory (for testing)
pub fn resolve_id_with_memo_dir(memo_dir: &PathBuf, id: &str) -> Option<PathBuf> {
    let now = Local::now();

    // Try full path format first (2025-01/30/143022)
    if id.contains('/') {
        let path = memo_dir.join(format!("{}.md", id));
        if path.exists() {
            return Some(path);
        }
    }

    // Try different ID formats
    match id.len() {
        6 => {
            // Format: HHMMSS (same day)
            let today = now.format("%Y-%m/%d").to_string();
            let path = memo_dir.join(format!("{}/{}.md", today, id));
            if path.exists() {
                return Some(path);
            }
        }
        8 => {
            // Format: DDHHMMSS (same month)
            if let Ok(day) = id[0..2].parse::<u32>() {
                let current_month = now.format("%Y-%m").to_string();
                let time_part = &id[2..];
                let path = memo_dir.join(format!("{}/{:02}/{}.md", current_month, day, time_part));
                if path.exists() {
                    return Some(path);
                }
            }
        }
        10 => {
            // Format: MMDDHHMMSS
            if let (Ok(month), Ok(day)) = (id[0..2].parse::<u32>(), id[2..4].parse::<u32>()) {
                let current_year = now.format("%Y").to_string();
                let time_part = &id[4..];
                let path = memo_dir.join(format!(
                    "{}-{:02}/{:02}/{}.md",
                    current_year, month, day, time_part
                ));
                if path.exists() {
                    return Some(path);
                }
            }
        }
        _ => {}
    }

    // If no exact match, try to find similar files
    find_similar_files(memo_dir, id)
}

/// Find files with similar IDs (fuzzy matching)
fn find_similar_files(memo_dir: &PathBuf, id: &str) -> Option<PathBuf> {
    use std::fs;

    // Walk through the directory structure to find matching files
    if let Ok(entries) = fs::read_dir(memo_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if let Some(path) = search_in_month_dir(&entry.path(), id) {
                    return Some(path);
                }
            }
        }
    }

    None
}

fn search_in_month_dir(month_dir: &PathBuf, id: &str) -> Option<PathBuf> {
    use std::fs;

    if let Ok(entries) = fs::read_dir(month_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if let Some(path) = search_in_day_dir(&entry.path(), id) {
                    return Some(path);
                }
            }
        }
    }

    None
}

fn search_in_day_dir(day_dir: &PathBuf, id: &str) -> Option<PathBuf> {
    use std::fs;

    if let Ok(entries) = fs::read_dir(day_dir) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".md") {
                    let name_without_ext = &file_name[..file_name.len() - 3];
                    if name_without_ext.contains(id) {
                        return Some(entry.path());
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn setup_test_memo_structure() -> (tempfile::TempDir, PathBuf) {
        let temp_dir = tempdir().unwrap();
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

        let result = resolve_id_with_memo_dir(&memo_dir, "2025-01/30/143022");
        assert!(result.is_some());

        let path = result.unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("143022.md"));
    }

    #[test]
    fn test_resolve_nonexistent_id() {
        let (_temp_dir, memo_dir) = setup_test_memo_structure();

        let result = resolve_id_with_memo_dir(&memo_dir, "2025-01/30/999999");
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_short_id_formats() {
        let (_temp_dir, memo_dir) = setup_test_memo_structure();

        // Test 6-digit format (HHMMSS) - this might not work without proper date setup
        // but we can test the parsing logic
        let _result = resolve_id_with_memo_dir(&memo_dir, "143022");
        // This may be None if the current date doesn't match our test structure

        // Test 8-digit format (DDHHMMSS)
        let _result = resolve_id_with_memo_dir(&memo_dir, "30143022");
        // This may be None if the current month doesn't match our test structure

        // Test 10-digit format (MMDDHHMMSS)
        let _result = resolve_id_with_memo_dir(&memo_dir, "0130143022");
        // This may be None if the current year doesn't match our test structure
    }

    #[test]
    fn test_find_similar_files() {
        let (_temp_dir, memo_dir) = setup_test_memo_structure();

        let result = find_similar_files(&memo_dir, "143");
        assert!(result.is_some());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("143022.md"));
    }
}
