use crate::context::Context;
use crate::error::MemoError;
use crate::repository::MemoRepository;
use crate::search::{IndexLock, SearchManager};

pub fn run_index(ctx: &Context) -> Result<(), MemoError> {
    println!("Building search index...");

    let search_manager = SearchManager::new(ctx.data_dir.clone());
    let repo = MemoRepository::from_data_dir(ctx.data_dir.clone());

    // 新しいインデックスを作成
    let mut index = search_manager.create_new_index()?;

    // ロックを取得
    let _lock = IndexLock::acquire(&index.index_dir)?;

    // 全メモを取得してインデックスに追加
    let memos = repo.list_all_memo_documents()?;
    let total = memos.len();

    println!("Indexing {} memos...", total);

    for (i, memo) in memos.iter().enumerate() {
        index.add_memo(memo)?;

        if (i + 1) % 100 == 0 || i + 1 == total {
            println!("Indexed {}/{} memos", i + 1, total);
        }
    }

    // コミット
    index.commit()?;

    println!("Search index built successfully!");
    println!("Index location: {}", index.index_dir.display());

    Ok(())
}
