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
fn test_index_with_front_matter_memos() {
    let context = TestContext::new();

    // フロントマター付きメモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::WITH_FRONT_MATTER);
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

    assert_command_success(&output2);
    assert_output_contains(&output2, "Indexing 2 memos");
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

#[cfg(test)]
mod integration_with_search {
    use super::*;

    #[test]
    fn test_index_enables_search() {
        let context = TestContext::new();
        context.create_memo("2025-01/30/143022.md", "Searchable content test");

        let index_output = context.run_command(&["index"]);
        assert_command_success(&index_output);

        let search_output = context.run_command(&["search", "searchable"]);
        assert_command_success(&search_output);
    }
}
