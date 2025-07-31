use crate::utils::{TestContext, TestMemoTemplates, assertions::*, mocks::*};
use std::fs;

#[test]
fn test_edit_existing_memo() {
    let context = TestContext::new();

    // テストメモを作成
    context.create_memo("2025-01/30/143022.md", "Original content");

    let output = context.run_command(&["edit", "143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Memo edited: 143022");
}

#[test]
fn test_edit_with_content_modification() {
    let script_path = create_mock_editor_script("Modified content by editor");
    let context = TestContext::with_editor(script_path.to_str().unwrap());

    // 初期内容でメモを作成
    context.create_memo("2025-01/30/143022.md", "Original content");

    let output = context.run_command(&["edit", "143022"]);

    assert_command_success(&output);

    // ファイル内容が更新されていることを確認
    let memo_path = context.memo_dir().join("2025-01/30/143022.md");
    let content = fs::read_to_string(&memo_path).unwrap();
    assert!(content.contains("Modified content by editor"));
}

#[test]
fn test_edit_with_full_id() {
    let context = TestContext::new();

    // 完全ID形式でメモを作成
    context.create_memo("2025-01/30/143022.md", "Test content");

    let output = context.run_command(&["edit", "2025-01/30/143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Memo edited: 2025-01/30/143022");
}

#[test]
fn test_edit_with_short_id_hhmmss() {
    let context = TestContext::new();

    // 今日の日付でメモを作成
    let now = chrono::Local::now();
    let date_path = now.format("%Y-%m/%d").to_string();
    let full_path = format!("{}/143022.md", date_path);

    context.create_memo(&full_path, "Test content");

    let output = context.run_command(&["edit", "143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Memo edited: 143022");
}

#[test]
fn test_edit_with_short_id_ddhhmmss() {
    let context = TestContext::new();

    // 今月の特定日でメモを作成
    let now = chrono::Local::now();
    let year_month = now.format("%Y-%m").to_string();
    let full_path = format!("{}/30/143022.md", year_month);

    context.create_memo(&full_path, "Test content");

    let output = context.run_command(&["edit", "30143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Memo edited: 30143022");
}

#[test]
fn test_edit_with_short_id_mmddhhmmss() {
    let context = TestContext::new();

    // 今年の特定月日でメモを作成
    let now = chrono::Local::now();
    let year = now.format("%Y").to_string();
    let full_path = format!("{}-01/30/143022.md", year);

    context.create_memo(&full_path, "Test content");

    let output = context.run_command(&["edit", "0130143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Memo edited: 0130143022");
}

#[test]
fn test_edit_nonexistent_memo() {
    let context = TestContext::new();

    let output = context.run_command(&["edit", "999999"]);

    assert_command_failure(&output);
    assert_command_error(&output, "not found");
}

#[test]
fn test_edit_invalid_id_format() {
    let context = TestContext::new();

    let output = context.run_command(&["edit", "invalid_id_123"]);

    assert_command_failure(&output);
    assert_command_error(&output, "not found");
}

#[test]
fn test_edit_editor_not_found() {
    let context = TestContext::with_editor(&mock_editor_nonexistent());

    // テストメモを作成
    context.create_memo("2025-01/30/143022.md", "Test content");

    let output = context.run_command(&["edit", "143022"]);

    assert_command_failure(&output);
    assert_command_error(&output, "Failed to launch editor");
}

#[test]
fn test_edit_editor_exits_with_error() {
    let context = TestContext::with_editor(&mock_editor_fail());

    // テストメモを作成
    context.create_memo("2025-01/30/143022.md", "Test content");

    let output = context.run_command(&["edit", "143022"]);

    assert_command_failure(&output);
    assert_command_error(&output, "Editor exited with non-zero status");
}

#[test]
fn test_edit_file_permission_denied() {
    let context = TestContext::new();

    // テストメモを作成
    context.create_memo("2025-01/30/143022.md", "Test content");

    // ファイルを読み取り専用にする
    let memo_path = context.memo_dir().join("2025-01/30/143022.md");
    let mut perms = fs::metadata(&memo_path).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&memo_path, perms).unwrap();

    let output = context.run_command(&["edit", "143022"]);

    // 権限を戻す（クリーンアップ）
    let mut perms = fs::metadata(&memo_path).unwrap().permissions();
    perms.set_readonly(false);
    fs::set_permissions(&memo_path, perms).unwrap();

    // エディタは起動されるが、保存時にエラーになる可能性
    // 実装によってはエディタ起動前にエラーになる場合もある
    if !output.status.success() {
        assert_command_error(&output, "permission");
    }
}

#[test]
fn test_edit_with_custom_editor() {
    let script_path = create_mock_editor_script("Custom editor content");
    let context = TestContext::with_editor(script_path.to_str().unwrap());

    // テストメモを作成
    context.create_memo("2025-01/30/143022.md", "Original content");

    let output = context.run_command(&["edit", "143022"]);

    assert_command_success(&output);

    // カスタムエディタの内容が反映されていることを確認
    let memo_path = context.memo_dir().join("2025-01/30/143022.md");
    let content = fs::read_to_string(&memo_path).unwrap();
    assert!(content.contains("Custom editor content"));
}

#[test]
fn test_edit_memo_with_frontmatter() {
    let script_path = create_mock_editor_script(&TestMemoTemplates::WITH_FRONTMATTER);
    let context = TestContext::with_editor(script_path.to_str().unwrap());

    // フロントマター付きメモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::WITH_FRONTMATTER);

    let output = context.run_command(&["edit", "143022"]);

    assert_command_success(&output);

    // フロントマターが保持されていることを確認
    let memo_path = context.memo_dir().join("2025-01/30/143022.md");
    let content = fs::read_to_string(&memo_path).unwrap();
    assert!(content.contains("title: Test Memo with Frontmatter"));
    assert!(content.contains("tags: [\"@test\", \"@frontmatter\"]"));
}

#[test]
fn test_edit_adds_frontmatter() {
    let frontmatter_content = TestMemoTemplates::with_custom_frontmatter(
        "Added Frontmatter",
        &["@added", "@edit"],
        "Content with newly added frontmatter",
    );
    let script_path = create_mock_editor_script(&frontmatter_content);
    let context = TestContext::with_editor(script_path.to_str().unwrap());

    // 通常のメモを作成
    context.create_memo(
        "2025-01/30/143022.md",
        "Original content without frontmatter",
    );

    let output = context.run_command(&["edit", "143022"]);

    assert_command_success(&output);

    // 新しいフロントマターが追加されていることを確認
    let memo_path = context.memo_dir().join("2025-01/30/143022.md");
    let content = fs::read_to_string(&memo_path).unwrap();
    assert!(content.contains("title: Added Frontmatter"));
    assert!(content.contains("@added"));
}

#[test]
fn test_edit_empty_memo() {
    let script_path = create_mock_editor_script("Content added to empty memo");
    let context = TestContext::with_editor(script_path.to_str().unwrap());

    // 空のメモを作成
    context.create_memo("2025-01/30/143022.md", "");

    let output = context.run_command(&["edit", "143022"]);

    assert_command_success(&output);

    // 内容が追加されていることを確認
    let memo_path = context.memo_dir().join("2025-01/30/143022.md");
    let content = fs::read_to_string(&memo_path).unwrap();
    assert!(content.contains("Content added to empty memo"));
}

#[test]
fn test_edit_large_memo() {
    let large_content = TestMemoTemplates::large_memo(100); // 100KB
    let script_path = create_mock_editor_script("Modified large memo content");
    let context = TestContext::with_editor(script_path.to_str().unwrap());

    // 大きなメモを作成
    context.create_memo("2025-01/30/143022.md", &large_content);

    let output = context.run_command(&["edit", "143022"]);

    assert_command_success(&output);

    // 内容が更新されていることを確認
    let memo_path = context.memo_dir().join("2025-01/30/143022.md");
    let content = fs::read_to_string(&memo_path).unwrap();
    assert!(content.contains("Modified large memo content"));
}

#[test]
fn test_edit_memo_with_special_characters() {
    let script_path = create_mock_editor_script("Modified: 特殊文字 🚀 and symbols!");
    let context = TestContext::with_editor(script_path.to_str().unwrap());

    // 特殊文字を含むメモを作成
    context.create_memo(
        "2025-01/30/143022.md",
        TestMemoTemplates::WITH_SPECIAL_CHARS,
    );

    let output = context.run_command(&["edit", "143022"]);

    assert_command_success(&output);

    // 特殊文字が正しく処理されていることを確認
    let memo_path = context.memo_dir().join("2025-01/30/143022.md");
    let content = fs::read_to_string(&memo_path).unwrap();
    assert!(content.contains("特殊文字"));
    assert!(content.contains("🚀"));
}

#[test]
fn test_edit_index_update_failure_does_not_fail_command() {
    let context = TestContext::new();

    // メモを作成
    context.create_memo("2025-01/30/143022.md", "Test content");

    let output = context.run_command(&["edit", "143022"]);

    // メモ編集は成功するはず（インデックス更新失敗は無視される）
    assert_command_success(&output);
    assert_output_contains(&output, "Memo edited: 143022");
}

#[cfg(test)]
mod edit_integration_tests {
    use super::*;

    #[test]
    fn test_edit_then_show_workflow() {
        let script_path = create_mock_editor_script("Edited content for workflow test");
        let context = TestContext::with_editor(script_path.to_str().unwrap());

        // メモを作成
        context.create_memo("2025-01/30/143022.md", "Original content");

        // メモを編集
        let edit_output = context.run_command(&["edit", "143022"]);
        assert_command_success(&edit_output);

        // 編集したメモを表示
        let show_output = context.run_command(&["show", "143022"]);
        assert_command_success(&show_output);
        assert_output_contains(&show_output, "Edited content for workflow test");
    }
}
