use crate::context::MemoContext;
use crate::error::MemoError;
use crate::repository::MemoRepository;
use crate::search::{IndexLock, SearchManager};

pub fn run_index(ctx: &MemoContext) -> Result<(), MemoError> {
    println!("Building search index...");

    let data_dir = ctx.memo_dir.clone();
    let index_dir = ctx.index_dir();

    let repo = MemoRepository::new(ctx.clone());
    let search_manager = SearchManager::new(data_dir, index_dir);

    let mut index = search_manager.create_new_index()?;
    let _lock = IndexLock::acquire(&index.index_dir)?;
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
