use crate::context::MemoContext;
use crate::error::MemoResult;
use crate::memo::{MemoDocument, MemoFile};
use crate::repository::MemoRepository;
use crate::search::SearchManager;
use crate::utils::editor;

pub fn run(context: &MemoContext, id: &str) -> MemoResult<()> {
    let repo = MemoRepository::new(context.clone());
    let memo = repo.find_memo_by_id(id)?;

    editor::open_editor(context, &memo.path)?;
    update_search_index(context, &memo.path)?;

    println!("Memo edited: {}", id);
    Ok(())
}

fn update_search_index(context: &MemoContext, memo_path: &std::path::Path) -> MemoResult<()> {
    let data_dir = context.memo_dir.clone();
    let index_dir = context.index_dir();
    let search_manager = SearchManager::new(data_dir, index_dir);

    if let Ok(memo_file) = MemoFile::from_path(memo_path) {
        let memo_doc = MemoDocument::from_memo_file(&memo_file);

        let _ = search_manager.remove_memo(&memo_doc)?;
        let _ = search_manager.add_memo(&memo_doc)?;
    }

    Ok(())
}
