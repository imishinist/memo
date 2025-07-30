use std::fs;
use std::path::PathBuf;

use crate::utils::id_resolver::resolve_id;
use crate::utils::xdg::get_memo_dir;

pub fn run(targets: &[String]) {
    if targets.is_empty() {
        eprintln!("Error: At least one target (ID, file path, or directory) is required");
        std::process::exit(1);
    }

    let memo_dir = get_memo_dir();
    let archive_dir = memo_dir.join(".archive");

    // Ensure .ignore file exists and contains .archive
    ensure_ignore_file(&memo_dir);

    let mut archived_count = 0;
    let mut errors = Vec::new();

    for target in targets {
        match archive_target(&memo_dir, &archive_dir, target) {
            Ok(count) => archived_count += count,
            Err(e) => errors.push(format!("Error archiving '{}': {}", target, e)),
        }
    }

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("{}", error);
        }
        if archived_count == 0 {
            std::process::exit(1);
        }
    }

    if archived_count > 0 {
        println!("Archived {} memo(s)", archived_count);
    }
}

fn archive_target(
    memo_dir: &PathBuf,
    archive_dir: &PathBuf,
    target: &str,
) -> Result<usize, String> {
    // Try to resolve as ID first
    if let Some(file_path) = resolve_id(target) {
        return archive_file(memo_dir, archive_dir, &file_path);
    }

    // Try as file path (with .md extension if not present)
    let file_path = if target.ends_with(".md") {
        memo_dir.join(target)
    } else {
        memo_dir.join(format!("{}.md", target))
    };

    if file_path.exists() && file_path.is_file() {
        return archive_file(memo_dir, archive_dir, &file_path);
    }

    // Try as file path without .md extension
    let file_path_no_ext = memo_dir.join(target);
    if file_path_no_ext.exists() && file_path_no_ext.is_file() {
        return archive_file(memo_dir, archive_dir, &file_path_no_ext);
    }

    // Try as directory path
    let dir_path = memo_dir.join(target);
    if dir_path.exists() && dir_path.is_dir() {
        return archive_directory(memo_dir, archive_dir, &dir_path);
    }

    // Try as directory path without trailing slash
    let dir_path_alt = memo_dir.join(format!("{}/", target.trim_end_matches('/')));
    if dir_path_alt.exists() && dir_path_alt.is_dir() {
        return archive_directory(memo_dir, archive_dir, &dir_path_alt);
    }

    Err(format!("No memo found for target '{}'", target))
}

fn archive_file(
    memo_dir: &PathBuf,
    archive_dir: &PathBuf,
    file_path: &PathBuf,
) -> Result<usize, String> {
    if !file_path.exists() {
        return Err("File does not exist".to_string());
    }

    // Calculate relative path from memo_dir
    let relative_path = file_path
        .strip_prefix(memo_dir)
        .map_err(|_| "File is not in memo directory".to_string())?;

    let archive_file_path = archive_dir.join(relative_path);

    // Create parent directories in archive
    if let Some(parent) = archive_file_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create archive directory: {}", e))?;
    }

    // Move file to archive
    fs::rename(file_path, &archive_file_path)
        .map_err(|e| format!("Failed to move file to archive: {}", e))?;

    Ok(1)
}

fn archive_directory(
    memo_dir: &PathBuf,
    archive_dir: &PathBuf,
    dir_path: &PathBuf,
) -> Result<usize, String> {
    if !dir_path.exists() || !dir_path.is_dir() {
        return Err("Directory does not exist".to_string());
    }

    let mut archived_count = 0;

    // Recursively archive all .md files in the directory
    match archive_directory_recursive(memo_dir, archive_dir, dir_path) {
        Ok(count) => archived_count += count,
        Err(e) => return Err(e),
    }

    // Remove empty directories after archiving
    let _ = remove_empty_dirs(dir_path);

    Ok(archived_count)
}

fn archive_directory_recursive(
    memo_dir: &PathBuf,
    archive_dir: &PathBuf,
    dir_path: &PathBuf,
) -> Result<usize, String> {
    let mut archived_count = 0;

    let entries = fs::read_dir(dir_path).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            match archive_file(memo_dir, archive_dir, &path) {
                Ok(count) => archived_count += count,
                Err(e) => return Err(e),
            }
        } else if path.is_dir() {
            match archive_directory_recursive(memo_dir, archive_dir, &path) {
                Ok(count) => archived_count += count,
                Err(e) => return Err(e),
            }
        }
    }

    Ok(archived_count)
}

fn remove_empty_dirs(dir_path: &PathBuf) -> Result<(), std::io::Error> {
    // Try to remove the directory if it's empty
    match fs::remove_dir(dir_path) {
        Ok(_) => {
            // If successful, try to remove parent directories if they're empty too
            if let Some(parent) = dir_path.parent() {
                let _ = remove_empty_dirs(&parent.to_path_buf());
            }
        }
        Err(_) => {
            // Directory not empty or other error, that's fine
        }
    }
    Ok(())
}

fn ensure_ignore_file(memo_dir: &PathBuf) {
    // Ensure memo directory exists
    if let Err(e) = fs::create_dir_all(memo_dir) {
        eprintln!("Warning: Failed to create memo directory: {}", e);
        return;
    }

    let ignore_path = memo_dir.join(".ignore");

    // Read existing content if file exists
    let existing_content = if ignore_path.exists() {
        fs::read_to_string(&ignore_path).unwrap_or_default()
    } else {
        String::new()
    };

    // Check if .archive is already in the file
    if existing_content
        .lines()
        .any(|line| line.trim() == ".archive")
    {
        return; // Already exists, nothing to do
    }

    // Add .archive to the file
    let new_content = if existing_content.is_empty() {
        ".archive\n".to_string()
    } else if existing_content.ends_with('\n') {
        format!("{}.archive\n", existing_content)
    } else {
        format!("{}\n.archive\n", existing_content)
    };

    if let Err(e) = fs::write(&ignore_path, new_content) {
        eprintln!("Warning: Failed to update .ignore file: {}", e);
    }
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

        // Create another day
        let test_date_dir2 = memo_dir.join("2025-01/29");
        fs::create_dir_all(&test_date_dir2).unwrap();
        fs::write(test_date_dir2.join("120000.md"), "Third memo").unwrap();

        (temp_dir, memo_dir)
    }

    #[test]
    fn test_ensure_ignore_file_new() {
        let temp_dir = tempdir().unwrap();
        let memo_dir = temp_dir.path().to_path_buf();
        fs::create_dir_all(&memo_dir).unwrap();

        ensure_ignore_file(&memo_dir);

        let ignore_path = memo_dir.join(".ignore");
        assert!(ignore_path.exists());

        let content = fs::read_to_string(&ignore_path).unwrap();
        assert!(content.contains(".archive"));
    }

    #[test]
    fn test_ensure_ignore_file_existing() {
        let temp_dir = tempdir().unwrap();
        let memo_dir = temp_dir.path().to_path_buf();
        fs::create_dir_all(&memo_dir).unwrap();

        let ignore_path = memo_dir.join(".ignore");
        fs::write(&ignore_path, "existing_content\n").unwrap();

        ensure_ignore_file(&memo_dir);

        let content = fs::read_to_string(&ignore_path).unwrap();
        assert!(content.contains("existing_content"));
        assert!(content.contains(".archive"));
    }

    #[test]
    fn test_archive_file() {
        let (_temp_dir, memo_dir) = setup_test_memo_structure();
        let archive_dir = memo_dir.join(".archive");

        let file_path = memo_dir.join("2025-01/30/143022.md");
        assert!(file_path.exists());

        let result = archive_file(&memo_dir, &archive_dir, &file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        // Check file is moved
        assert!(!file_path.exists());
        assert!(archive_dir.join("2025-01/30/143022.md").exists());
    }

    #[test]
    fn test_archive_directory() {
        let (_temp_dir, memo_dir) = setup_test_memo_structure();
        let archive_dir = memo_dir.join(".archive");

        let dir_path = memo_dir.join("2025-01/30");

        let result = archive_directory(&memo_dir, &archive_dir, &dir_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2); // Two files in the directory

        // Check files are moved
        assert!(archive_dir.join("2025-01/30/143022.md").exists());
        assert!(archive_dir.join("2025-01/30/151545.md").exists());

        // Check original files are gone
        assert!(!memo_dir.join("2025-01/30/143022.md").exists());
        assert!(!memo_dir.join("2025-01/30/151545.md").exists());
    }
}
