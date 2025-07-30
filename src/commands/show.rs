use crate::context::MemoContext;
use crate::error::MemoResult;
use crate::repository::MemoRepository;
use std::fs;

pub fn run(context: &MemoContext, id: &str) -> MemoResult<()> {
    let repo = MemoRepository::new(context.clone());
    let memo = repo.find_memo_by_id(id)?;

    // ファイルの内容を読み込んで出力
    let content = fs::read_to_string(&memo.path)?;
    print!("{}", content);

    Ok(())
}
