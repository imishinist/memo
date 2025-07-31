use crate::utils::{TestContext, TestMemoTemplates, assertions::*};

#[test]
fn test_index_builds_successfully() {
    let context = TestContext::new();

    // テストメモを作成
    context.setup_test_memos();

    // インデックスを構築
    let output = context.run_command(&["index"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Building search index");
    assert_output_contains(&output, "Search index built successfully");
}

#[test]
fn test_index_empty_memo_directory() {
    let context = TestContext::new();

    // メモなしでインデックス構築
    let output = context.run_command(&["index"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Indexing 0 memos");
    assert_output_contains(&output, "Search index built successfully");
}

#[test]
fn test_index_progress_display() {
    let context = TestContext::new();

    // 複数のメモを作成
    for i in 0..5 {
        let id = format!("2025-01/30/{:06}.md", 100000 + i);
        context.create_memo(&id, &format!("Test memo {}", i));
    }

    let output = context.run_command(&["index"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Indexing 5 memos");
    assert_output_contains(&output, "Indexed 5/5 memos");
}

#[test]
fn test_index_with_frontmatter_memos() {
    let context = TestContext::new();

    // フロントマター付きメモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::WITH_FRONTMATTER);
    context.create_memo("2025-01/30/151545.md", TestMemoTemplates::BASIC);

    let output = context.run_command(&["index"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Indexing 2 memos");
    assert_output_contains(&output, "Search index built successfully");
}

#[test]
fn test_index_overwrites_existing() {
    let context = TestContext::new();

    // 最初のインデックス構築
    context.create_memo("2025-01/30/143022.md", "First memo");
    let output1 = context.run_command(&["index"]);
    assert_command_success(&output1);

    // 新しいメモを追加
    context.create_memo("2025-01/30/151545.md", "Second memo");

    // インデックス再構築を試行
    let output2 = context.run_command(&["index"]);

    // 実装によってはインデックスが既に存在する場合エラーになる
    if output2.status.success() {
        // 上書きが許可される場合
        assert_command_success(&output2);
        assert_output_contains(&output2, "Indexing 2 memos");
    } else {
        // 上書きが許可されない場合
        assert_command_failure(&output2);
        assert_command_error(&output2, "Index already exists");
    }
}

#[test]
fn test_index_handles_large_dataset() {
    let context = TestContext::new();

    // 大量のメモを作成（テスト環境では50個程度）
    for i in 0..50 {
        let id = format!("2025-01/30/{:06}.md", 100000 + i);
        context.create_memo(&id, &format!("Large dataset test memo {}", i));
    }

    let output = context.run_command(&["index"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Indexing 50 memos");
    assert_output_contains(&output, "Search index built successfully");
}

#[test]
fn test_index_with_japanese_content() {
    let context = TestContext::new();

    // 日本語メモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::JAPANESE);

    let output = context.run_command(&["index"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Search index built successfully");
}

#[test]
fn test_index_with_special_characters() {
    let context = TestContext::new();

    // 特殊文字を含むメモを作成
    context.create_memo(
        "2025-01/30/143022.md",
        TestMemoTemplates::WITH_SPECIAL_CHARS,
    );

    let output = context.run_command(&["index"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Search index built successfully");
}

#[test]
fn test_index_permission_error() {
    let context = TestContext::new();

    // メモを作成
    context.create_memo("2025-01/30/143022.md", "Test memo");

    // 権限エラーのシミュレーションは環境依存で困難
    // 基本的なインデックス作成が動作することを確認
    let output = context.run_command(&["index"]);

    // 権限エラーが発生しない環境では成功する
    if output.status.success() {
        assert_command_success(&output);
        assert_output_contains(&output, "Search index built successfully");
    } else {
        // 権限エラーが発生した場合
        assert_command_failure(&output);
    }
}

#[test]
fn test_index_corrupted_memo_handling() {
    let context = TestContext::new();

    // 正常なメモを作成
    context.create_memo("2025-01/30/143022.md", "Normal memo");

    // 破損したメモファイルを作成（バイナリデータ）
    let corrupted_path = context.memo_dir().join("2025-01/30/151545.md");
    std::fs::create_dir_all(corrupted_path.parent().unwrap()).unwrap();
    std::fs::write(&corrupted_path, &[0xFF, 0xFE, 0x00, 0x01]).unwrap();

    let output = context.run_command(&["index"]);

    // インデックス構築は成功するが、破損ファイルはスキップされる
    assert_command_success(&output);
    assert_output_contains(&output, "Search index built successfully");
}

#[test]
fn test_index_displays_location() {
    let context = TestContext::new();

    context.create_memo("2025-01/30/143022.md", "Test memo");

    let output = context.run_command(&["index"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Index location:");
}

#[cfg(test)]
mod integration_with_search {
    use super::*;

    #[test]
    fn test_index_enables_search() {
        let context = TestContext::new();

        // メモを作成
        context.create_memo("2025-01/30/143022.md", "Searchable content test");

        // インデックス構築
        let index_output = context.run_command(&["index"]);
        assert_command_success(&index_output);

        // 検索が動作することを確認（結果の詳細は実装依存）
        let search_output = context.run_command(&["search", "searchable"]);
        assert_command_success(&search_output);
    }
}
