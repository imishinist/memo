use crate::utils::{TestContext, assertions::*, mocks::*};
use std::fs;

#[test]
fn test_add_creates_memo_file() {
    let context = TestContext::new();
    let output = context.run_command(&["add"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Memo created:");

    // 作成されたファイルの確認
    let memo_dir = context.memo_dir();
    let entries: Vec<_> = fs::read_dir(memo_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .collect();

    assert!(!entries.is_empty(), "No memo files were created");
}

#[test]
fn test_add_with_echo_editor() {
    let context = TestContext::with_editor("echo");

    let output = context.run_command(&["add"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Memo created:");
}


#[test]
fn test_add_editor_not_found() {
    let context = TestContext::with_editor(&mock_editor_nonexistent());

    let output = context.run_command(&["add"]);

    assert_command_failure(&output);
    assert_command_error(&output, "Failed to launch editor");
}

#[test]
fn test_add_editor_exits_with_error() {
    let context = TestContext::with_editor(&mock_editor_fail());

    let output = context.run_command(&["add"]);

    assert_command_failure(&output);
    assert_command_error(&output, "Editor exited with non-zero status");
}

#[test]
fn test_add_respects_xdg_data_home() {
    let context = TestContext::new();

    let output = context.run_command(&["add"]);

    assert_command_success(&output);

    // XDG_DATA_HOME配下にメモが作成されることを確認
    let memo_dir = context.memo_dir();
    assert!(memo_dir.starts_with(context.temp_dir.path()));
}

#[cfg(test)]
mod add_integration_tests {
    use super::*;

    #[test]
    fn test_add_then_show_workflow() {
        let script_path = create_mock_editor_script("Content for workflow test");
        let context = TestContext::with_editor(script_path.to_str().unwrap());

        // メモを追加
        let add_output = context.run_command(&["add"]);
        assert_command_success(&add_output);

        // 作成されたIDを抽出
        let stdout = String::from_utf8_lossy(&add_output.stdout);
        let id_line = stdout
            .lines()
            .find(|line| line.contains("Memo created:"))
            .expect("Could not find created memo ID");

        let id = id_line
            .split("Memo created: ")
            .nth(1)
            .expect("Could not extract memo ID");

        // 作成したメモを表示
        let show_output = context.run_command(&["show", id]);
        assert_command_success(&show_output);
        assert_output_contains(&show_output, "Content for workflow test");
    }

    #[test]
    fn test_add_then_list_workflow() {
        let script_path = create_mock_editor_script("Content for list test");
        let context = TestContext::with_editor(script_path.to_str().unwrap());

        // メモを追加
        let add_output = context.run_command(&["add"]);
        assert_command_success(&add_output);

        // リストに表示されることを確認
        let list_output = context.run_command(&["list"]);
        assert_command_success(&list_output);
        assert_output_contains(&list_output, "Content for list test");
    }
}
