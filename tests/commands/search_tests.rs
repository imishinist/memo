use crate::utils::{TestContext, TestMemoTemplates, assertions::*};

#[test]
fn test_search_basic_query() {
    let context = TestContext::new();
    context.create_memo("2025-01/30/143022.md", "This is a basic test memo");

    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);

    let output = context.run_command(&["search", "basic"]);
    assert_command_success(&output);
}

#[test]
fn test_search_multiple_keywords() {
    let context = TestContext::new();
    context.create_memo(
        "2025-01/30/143022.md",
        "This memo contains both keywords: test and search",
    );

    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);

    let output = context.run_command(&["search", "test search"]);
    assert_command_success(&output);
}

#[test]
fn test_search_tag_search() {
    let context = TestContext::new();
    context.create_memo("2025-01/30/143022.md", "Memo with @important tag");

    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);

    let output = context.run_command(&["search", "@important"]);
    assert_command_success(&output);
}

#[test]
fn test_search_metadata_tag() {
    let context = TestContext::new();
    context.create_memo(
        "2025-01/30/143022.md",
        r#"---
tags: ["@project"]
---
Memo with metadata tag @project"#,
    );
    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);

    let output = context.run_command(&["search", "tags:/@project"]);
    assert_command_success(&output);

    let output = context.run_command(&["search", "tags:@project"]);
    assert_command_failure(&output)
}

#[test]
fn test_search_frontmatter_search() {
    let context = TestContext::new();
    let front_matter_memo = TestMemoTemplates::with_custom_frontmatter(
        "Important Meeting",
        &["@meeting", "@important"],
        "Meeting notes content",
    );
    context.create_memo("2025-01/30/143022.md", &front_matter_memo);

    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);

    let output = context.run_command(&["search", "Important Meeting"]);
    assert_command_success(&output);
}

#[test]
fn test_search_japanese_content() {
    let context = TestContext::new();
    context.create_memo("2025-01/30/143022.md", TestMemoTemplates::JAPANESE);

    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);

    let output = context.run_command(&["search", "日本語"]);
    assert_command_success(&output);
}

#[test]
fn test_search_result_display() {
    let context = TestContext::new();
    context.create_memo(
        "2025-01/30/143022.md",
        "Searchable content for display test",
    );

    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);

    let output = context.run_command(&["search", "searchable"]);
    assert_command_success(&output);
}

#[test]
fn test_search_no_results() {
    let context = TestContext::new();
    context.create_memo("2025-01/30/143022.md", "Some content");

    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);

    let output = context.run_command(&["search", "nonexistent"]);
    assert_command_success(&output);
    assert_output_contains(&output, "No results found");
}

#[test]
fn test_search_empty_query() {
    let context = TestContext::new();

    let output = context.run_command(&["search"]);
    assert_command_failure(&output);
}

#[test]
fn test_search_with_emoji() {
    let context = TestContext::new();
    context.create_memo(
        "2025-01/30/143022.md",
        TestMemoTemplates::WITH_SPECIAL_CHARS,
    );

    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);

    let output = context.run_command(&["search", "Special"]);
    assert_command_success(&output);
}

#[test]
fn test_search_special_characters_in_query() {
    let context = TestContext::new();
    context.create_memo("2025-01/30/143022.md", "Content with special chars: !@#$%");

    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);

    let output = context.run_command(&["search", "special"]);
    assert_command_success(&output);
}
