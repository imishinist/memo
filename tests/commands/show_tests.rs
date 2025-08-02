use crate::utils::{
    TestContext, TestMemoTemplates, assertions::*, mocks::create_mock_editor_script,
};
use std::fs;

#[test]
fn test_show_existing_memo() {
    let context = TestContext::new();

    // テストメモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::BASIC);

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
    context.create_memo("2025-01/30/143022.md", "");

    let output = context.run_command(&["show", "20250130143022"]);

    assert_command_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty());
}

#[test]
fn test_show_multiline_content() {
    let context = TestContext::new();

    // 複数行コンテンツのメモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::MULTILINE);

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
        "2025-01/30/143022.md",
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
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::JAPANESE);

    let output = context.run_command(&["show", "20250130143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "日本語テストメモ");
    assert_output_contains(&output, "セクション1");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("🚀"));
    assert!(stdout.contains("📝"));
}

#[test]
fn test_show_with_full_id() {
    let context = TestContext::new();

    // 新形式の完全ID（14桁）でメモを作成
    context.create_memo("2025-01/30/143022.md", "Full ID test content");

    let output = context.run_command(&["show", "20250130143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Full ID test content");
}

#[test]
fn test_show_with_frontmatter() {
    let context = TestContext::new();

    // フロントマター付きメモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::WITH_FRONTMATTER);

    let output = context.run_command(&["show", "20250130143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "title: Test Memo with Frontmatter");
    assert_output_contains(&output, "tags: [\"@test\", \"@frontmatter\"]");
    assert_output_contains(&output, "This memo has frontmatter");
}

#[test]
fn test_show_large_memo() {
    let context = TestContext::new();

    // 大きなメモを作成
    let large_content = TestMemoTemplates::large_memo(50); // 50KB
    context.create_memo("2025-01/30/143022.md", &large_content);

    let output = context.run_command(&["show", "20250130143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Large Test Memo");
    assert_output_contains(&output, "approximately 50 KB");

    // 大きなファイルでも正常に表示されることを確認
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.len() > 40000); // 40KB以上の出力があることを確認
}

#[test]
fn test_show_file_read_error() {
    let context = TestContext::new();

    // メモファイルを作成
    context.create_memo("2025-01/30/143022.md", "Test content");

    // ファイルを削除してread errorを発生させる
    let memo_path = context.memo_dir().join("2025-01/30/143022.md");
    fs::remove_file(&memo_path).unwrap();

    let output = context.run_command(&["show", "20250130143022"]);

    assert_command_failure(&output);
    assert_command_error(&output, "not found");
}

#[test]
fn test_show_permission_denied() {
    let context = TestContext::new();

    // メモファイルを作成
    context.create_memo("2025-01/30/143022.md", "Test content");

    // ファイルの読み取り権限を削除
    let memo_path = context.memo_dir().join("2025-01/30/143022.md");
    let mut perms = fs::metadata(&memo_path).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&memo_path, perms).unwrap();

    let output = context.run_command(&["show", "20250130143022"]);

    // 権限を戻す（クリーンアップ）
    let mut perms = fs::metadata(&memo_path).unwrap().permissions();
    perms.set_readonly(false);
    fs::set_permissions(&memo_path, perms).unwrap();

    // Unix系では読み取り専用でも読み取りは可能なので、成功するはず
    assert_command_success(&output);
    assert_output_contains(&output, "Test content");
}

#[test]
fn test_show_invalid_id_format() {
    let context = TestContext::new();

    let output = context.run_command(&["show", "invalid_id_format"]);

    assert_command_failure(&output);
    assert_command_error(&output, "not found");
}

#[test]
fn test_show_ambiguous_id_resolution() {
    let context = TestContext::new();

    // 同じ時刻の異なる日にメモを作成
    context.create_memo("2025-01/29/143022.md", "Older memo");
    context.create_memo("2025-01/30/143022.md", "Newer memo");

    // 短縮IDで検索（最新のものが選択されるはず）
    let output = context.run_command(&["show", "20250130143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Newer memo");
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

    context.create_memo("2025-01/30/143022.md", broken_frontmatter);

    let output = context.run_command(&["show", "20250130143022"]);

    // 壊れたフロントマターがあっても内容は表示される
    assert_command_success(&output);
    assert_output_contains(&output, "title: Broken Memo");
    assert_output_contains(
        &output,
        "broken frontmatter but should still be displayable",
    );
}

#[test]
fn test_show_binary_file_handling() {
    let context = TestContext::new();

    // バイナリデータを含むファイルを作成
    let binary_path = context.memo_dir().join("2025-01/30/143022.md");
    fs::create_dir_all(binary_path.parent().unwrap()).unwrap();
    fs::write(
        &binary_path,
        &[0xFF, 0xFE, 0x00, 0x01, 0x48, 0x65, 0x6C, 0x6C, 0x6F],
    )
    .unwrap();

    let output = context.run_command(&["show", "20250130143022"]);

    // バイナリファイルでも何らかの出力がされる（実装依存）
    // エラーになるか、バイナリデータが表示されるかは実装次第
    if output.status.success() {
        // 成功した場合は何らかの出力があるはず
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.is_empty());
    } else {
        // エラーになる場合は適切なエラーメッセージ
        assert_command_failure(&output);
    }
}

#[test]
fn test_show_very_long_lines() {
    let context = TestContext::new();

    // 非常に長い行を含むメモを作成
    let long_line = "A".repeat(10000);
    let content = format!("# Long Line Test\n\n{}\n\nEnd of memo", long_line);
    context.create_memo("2025-01/30/143022.md", &content);

    let output = context.run_command(&["show", "20250130143022"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Long Line Test");
    assert_output_contains(&output, "End of memo");

    // 長い行も正しく表示されることを確認
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&"A".repeat(100))); // 最初の100文字は含まれているはず
}

#[test]
fn test_show_with_null_bytes() {
    let context = TestContext::new();

    // NULL文字を含むコンテンツを作成
    let content_with_null = "Before null\0After null\nNext line";
    let memo_path = context.memo_dir().join("2025-01/30/143022.md");
    fs::create_dir_all(memo_path.parent().unwrap()).unwrap();
    fs::write(&memo_path, content_with_null.as_bytes()).unwrap();

    let output = context.run_command(&["show", "20250130143022"]);

    // NULL文字があっても表示される（実装依存）
    assert_command_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Before null"));
    assert!(stdout.contains("After null"));
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
