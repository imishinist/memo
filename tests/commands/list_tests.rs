use crate::utils::{TestContext, TestMemoTemplates, assertions::*};
use serde_json::Value;

#[test]
fn test_list_empty() {
    let context = TestContext::new();

    let output = context.run_command(&["list"]);

    assert_command_success(&output);
    assert_output_contains(&output, "No memos found");
}

#[test]
fn test_list_json_empty() {
    let context = TestContext::new();

    let output = context.run_command(&["list", "--json"]);

    assert_command_success(&output);

    // 空の場合は何も出力されない
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().is_empty());
}

#[test]
fn test_list_with_memos() {
    let context = TestContext::new();

    // テストメモを作成
    context.setup_test_memos();

    let output = context.run_command(&["list"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Recent memos");
    assert_output_contains(&output, "143022");
    assert_output_contains(&output, "151545");
}

#[test]
fn test_list_json_output_integration() {
    let context = TestContext::new();

    // フロントマター付きメモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::WITH_FRONT_MATTER);
    context.create_memo("2025-01/30/151545.md", TestMemoTemplates::BASIC);

    let output = context.run_command(&["list", "--json"]);

    assert_command_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    // 各行がJSONであることを確認
    for line in lines {
        if !line.trim().is_empty() {
            let json: Value = assert_valid_json(line);

            // 必要なフィールドが存在することを確認
            assert!(json.get("id").is_some());
            assert!(json.get("modified").is_some());
            assert!(json.get("preview").is_some());
            assert!(json.get("content").is_some());
        }
    }
}

#[test]
fn test_list_with_front_matter_memos() {
    let context = TestContext::new();

    // フロントマター付きメモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::WITH_FRONT_MATTER);
    context.create_memo("2025-01/30/151545.md", TestMemoTemplates::BASIC);

    let output = context.run_command(&["list"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Test Memo with Frontmatter");
    assert_output_contains(&output, "Basic Memo");
}

#[test]
fn test_list_json_with_frontmatter() {
    let context = TestContext::new();

    // フロントマター付きメモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::WITH_FRONT_MATTER);

    let output = context.run_command(&["list", "--json"]);

    assert_command_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = assert_valid_json(stdout.trim());

    // フロントマターがメタデータとして含まれていることを確認
    assert!(json.get("metadata").is_some());
    let metadata = json.get("metadata").unwrap();
    assert!(metadata.get("title").is_some());
    assert!(metadata.get("tags").is_some());
}

#[test]
fn test_list_with_japanese_content() {
    let context = TestContext::new();

    // 日本語メモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::JAPANESE);
    context.create_memo("2025-01/30/151545.md", "English memo");

    let output = context.run_command(&["list"]);

    assert_command_success(&output);
    assert_output_contains(&output, "日本語テストメモ");
    assert_output_contains(&output, "English memo");
}

#[test]
fn test_list_json_with_japanese_content() {
    let context = TestContext::new();

    // 日本語メモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::JAPANESE);

    let output = context.run_command(&["list", "--json"]);

    assert_command_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = assert_valid_json(stdout.trim());

    // 日本語コンテンツが正しくJSONエンコードされていることを確認
    let content = json.get("content").unwrap().as_str().unwrap();
    assert!(content.contains("日本語"));
}

#[test]
fn test_list_with_special_characters() {
    let context = TestContext::new();

    // 特殊文字を含むメモを作成
    context.create_memo(
        "2025-01/30/143022.md",
        TestMemoTemplates::WITH_SPECIAL_CHARS,
    );

    let output = context.run_command(&["list"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Special Characters");
    // 絵文字の表示は環境依存のため、基本的な内容のみ確認
}

#[test]
fn test_list_preview_truncation() {
    let context = TestContext::new();

    // 非常に長いコンテンツのメモを作成
    let long_content = format!("# Long Content\n\n{}", "A".repeat(500));
    context.create_memo("2025-01/30/143022.md", &long_content);

    let output = context.run_command(&["list"]);

    assert_command_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    // プレビューが適切に切り詰められていることを確認
    // 具体的な長さは実装依存だが、500文字全てが表示されることはないはず
    assert!(stdout.len() < long_content.len());
}

#[test]
fn test_list_with_broken_frontmatter() {
    let context = TestContext::new();

    // 壊れたフロントマターを持つメモを作成
    let broken_frontmatter = r#"---
title: Broken Memo
invalid: [unclosed array
---

This memo has broken frontmatter."#;

    context.create_memo("2025-01/30/143022.md", broken_frontmatter);
    context.create_memo("2025-01/30/151545.md", "Normal memo");

    let output = context.run_command(&["list"]);

    // 壊れたフロントマターがあってもリストは表示される
    assert_command_success(&output);
    assert_output_contains(&output, "143022");
    assert_output_contains(&output, "151545");
}

#[test]
fn test_list_json_with_broken_frontmatter() {
    let context = TestContext::new();

    // 壊れたフロントマターを持つメモを作成
    let broken_frontmatter = r#"---
title: Broken Memo
invalid: [unclosed array
---

This memo has broken frontmatter."#;

    context.create_memo("2025-01/30/143022.md", broken_frontmatter);

    let output = context.run_command(&["list", "--json"]);

    assert_command_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = assert_valid_json(stdout.trim());

    // 実装では壊れたフロントマターでもメタデータが含まれる場合がある
    // 具体的な動作は実装に依存するため、JSONが有効であることのみ確認
    assert!(json.get("id").is_some());
}
