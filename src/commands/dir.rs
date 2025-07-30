use crate::utils::xdg::get_memo_dir;

pub fn run() {
    let memo_dir = get_memo_dir();
    println!("{}", memo_dir.display());
}
