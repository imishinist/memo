use crate::context::MemoContext;
use crate::error::MemoResult;

pub fn run(context: &MemoContext) -> MemoResult<()> {
    println!("{}", context.memo_dir.display());
    Ok(())
}
