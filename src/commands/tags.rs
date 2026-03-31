use crate::context::MemoContext;
use crate::error::MemoResult;
use crate::search::SearchManager;

pub fn run(context: &MemoContext) -> MemoResult<()> {
    let search_manager = SearchManager::new(context.memo_dir.clone(), context.index_dir());
    let tags = search_manager.list_tags()?;

    if tags.is_empty() {
        println!("No tags found. Run `memo index` first if you have tagged memos.");
        return Ok(());
    }

    for (tag, count) in &tags {
        println!("{:>4}  {}", count, tag);
    }
    Ok(())
}
