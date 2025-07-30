use crate::context::MemoContext;
use crate::error::{MemoError, MemoResult};
use crate::repository::MemoRepository;
use std::process::Command;

pub fn run(context: &MemoContext, id: &str) -> MemoResult<()> {
    let repo = MemoRepository::new(context.clone());
    let memo = repo.find_memo_by_id(id)?;

    open_editor(context, &memo.path)?;

    println!("Memo edited: {}", id);
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
