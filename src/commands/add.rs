use crate::context::MemoContext;
use crate::error::{MemoError, MemoResult};
use crate::repository::MemoRepository;
use chrono::Local;
use std::process::Command;

pub fn run(context: &MemoContext) -> MemoResult<()> {
    let now = Local::now();

    let date_dir = now.format("%Y-%m/%d").to_string();
    let time_filename = now.format("%H%M%S.md").to_string();
    let relative_path = format!("{}/{}", date_dir, time_filename);

    let repo = MemoRepository::new(context.clone());

    let memo = repo.create_memo(&relative_path, String::new())?;

    open_editor(context, &memo.path)?;

    let id = time_filename.trim_end_matches(".md");
    println!("Memo created: {}/{}", date_dir, id);

    Ok(())
}

fn open_editor(context: &MemoContext, file_path: &std::path::Path) -> MemoResult<()> {
    let status = Command::new(&context.editor)
        .arg(file_path)
        .status()
        .map_err(|e| {
            MemoError::EditorError(format!(
                "Failed to launch editor '{}': {}",
                context.editor, e
            ))
        })?;

    if !status.success() {
        return Err(MemoError::EditorError(
            "Editor exited with non-zero status".to_string(),
        ));
    }

    Ok(())
}
