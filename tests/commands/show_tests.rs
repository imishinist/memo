use crate::utils::{
    TestContext, TestMemoTemplates, assertions::*, mocks::create_mock_editor_script,
};

#[test]
fn test_show_existing_memo() {
    let context = TestContext::new();

    // テストメモを作成
    context.create_memo("2025-01/30/20250130143022.md", TestMemoTemplates::BASIC);

    let output = context.run_command(&["show", "20250130143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Basic Memo");
    assert_output_contains(&output, "@test @basic");
}

#[test]
fn test_show_nonexistent_memo() {
    let context = TestContext::new();

    let output = context.run_command(&["show", "999999"]);

    assert_command_failure(&output);
    assert_command_error(&output, "not found");
}

#[test]
fn test_show_empty_memo() {
    let context = TestContext::new();

    // 空のメモを作成
    context.create_memo("2025-01/30/20250130143022.md", "");

    let output = context.run_command(&["show", "20250130143022"]);

    assert_command_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty());
}

#[test]
fn test_show_multiline_content() {
    let context = TestContext::new();

    // 複数行コンテンツのメモを作成
    context.create_memo("2025-01/30/20250130143022.md", TestMemoTemplates::MULTILINE);

    let output = context.run_command(&["show", "20250130143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Multiline Test Memo");
    assert_output_contains(&output, "Section 1");
    assert_output_contains(&output, "Section 2");
    assert_output_contains(&output, "Subsection");
}

#[test]
fn test_show_with_special_characters() {
    let context = TestContext::new();

    // 特殊文字を含むメモを作成
    context.create_memo(
        "2025-01/30/20250130143022.md",
        TestMemoTemplates::WITH_SPECIAL_CHARS,
    );

    let output = context.run_command(&["show", "20250130143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Special Characters");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("🚀"));
    assert!(stdout.contains("📝"));
    assert!(stdout.contains("!@#$%^&*()"));
}

#[test]
fn test_show_with_japanese_content() {
    let context = TestContext::new();

    // 日本語メモを作成
    context.create_memo("2025-01/30/20250130143022.md", TestMemoTemplates::JAPANESE);

    let output = context.run_command(&["show", "20250130143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "日本語テストメモ");
    assert_output_contains(&output, "セクション1");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("🚀"));
    assert!(stdout.contains("📝"));
}

#[test]
fn test_show_with_frontmatter() {
    let context = TestContext::new();

    // フロントマター付きメモを作成
    context.create_memo("2025-01/30/20250130143022.md", TestMemoTemplates::WITH_FRONT_MATTER);

    let output = context.run_command(&["show", "20250130143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "title: Test Memo with Frontmatter");
    assert_output_contains(&output, "tags: [\"@test\", \"@frontmatter\"]");
    assert_output_contains(&output, "This memo has frontmatter");
}

#[test]
fn test_show_invalid_id_format() {
    let context = TestContext::new();

    let output = context.run_command(&["show", "invalid_id_format"]);

    assert_command_failure(&output);
    assert_command_error(&output, "not found");
}

#[test]
fn test_show_with_broken_frontmatter() {
    let context = TestContext::new();

    // 壊れたフロントマターを持つメモを作成
    let broken_frontmatter = r#"---
title: Broken Memo
invalid: [unclosed array
---

This memo has broken frontmatter but should still be displayable."#;

    context.create_memo("2025-01/30/20250130143022.md", broken_frontmatter);

    let output = context.run_command(&["show", "20250130143022"]);

    // 壊れたフロントマターがあっても内容は表示される
    assert_command_success(&output);
    assert_output_contains(&output, "title: Broken Memo");
    assert_output_contains(
        &output,
        "broken frontmatter but should still be displayable",
    );
}

#[cfg(test)]
mod show_integration_tests {
    use super::*;

    #[test]
    fn test_show_after_add_workflow() {
        let script_path = create_mock_editor_script("Content created by add command");
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
        assert_output_contains(&show_output, "Content created by add command");
    }
}
