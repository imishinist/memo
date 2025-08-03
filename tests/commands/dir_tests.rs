use crate::utils::{TestContext, assertions::*};
use std::fs;

#[test]
fn test_dir_displays_path() {
    let context = TestContext::new();

    let output = context.run_command(&["dir"]);

    assert_command_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let displayed_path = stdout.trim();

    // 表示されたパスが実際のメモディレクトリと一致することを確認
    assert_eq!(displayed_path, context.memo_dir().to_string_lossy());

    assert!(displayed_path.starts_with(context.temp_dir.path().to_str().unwrap()));
    assert!(displayed_path.ends_with("memo"));
}

#[test]
fn test_dir_creates_directory_if_not_exists() {
    let context = TestContext::new();

    // メモディレクトリを削除
    fs::remove_dir_all(context.memo_dir()).unwrap();

    let output = context.run_command(&["dir"]);

    assert_command_success(&output);

    // ディレクトリが再作成されていることを確認
    assert!(context.memo_dir().exists());
    assert!(context.memo_dir().is_dir());
}

#[test]
fn test_dir_absolute_path() {
    let context = TestContext::new();

    let output = context.run_command(&["dir"]);

    assert_command_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let displayed_path = stdout.trim();

    // 絶対パスが表示されることを確認
    assert!(displayed_path.starts_with('/'));
}

#[cfg(test)]
mod dir_integration_tests {
    use super::*;

    #[test]
    fn test_dir_then_add_workflow() {
        let context = TestContext::new();

        // dirコマンドでパスを取得
        let dir_output = context.run_command(&["dir"]);
        assert_command_success(&dir_output);

        let memo_dir_path = String::from_utf8_lossy(&dir_output.stdout)
            .trim()
            .to_string();

        // addコマンドでメモを作成
        let add_output = context.run_command(&["add"]);
        assert_command_success(&add_output);

        // 指定されたディレクトリにメモが作成されていることを確認
        assert!(fs::read_dir(&memo_dir_path).unwrap().count() > 0);
    }

    #[test]
    fn test_dir_path_usable_by_shell() {
        let context = TestContext::new();

        // テストメモを作成
        context.create_memo("2025-01/30/143022.md", "Shell test content");

        // dirコマンドでパスを取得
        let dir_output = context.run_command(&["dir"]);
        assert_command_success(&dir_output);

        let memo_dir_path = String::from_utf8_lossy(&dir_output.stdout)
            .trim()
            .to_string();

        // シェルコマンドでファイル数をカウント
        let ls_output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&format!("find '{}' -name '*.md' | wc -l", memo_dir_path))
            .output()
            .expect("Failed to execute shell command");

        assert!(ls_output.status.success());

        let count = String::from_utf8_lossy(&ls_output.stdout)
            .trim()
            .parse::<i32>()
            .unwrap();
        assert!(count > 0);
    }
}
