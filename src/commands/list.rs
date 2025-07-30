use chrono::{DateTime, Local};
use std::fs;
use std::path::PathBuf;

use crate::utils::xdg::get_memo_dir;

#[derive(Debug)]
pub struct MemoFile {
    pub id: String,
    pub modified: DateTime<Local>,
    pub preview: String,
}

pub fn run() {
    let memo_dir = get_memo_dir();

    if !memo_dir.exists() {
        println!("No memos found. Use 'memo add' to create your first memo.");
        return;
    }

    let mut memos = collect_memos(&memo_dir);

    if memos.is_empty() {
        println!("No memos found. Use 'memo add' to create your first memo.");
        return;
    }

    // Sort by modification time (newest first)
    memos.sort_by(|a, b| b.modified.cmp(&a.modified));

    println!("Recent memos:");
    println!();

    for memo in memos.iter().take(20) {
        // Show latest 20 memos
        println!("ID: {}", memo.id);
        println!("Modified: {}", memo.modified.format("%Y-%m-%d %H:%M:%S"));
        if !memo.preview.is_empty() {
            println!("Preview: {}", memo.preview);
        }
        println!("---");
    }

    if memos.len() > 20 {
        println!("... and {} more memos", memos.len() - 20);
    }
}

pub fn collect_memos(memo_dir: &PathBuf) -> Vec<MemoFile> {
    collect_memos_with_memo_dir(memo_dir, memo_dir)
}

pub fn collect_memos_with_memo_dir(memo_dir: &PathBuf, base_memo_dir: &PathBuf) -> Vec<MemoFile> {
    let mut memos = Vec::new();

    if let Ok(year_months) = fs::read_dir(memo_dir) {
        for year_month in year_months.flatten() {
            if year_month
                .file_type()
                .map(|ft| ft.is_dir())
                .unwrap_or(false)
            {
                collect_from_month_dir(&year_month.path(), &mut memos, base_memo_dir);
            }
        }
    }

    memos
}

fn collect_from_month_dir(month_dir: &PathBuf, memos: &mut Vec<MemoFile>, base_memo_dir: &PathBuf) {
    if let Ok(days) = fs::read_dir(month_dir) {
        for day in days.flatten() {
            if day.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                collect_from_day_dir(&day.path(), memos, base_memo_dir);
            }
        }
    }
}

fn collect_from_day_dir(day_dir: &PathBuf, memos: &mut Vec<MemoFile>, base_memo_dir: &PathBuf) {
    if let Ok(files) = fs::read_dir(day_dir) {
        for file in files.flatten() {
            if let Some(file_name) = file.file_name().to_str() {
                if file_name.ends_with(".md") {
                    if let Some(memo) = create_memo_file(&file.path(), base_memo_dir) {
                        memos.push(memo);
                    }
                }
            }
        }
    }
}

fn create_memo_file(file_path: &PathBuf, memo_dir: &PathBuf) -> Option<MemoFile> {
    // Extract ID from path
    let relative_path = file_path.strip_prefix(memo_dir).ok()?;
    let id = relative_path.to_str()?.trim_end_matches(".md").to_string();

    // Get modification time
    let metadata = fs::metadata(file_path).ok()?;
    let modified = DateTime::from(metadata.modified().ok()?);

    // Get preview (first few lines)
    let preview = get_preview(file_path);

    Some(MemoFile {
        id,
        modified,
        preview,
    })
}

fn get_preview(file_path: &PathBuf) -> String {
    if let Ok(content) = fs::read_to_string(file_path) {
        let lines: Vec<&str> = content.lines().take(3).collect();
        let preview = lines.join(" ");
        if preview.chars().count() > 100 {
            let truncated: String = preview.chars().take(97).collect();
            format!("{}...", truncated)
        } else {
            preview
        }
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_test_memos() -> (tempfile::TempDir, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let memo_dir = temp_dir.path().join("memo");

        // Create test directory structure
        let test_date_dir = memo_dir.join("2025-01/30");
        fs::create_dir_all(&test_date_dir).unwrap();

        // Create test memo files
        fs::write(
            test_date_dir.join("143022.md"),
            "# Test Memo\n\nThis is a test memo with @tag1 @meeting",
        )
        .unwrap();
        fs::write(
            test_date_dir.join("151545.md"),
            "# Another Memo\n\nAnother test memo with @tag2",
        )
        .unwrap();

        (temp_dir, memo_dir)
    }

    #[test]
    fn test_collect_memos() {
        let (_temp_dir, memo_dir) = setup_test_memos();

        let memos = collect_memos_with_memo_dir(&memo_dir, &memo_dir);
        assert_eq!(memos.len(), 2);

        // Check that memos have correct IDs
        let ids: Vec<&String> = memos.iter().map(|m| &m.id).collect();
        assert!(ids.contains(&&"2025-01/30/143022".to_string()));
        assert!(ids.contains(&&"2025-01/30/151545".to_string()));
    }

    #[test]
    fn test_get_preview() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.md");

        fs::write(
            &test_file,
            "# Title\n\nFirst line\nSecond line\nThird line\nFourth line",
        )
        .unwrap();

        let preview = get_preview(&test_file);
        // The preview takes first 3 lines: "# Title", "", "First line"
        assert_eq!(preview, "# Title  First line");
    }

    #[test]
    fn test_get_preview_long_content() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.md");

        let long_content = "a".repeat(150);
        fs::write(&test_file, &long_content).unwrap();

        let preview = get_preview(&test_file);
        assert!(preview.ends_with("..."));
        assert!(preview.len() <= 100);
    }

    #[test]
    fn test_create_memo_file() {
        let (_temp_dir, memo_dir) = setup_test_memos();

        let test_file = memo_dir.join("2025-01/30/143022.md");

        // Use the test version that accepts memo_dir parameter
        if let Some(memo_file) = create_memo_file(&test_file, &memo_dir) {
            assert_eq!(memo_file.id, "2025-01/30/143022");
            assert!(memo_file.preview.contains("Test Memo"));
        } else {
            panic!("Failed to create memo file");
        }
    }
}
