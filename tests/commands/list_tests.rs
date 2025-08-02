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
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::WITH_FRONTMATTER);
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
fn test_list_with_frontmatter_memos() {
    let context = TestContext::new();

    // フロントマター付きメモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::WITH_FRONTMATTER);
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
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::WITH_FRONTMATTER);

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
fn test_list_more_than_20_memos() {
    let context = TestContext::new();

    // 25個のメモを作成
    for i in 0..25 {
        let id = format!("2025-01/30/{:06}.md", 100000 + i);
        let content = format!("Test memo number {}", i);
        context.create_memo(&id, &content);
    }

    let output = context.run_command(&["list"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Recent memos");
    assert_output_contains(&output, "... and 5 more memos");
}

#[test]
fn test_list_exactly_20_memos() {
    let context = TestContext::new();

    // ちょうど20個のメモを作成
    for i in 0..20 {
        let id = format!("2025-01/30/{:06}.md", 100000 + i);
        let content = format!("Test memo number {}", i);
        context.create_memo(&id, &content);
    }

    let output = context.run_command(&["list"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Recent memos");
    // "more memos" メッセージは表示されないはず
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("more memos"));
}

#[test]
fn test_list_large_dataset() {
    let context = TestContext::new();

    // 100個のメモを作成（有効な時刻を使用）
    for i in 0..100 {
        let hour = 10 + (i / 60);
        let minute = i % 60;
        let second = (i * 7) % 60; // 秒も有効な範囲に
        let id = format!("2025-01/30/{:02}{:02}{:02}.md", hour, minute, second);
        let content = format!("Large dataset memo {}", i);
        context.create_memo(&id, &content);
    }

    let output = context.run_command(&["list"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Recent memos");
    assert_output_contains(&output, "... and 80 more memos");
}

#[test]
fn test_list_json_large_dataset() {
    let context = TestContext::new();

    // 30個のメモを作成
    for i in 0..30 {
        let id = format!("2025-01/30/{:06}.md", 100000 + i);
        let content = format!("JSON test memo {}", i);
        context.create_memo(&id, &content);
    }

    let output = context.run_command(&["list", "--json"]);

    assert_command_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    // 最新20件のみが出力されることを確認
    assert_eq!(lines.len(), 20);

    // 各行がJSONであることを確認
    for line in lines {
        let json: Value = assert_valid_json(line);
        assert!(json.get("id").is_some());
        assert!(json.get("content").is_some());
    }
}

#[test]
fn test_list_corrupted_memo_handling() {
    let context = TestContext::new();

    // 正常なメモを作成
    context.create_memo("2025-01/30/143022.md", "Normal memo");

    // 破損したメモファイルを作成
    let corrupted_path = context.memo_dir().join("2025-01/30/151545.md");
    std::fs::create_dir_all(corrupted_path.parent().unwrap()).unwrap();
    std::fs::write(&corrupted_path, &[0xFF, 0xFE, 0x00, 0x01]).unwrap();

    let output = context.run_command(&["list"]);

    // 正常なメモは表示され、破損ファイルはスキップされる
    assert_command_success(&output);
    assert_output_contains(&output, "143022");
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
fn test_list_sorting_by_modification_time() {
    let context = TestContext::new();

    // 時間差でメモを作成
    context.create_memo("2025-01/30/143022.md", "First memo");
    std::thread::sleep(std::time::Duration::from_millis(1100));

    context.create_memo("2025-01/30/143023.md", "Second memo");
    std::thread::sleep(std::time::Duration::from_millis(1100));

    context.create_memo("2025-01/30/143024.md", "Third memo");

    let output = context.run_command(&["list"]);

    assert_command_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);

    // 最新のメモが最初に表示されることを確認
    let third_pos = stdout.find("Third memo").unwrap();
    let second_pos = stdout.find("Second memo").unwrap();
    let first_pos = stdout.find("First memo").unwrap();

    assert!(third_pos < second_pos);
    assert!(second_pos < first_pos);
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

#[cfg(test)]
mod list_performance_tests {
    use super::*;

    #[test]
    fn test_list_performance_with_many_memos() {
        let context = TestContext::new();

        // 1000個のメモを作成（有効な時刻を使用）
        for i in 0..1000 {
            let day = (i % 30) + 1;
            let hour = 10 + (i / 3600) % 14; // 10-23時
            let minute = (i / 60) % 60;
            let second = i % 60;
            let id = format!("2025-01/{:02}/{:02}{:02}{:02}.md", day, hour, minute, second);
            let content = format!("Performance test memo {}", i);
            context.create_memo(&id, &content);
        }

        let start = std::time::Instant::now();
        let output = context.run_command(&["list"]);
        let duration = start.elapsed();

        assert_command_success(&output);
        assert_output_contains(&output, "Recent memos");

        // パフォーマンステスト: 1000個のメモがあっても5秒以内に完了
        assert!(
            duration.as_secs() < 5,
            "List command took too long: {:?}",
            duration
        );
    }
}
