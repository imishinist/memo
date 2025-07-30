use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn get_binary_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("target");
    path.push("debug");
    path.push("memo");
    path
}

fn setup_test_memo_dir() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    // XDG_DATA_HOME will be set to temp_dir.path()
    // memo directory will be temp_dir.path()/memo
    let memo_dir = temp_dir.path().join("memo");

    // Create test memo structure
    let year_month = "2025-01";
    let day = "30";
    let memo_path = memo_dir.join(year_month).join(day);
    fs::create_dir_all(&memo_path).unwrap();

    // Create test memo files
    fs::write(
        memo_path.join("143022.md"),
        "# Test memo 1\nContent 1\n@tag1",
    )
    .unwrap();
    fs::write(
        memo_path.join("151545.md"),
        "# Test memo 2\nContent 2\n@tag2",
    )
    .unwrap();
    fs::write(
        memo_path.join("090000.md"),
        "# Test memo 3\nContent 3\n@tag3",
    )
    .unwrap();

    // Create another day
    let day2 = "29";
    let memo_path2 = memo_dir.join(year_month).join(day2);
    fs::create_dir_all(&memo_path2).unwrap();
    fs::write(
        memo_path2.join("120000.md"),
        "# Test memo 4\nContent 4\n@tag4",
    )
    .unwrap();

    temp_dir
}

#[test]
fn test_archive_single_id() {
    let temp_dir = setup_test_memo_dir();
    let xdg_data_home = temp_dir.path();
    let memo_dir = xdg_data_home.join("memo");
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("archive")
        .arg("2025-01/30/143022.md")
        .env("XDG_DATA_HOME", xdg_data_home)
        .output()
        .expect("Failed to execute memo archive");

    assert!(output.status.success());

    // Check that original file is moved
    assert!(!memo_dir.join("2025-01/30/143022.md").exists());

    // Check that file exists in archive
    assert!(memo_dir.join(".archive/2025-01/30/143022.md").exists());

    // Check .ignore file is created
    assert!(memo_dir.join(".ignore").exists());
    let ignore_content = fs::read_to_string(memo_dir.join(".ignore")).unwrap();
    assert!(ignore_content.contains(".archive"));
}

#[test]
fn test_archive_multiple_ids() {
    let temp_dir = setup_test_memo_dir();
    let xdg_data_home = temp_dir.path();
    let memo_dir = xdg_data_home.join("memo");
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("archive")
        .arg("2025-01/30/143022.md")
        .arg("2025-01/30/151545.md")
        .env("XDG_DATA_HOME", xdg_data_home)
        .output()
        .expect("Failed to execute memo archive");

    assert!(output.status.success());

    // Check that original files are moved
    assert!(!memo_dir.join("2025-01/30/143022.md").exists());
    assert!(!memo_dir.join("2025-01/30/151545.md").exists());

    // Check that files exist in archive
    assert!(memo_dir.join(".archive/2025-01/30/143022.md").exists());
    assert!(memo_dir.join(".archive/2025-01/30/151545.md").exists());

    // Check that other file remains
    assert!(memo_dir.join("2025-01/30/090000.md").exists());
}

#[test]
fn test_archive_directory() {
    let temp_dir = setup_test_memo_dir();
    let xdg_data_home = temp_dir.path();
    let memo_dir = xdg_data_home.join("memo");
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("archive")
        .arg("2025-01/30/")
        .env("XDG_DATA_HOME", xdg_data_home)
        .output()
        .expect("Failed to execute memo archive");

    assert!(output.status.success());

    // Check that original directory files are moved
    assert!(!memo_dir.join("2025-01/30/143022.md").exists());
    assert!(!memo_dir.join("2025-01/30/151545.md").exists());
    assert!(!memo_dir.join("2025-01/30/090000.md").exists());

    // Check that files exist in archive
    assert!(memo_dir.join(".archive/2025-01/30/143022.md").exists());
    assert!(memo_dir.join(".archive/2025-01/30/151545.md").exists());
    assert!(memo_dir.join(".archive/2025-01/30/090000.md").exists());

    // Check that other day remains
    assert!(memo_dir.join("2025-01/29/120000.md").exists());
}

#[test]
fn test_archive_file_path() {
    let temp_dir = setup_test_memo_dir();
    let xdg_data_home = temp_dir.path();
    let memo_dir = xdg_data_home.join("memo");
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("archive")
        .arg("2025-01/30/143022.md")
        .env("XDG_DATA_HOME", xdg_data_home)
        .output()
        .expect("Failed to execute memo archive");

    assert!(output.status.success());

    // Check that original file is moved
    assert!(!memo_dir.join("2025-01/30/143022.md").exists());

    // Check that file exists in archive
    assert!(memo_dir.join(".archive/2025-01/30/143022.md").exists());
}

#[test]
fn test_archive_mixed_targets() {
    let temp_dir = setup_test_memo_dir();
    let xdg_data_home = temp_dir.path();
    let memo_dir = xdg_data_home.join("memo");
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("archive")
        .arg("2025-01/30/143022.md") // File path
        .arg("2025-01/30/151545.md") // File path
        .arg("2025-01/29/") // Directory
        .env("XDG_DATA_HOME", xdg_data_home)
        .output()
        .expect("Failed to execute memo archive");

    assert!(output.status.success());

    // Check that files are moved
    assert!(!memo_dir.join("2025-01/30/143022.md").exists());
    assert!(!memo_dir.join("2025-01/30/151545.md").exists());
    assert!(!memo_dir.join("2025-01/29/120000.md").exists());

    // Check that files exist in archive
    assert!(memo_dir.join(".archive/2025-01/30/143022.md").exists());
    assert!(memo_dir.join(".archive/2025-01/30/151545.md").exists());
    assert!(memo_dir.join(".archive/2025-01/29/120000.md").exists());

    // Check that remaining file exists
    assert!(memo_dir.join("2025-01/30/090000.md").exists());
}

#[test]
fn test_archive_nonexistent_id() {
    let temp_dir = setup_test_memo_dir();
    let xdg_data_home = temp_dir.path();
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("archive")
        .arg("999999")
        .env("XDG_DATA_HOME", xdg_data_home)
        .output()
        .expect("Failed to execute memo archive");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("not found") || stderr.contains("No memo found"));
}

#[test]
fn test_archive_ignore_file_already_exists() {
    let temp_dir = setup_test_memo_dir();
    let xdg_data_home = temp_dir.path();
    let memo_dir = xdg_data_home.join("memo");

    // Create existing .ignore file
    fs::write(memo_dir.join(".ignore"), "existing_content\n").unwrap();

    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("archive")
        .arg("2025-01/30/143022.md")
        .env("XDG_DATA_HOME", xdg_data_home)
        .output()
        .expect("Failed to execute memo archive");

    assert!(output.status.success());

    // Check that .ignore file contains both existing content and .archive
    let ignore_content = fs::read_to_string(memo_dir.join(".ignore")).unwrap();
    assert!(ignore_content.contains("existing_content"));
    assert!(ignore_content.contains(".archive"));
}

#[test]
fn test_archive_short_id_formats() {
    let temp_dir = setup_test_memo_dir();
    let xdg_data_home = temp_dir.path();
    let binary = get_binary_path();

    // Test different ID formats - these might work if the current date matches
    // Let's test with a non-existent ID that won't match
    let test_cases = vec![
        "999999",     // HHMMSS - non-existent
        "99999999",   // DDHHMMSS - non-existent
        "9999999999", // MMDDHHMMSS - non-existent
    ];

    for id in test_cases.iter() {
        let output = Command::new(&binary)
            .arg("archive")
            .arg(id)
            .env("XDG_DATA_HOME", xdg_data_home)
            .output()
            .expect("Failed to execute memo archive");

        // These should fail since they don't exist
        assert!(!output.status.success());
    }
}

#[test]
fn test_archive_no_arguments() {
    let temp_dir = setup_test_memo_dir();
    let xdg_data_home = temp_dir.path();
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("archive")
        .env("XDG_DATA_HOME", xdg_data_home)
        .output()
        .expect("Failed to execute memo archive");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("required") || stderr.contains("At least one target"));
}
