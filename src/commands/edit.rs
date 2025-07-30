use std::path::PathBuf;
use std::process::Command;

use crate::utils::id_resolver::resolve_id;

pub fn run(id: &str) {
    let file_path = match resolve_id(id) {
        Some(path) => path,
        None => {
            eprintln!("Memo with ID '{}' not found", id);
            std::process::exit(1);
        }
    };

    if !file_path.exists() {
        eprintln!("Memo file does not exist: {}", file_path.display());
        std::process::exit(1);
    }

    // Open editor
    open_editor(&file_path);

    println!("Memo edited: {}", id);
}

fn open_editor(file_path: &PathBuf) {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    let status = Command::new(&editor).arg(file_path).status();

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
