use crate::context::MemoContext;
use crate::error::{MemoError, MemoResult};
use std::process::Command;

pub fn open_editor(context: &MemoContext, file_path: &std::path::Path) -> MemoResult<()> {
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
