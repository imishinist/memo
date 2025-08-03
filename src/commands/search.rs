use crate::context::MemoContext;
use crate::display::MemoDisplayFormatter;
use crate::error::MemoError;
use crate::memo::MemoFile;
use crate::search::SearchManager;

pub fn run_search(ctx: &MemoContext, query: &str) -> Result<(), MemoError> {
    let data_dir = ctx.memo_dir.clone();
    let index_dir = ctx.index_dir();
    let search_manager = SearchManager::new(data_dir, index_dir);

    let results = search_manager.search(query)?;
    if results.is_empty() {
        println!("No results found for query: {}", query);
        return Ok(());
    }

    // 検索結果をMemoFileに変換
    let memos: Vec<MemoFile> = results
        .iter()
        .map(|result| MemoFile::from_path(&result.memo.path))
        .collect::<Result<Vec<_>, _>>()?;

    let title = format!("Found {} results for query: {}", results.len(), query);
    MemoDisplayFormatter::display_memo_list(&memos, &title);

    Ok(())
}
