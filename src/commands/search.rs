use crate::context::Context;
use crate::display::MemoDisplayFormatter;
use crate::error::MemoError;
use crate::memo::MemoFile;
use crate::search::SearchManager;

pub fn run_search(ctx: &Context, query: &str) -> Result<(), MemoError> {
    let search_manager = SearchManager::new(ctx.data_dir.clone());

    // 検索実行
    let results = search_manager.search(query)?;

    if results.is_empty() {
        println!("No results found for query: {}", query);
        return Ok(());
    }

    // 検索結果をMemoFileに変換
    let memos: Vec<MemoFile> = results
        .iter()
        .map(|result| {
            // MemoDocumentからMemoFileに変換
            // パスからMemoFileを作成
            MemoFile::from_path(&result.memo.path)
        })
        .collect::<Result<Vec<_>, _>>()?;

    // スコアを抽出
    let scores: Vec<f64> = results.iter().map(|r| r.score as f64).collect();

    // 共通の表示機能を使用
    let title = format!("Found {} results for query: {}", results.len(), query);
    MemoDisplayFormatter::display_memo_list(&memos, &title, Some(&scores));

    Ok(())
}
