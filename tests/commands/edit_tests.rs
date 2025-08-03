use crate::utils::{TestContext, TestMemoTemplates, assertions::*, mocks::*};
use std::fs;

#[test]
fn test_edit_existing_memo() {
    let context = TestContext::new();

    // テストメモを作成
    context.create_memo("2025-01/30/143022.md", "Original content");

    let output = context.run_command(&["edit", "20250130143022"]);
    assert_command_success(&output);
    assert_output_contains(&output, "Memo edited: 20250130143022");
}

#[test]
fn test_edit_with_content_modification() {
    let script_path = create_mock_editor_script("Modified content by editor");
    let context = TestContext::with_editor(script_path.to_str().unwrap());

    // 初期内容でメモを作成
    context.create_memo("2025-01/30/143022.md", "Original content");

    let output = context.run_command(&["edit", "20250130143022"]);
    assert_command_success(&output);

    // ファイル内容が更新されていることを確認
    let memo_path = context.memo_dir().join("2025-01/30/143022.md");
    let content = fs::read_to_string(&memo_path).unwrap();
    assert!(content.contains("Modified content by editor"));
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

    let output = context.run_command(&["edit", "20250130143022"]);
    assert_command_failure(&output);
    assert_command_error(&output, "Failed to launch editor");
}

#[test]
fn test_edit_editor_exits_with_error() {
    let context = TestContext::with_editor(&mock_editor_fail());

    // テストメモを作成
    context.create_memo("2025-01/30/143022.md", "Test content");

    let output = context.run_command(&["edit", "20250130143022"]);
    assert_command_failure(&output);
    assert_command_error(&output, "Editor exited with non-zero status");
}

#[test]
fn test_edit_memo_with_front_matter() {
    let script_path = create_mock_editor_script(&TestMemoTemplates::WITH_FRONT_MATTER);
    let context = TestContext::with_editor(script_path.to_str().unwrap());

    // フロントマター付きメモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::WITH_FRONT_MATTER);

    let output = context.run_command(&["edit", "20250130143022"]);
    assert_command_success(&output);

    // フロントマターが保持されていることを確認
    let memo_path = context.memo_dir().join("2025-01/30/143022.md");
    let content = fs::read_to_string(&memo_path).unwrap();
    assert!(content.contains("title: Test Memo with Frontmatter"));
    assert!(content.contains("tags: [\"@test\", \"@frontmatter\"]"));
}

#[test]
fn test_edit_adds_front_matter() {
    let front_matter_content = TestMemoTemplates::with_custom_frontmatter(
        "Added Frontmatter",
        &["@added", "@edit"],
        "Content with newly added frontmatter",
    );
    let script_path = create_mock_editor_script(&front_matter_content);
    let context = TestContext::with_editor(script_path.to_str().unwrap());

    // 通常のメモを作成
    context.create_memo(
        "2025-01/30/143022.md",
        "Original content without frontmatter",
    );

    let output = context.run_command(&["edit", "20250130143022"]);

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

    let output = context.run_command(&["edit", "20250130143022"]);
    assert_command_success(&output);

    // 内容が追加されていることを確認
    let memo_path = context.memo_dir().join("2025-01/30/143022.md");
    let content = fs::read_to_string(&memo_path).unwrap();
    assert!(content.contains("Content added to empty memo"));
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

    let output = context.run_command(&["edit", "20250130143022"]);

    assert_command_success(&output);

    // 特殊文字が正しく処理されていることを確認
    let memo_path = context.memo_dir().join("2025-01/30/143022.md");
    let content = fs::read_to_string(&memo_path).unwrap();
    assert!(content.contains("特殊文字"));
    assert!(content.contains("🚀"));
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
        let edit_output = context.run_command(&["edit", "20250130143022"]);
        assert_command_success(&edit_output);

        // 編集したメモを表示
        let show_output = context.run_command(&["show", "20250130143022"]);
        assert_command_success(&show_output);
        assert_output_contains(&show_output, "Edited content for workflow test");
    }

    #[test]
    fn test_edit_then_search_workflow() {
        let script_path = create_mock_editor_script("Edited content for search test");
        let context = TestContext::with_editor(script_path.to_str().unwrap());

        // メモを作成
        context.create_memo("2025-01/30/143022.md", "Original content");

        // メモを編集
        let edit_output = context.run_command(&["edit", "20250130143022"]);
        assert_command_success(&edit_output);

        // 編集したメモが検索に表示されることを確認
        let search_output = context.run_command(&["search", "search test"]);
        assert_command_success(&search_output);
        assert_output_contains(&search_output, "Edited content for search test");
    }
}
