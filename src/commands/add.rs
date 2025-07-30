use chrono::Local;
use std::process::Command;
use std::path::PathBuf;

use crate::utils::xdg::ensure_memo_dir;

pub fn run() {
    let now = Local::now();
    
    // Create directory structure: YYYY-MM/DD/
    let date_dir = now.format("%Y-%m/%d").to_string();
    let time_filename = now.format("%H%M%S.md").to_string();
    
    // Ensure memo directory exists
    let memo_dir = match ensure_memo_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Error creating memo directory: {}", e);
            std::process::exit(1);
        }
    };
    
    // Create the full path
    let full_dir = memo_dir.join(&date_dir);
    if let Err(e) = std::fs::create_dir_all(&full_dir) {
        eprintln!("Error creating date directory: {}", e);
        std::process::exit(1);
    }
    
    let file_path = full_dir.join(&time_filename);
    
    // Create empty file if it doesn't exist
    if !file_path.exists() {
        if let Err(e) = std::fs::write(&file_path, "") {
            eprintln!("Error creating memo file: {}", e);
            std::process::exit(1);
        }
    }
    
    // Open editor
    open_editor(&file_path);
    
    println!("Memo created: {}/{}", date_dir, time_filename.trim_end_matches(".md"));
}

fn open_editor(file_path: &PathBuf) {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    
    let status = Command::new(&editor)
        .arg(file_path)
        .status();
    
    match status {
        Ok(exit_status) => {
            if !exit_status.success() {
                eprintln!("Editor exited with non-zero status");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error launching editor '{}': {}", editor, e);
            std::process::exit(1);
        }
    }
}
