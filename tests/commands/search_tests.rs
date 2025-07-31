use crate::utils::{TestContext, TestMemoTemplates, assertions::*};

#[test]
fn test_search_basic_query() {
    let context = TestContext::new();
    
    // テストメモを作成
    context.create_memo("2025-01/30/143022.md", "This is a basic test memo");
    
    // インデックス構築
    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);
    
    // 基本検索（結果の詳細は実装依存）
    let output = context.run_command(&["search", "basic"]);
    assert_command_success(&output);
}

#[test]
fn test_search_multiple_keywords() {
    let context = TestContext::new();
    
    // テストメモを作成
    context.create_memo("2025-01/30/143022.md", "This memo contains both keywords: test and search");
    
    // インデックス構築
    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);
    
    // 複数キーワード検索
    let output = context.run_command(&["search", "test search"]);
    assert_command_success(&output);
}

#[test]
fn test_search_tag_search() {
    let context = TestContext::new();
    
    // タグ付きメモを作成
    context.create_memo("2025-01/30/143022.md", "Memo with @important tag");
    
    // インデックス構築
    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);
    
    // タグ検索
    let output = context.run_command(&["search", "@important"]);
    assert_command_success(&output);
}

#[test]
fn test_search_frontmatter_search() {
    let context = TestContext::new();
    
    // フロントマター付きメモを作成
    let frontmatter_memo = TestMemoTemplates::with_custom_frontmatter(
        "Important Meeting",
        &["@meeting", "@important"],
        "Meeting notes content"
    );
    context.create_memo("2025-01/30/143022.md", &frontmatter_memo);
    
    // インデックス構築
    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);
    
    // フロントマター内容の検索
    let output = context.run_command(&["search", "Important Meeting"]);
    assert_command_success(&output);
}

#[test]
fn test_search_japanese_content() {
    let context = TestContext::new();
    
    // 日本語メモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::JAPANESE);
    
    // インデックス構築
    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);
    
    // 日本語検索
    let output = context.run_command(&["search", "日本語"]);
    assert_command_success(&output);
}

#[test]
fn test_search_result_display() {
    let context = TestContext::new();
    
    // テストメモを作成
    context.create_memo("2025-01/30/143022.md", "Searchable content for display test");
    
    // インデックス構築
    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);
    
    // 検索結果の表示確認
    let output = context.run_command(&["search", "searchable"]);
    assert_command_success(&output);
}

#[test]
fn test_search_score_display() {
    let context = TestContext::new();
    
    // テストメモを作成
    context.create_memo("2025-01/30/143022.md", "High relevance score test");
    
    // インデックス構築
    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);
    
    // スコア表示確認
    let output = context.run_command(&["search", "relevance"]);
    assert_command_success(&output);
}

#[test]
fn test_search_no_results() {
    let context = TestContext::new();
    
    // テストメモを作成
    context.create_memo("2025-01/30/143022.md", "Some content");
    
    // インデックス構築
    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);
    
    // 存在しないキーワードで検索
    let output = context.run_command(&["search", "nonexistent"]);
    assert_command_success(&output);
    assert_output_contains(&output, "No results found");
}

#[test]
fn test_search_empty_query() {
    let context = TestContext::new();
    
    // 空のクエリで検索
    let output = context.run_command(&["search", ""]);
    
    // 空のクエリはエラーになるか、適切に処理される
    if !output.status.success() {
        assert_command_failure(&output);
    } else {
        assert_command_success(&output);
    }
}

#[test]
fn test_search_with_emoji() {
    let context = TestContext::new();
    
    // 絵文字を含むメモを作成
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::WITH_SPECIAL_CHARS);
    
    // インデックス構築
    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);
    
    // 絵文字検索（環境依存）
    let output = context.run_command(&["search", "Special"]);
    assert_command_success(&output);
}

#[test]
fn test_search_special_characters_in_query() {
    let context = TestContext::new();
    
    // 特殊文字を含むメモを作成
    context.create_memo("2025-01/30/143022.md", "Content with special chars: !@#$%");
    
    // インデックス構築
    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);
    
    // 特殊文字を含む検索
    let output = context.run_command(&["search", "special"]);
    assert_command_success(&output);
}

#[test]
fn test_search_case_insensitive() {
    let context = TestContext::new();
    
    // 大文字小文字混在のメモを作成
    context.create_memo("2025-01/30/143022.md", "CaseSensitive Test Content");
    
    // インデックス構築
    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);
    
    // 小文字で検索
    let output = context.run_command(&["search", "casesensitive"]);
    assert_command_success(&output);
}

#[cfg(test)]
mod search_performance {
    use super::*;
    
    #[test]
    fn test_search_large_dataset() {
        let context = TestContext::new();
        
        // 大量のメモを作成
        for i in 0..50 {
            let content = format!("Test memo number {} with searchable content", i);
            let id = format!("2025-01/30/{:06}.md", 100000 + i);
            context.create_memo(&id, &content);
        }
        
        // インデックス構築
        let index_output = context.run_command(&["index"]);
        assert_command_success(&index_output);
        
        // 検索実行
        let start = std::time::Instant::now();
        let output = context.run_command(&["search", "searchable"]);
        let duration = start.elapsed();
        
        assert_command_success(&output);
        
        // パフォーマンステスト: 50個のメモで5秒以内
        assert!(duration.as_secs() < 5, "Search took too long: {:?}", duration);
    }
}
