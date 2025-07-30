use crate::context::MemoContext;
use crate::error::{MemoError, MemoResult};
use crate::repository::MemoRepository;
use crate::search::SearchManager;
use crate::memo::MemoDocument;
use std::process::Command;

pub fn run(context: &MemoContext, id: &str) -> MemoResult<()> {
    let repo = MemoRepository::new(context.clone());
    let memo = repo.find_memo_by_id(id)?;

    open_editor(context, &memo.path)?;

    // インデックスを更新
    update_search_index(context, &memo.path)?;

    println!("Memo edited: {}", id);
    Ok(())
}

fn update_search_index(context: &MemoContext, memo_path: &std::path::Path) -> MemoResult<()> {
    // data_dirを取得（memo_dirの親ディレクトリ）
    let data_dir = context.memo_dir.parent()
        .unwrap_or(&context.memo_dir)
        .to_path_buf();
    
    let search_manager = SearchManager::new(data_dir);
    
    // メモファイルを読み込んでMemoDocumentに変換
    if let Ok(memo_file) = crate::memo::MemoFile::from_path(memo_path) {
        let memo_doc = MemoDocument::from_memo_file(&memo_file);
        
        // 既存のエントリを削除してから追加（更新）
        let _ = search_manager.remove_memo(&memo_doc.path);
        let _ = search_manager.add_memo(&memo_doc);
    }
    
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
