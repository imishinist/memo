use std::path::PathBuf;
use std::process::Command;

fn get_binary_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("target");
    path.push("debug");
    path.push("memo");
    path
}

#[test]
fn test_memo_dir_command() {
    let binary = get_binary_path();
    
    let output = Command::new(&binary)
        .arg("dir")
        .output()
        .expect("Failed to execute memo dir");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("memo"));
}

#[test]
fn test_memo_list_empty() {
    let binary = get_binary_path();
    
    let output = Command::new(&binary)
        .arg("list")
        .output()
        .expect("Failed to execute memo list");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("No memos found"));
}

#[test]
fn test_memo_edit_nonexistent() {
    let binary = get_binary_path();
    
    let output = Command::new(&binary)
        .arg("edit")
        .arg("999999")
        .output()
        .expect("Failed to execute memo edit");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("not found"));
}

#[test]
#[ignore] // Skip this test due to permission issues with XDG directory
fn test_memo_add_with_echo_editor() {
    let binary = get_binary_path();
    
    let output = Command::new(&binary)
        .arg("add")
        .env("EDITOR", "echo")
        .output()
        .expect("Failed to execute memo add");
    
    if !output.status.success() {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        println!("status: {}", output.status);
    }
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Memo created"));
}

#[test]
fn test_memo_help() {
    let binary = get_binary_path();
    
    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to execute memo --help");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("memo"));
    assert!(stdout.contains("add"));
    assert!(stdout.contains("edit"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("dir"));
}
