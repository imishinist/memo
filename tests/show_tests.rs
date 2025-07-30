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

fn create_test_memo(temp_dir: &TempDir, content: &str) -> String {
    let xdg_data_home = temp_dir.path();
    let memo_dir = xdg_data_home.join("memo");
    
    // 現在の日時を使ってディレクトリ構造を作成
    let now = chrono::Local::now();
    let year_month = now.format("%Y-%m").to_string();
    let day = now.format("%d").to_string();
    let time_id = now.format("%H%M%S").to_string();
    
    let memo_path = memo_dir
        .join(&year_month)
        .join(&day);
    
    fs::create_dir_all(&memo_path).unwrap();
    
    let file_path = memo_path.join(format!("{}.md", time_id));
    fs::write(&file_path, content).unwrap();
    
    // 短縮IDを返す（時分秒のみ）
    time_id
}

#[test]
fn test_memo_show_existing_memo() {
    let temp_dir = TempDir::new().unwrap();
    let xdg_data_home = temp_dir.path();
    let binary = get_binary_path();
    
    let test_content = "# Test Memo\n\nThis is a test memo content.\n\n@test @memo\n";
    let memo_id = create_test_memo(&temp_dir, test_content);
    
    let output = Command::new(&binary)
        .arg("show")
        .arg(&memo_id)
        .env("XDG_DATA_HOME", xdg_data_home)
        .output()
        .expect("Failed to execute memo show");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, test_content);
}

#[test]
fn test_memo_show_nonexistent_memo() {
    let temp_dir = TempDir::new().unwrap();
    let xdg_data_home = temp_dir.path();
    let binary = get_binary_path();
    
    let output = Command::new(&binary)
        .arg("show")
        .arg("999999")
        .env("XDG_DATA_HOME", xdg_data_home)
        .output()
        .expect("Failed to execute memo show");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("not found") || stderr.contains("Error"));
}

#[test]
fn test_memo_show_empty_memo() {
    let temp_dir = TempDir::new().unwrap();
    let xdg_data_home = temp_dir.path();
    let binary = get_binary_path();
    
    let test_content = "";
    let memo_id = create_test_memo(&temp_dir, test_content);
    
    let output = Command::new(&binary)
        .arg("show")
        .arg(&memo_id)
        .env("XDG_DATA_HOME", xdg_data_home)
        .output()
        .expect("Failed to execute memo show");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, test_content);
}

#[test]
fn test_memo_show_multiline_content() {
    let temp_dir = TempDir::new().unwrap();
    let xdg_data_home = temp_dir.path();
    let binary = get_binary_path();
    
    let test_content = r#"# Meeting Notes

## Agenda
1. Project status
2. Next steps
3. Action items

## Discussion
- Point 1
- Point 2
- Point 3

@meeting @project @important
"#;
    
    let memo_id = create_test_memo(&temp_dir, test_content);
    
    let output = Command::new(&binary)
        .arg("show")
        .arg(&memo_id)
        .env("XDG_DATA_HOME", xdg_data_home)
        .output()
        .expect("Failed to execute memo show");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, test_content);
}

#[test]
fn test_memo_show_with_special_characters() {
    let temp_dir = TempDir::new().unwrap();
    let xdg_data_home = temp_dir.path();
    let binary = get_binary_path();
    
    let test_content = "# 日本語メモ\n\n特殊文字: !@#$%^&*()_+-=[]{}|;':\",./<>?\n\nEmoji: 🚀 📝 ✅\n";
    let memo_id = create_test_memo(&temp_dir, test_content);
    
    let output = Command::new(&binary)
        .arg("show")
        .arg(&memo_id)
        .env("XDG_DATA_HOME", xdg_data_home)
        .output()
        .expect("Failed to execute memo show");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, test_content);
}
