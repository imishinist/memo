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

    // Search with facet syntax should work
    let output = context.run_command(&["search", "tags:/@project"]);
    assert_command_success(&output);
    assert_output_contains(&output, "143022");

    // Search without facet syntax should also work (searches in content)
    let output = context.run_command(&["search", "tags:@project"]);
    assert_command_success(&output);
    assert_output_contains(&output, "143022");
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

#[test]
fn test_search_metadata_numeric_values() {
    let context = TestContext::new();
    let memo_with_numeric_metadata = r#"---
title: Priority Task
priority: 1
score: 85.5
completed: false
tags: ["@urgent"]
---

This task has numeric metadata for testing."#;

    context.create_memo("2025-01/30/143022.md", memo_with_numeric_metadata);

    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);

    // Search by title should work
    let output = context.run_command(&["search", "metadata.priority:1"]);
    assert_command_success(&output);
    assert_output_contains(&output, "143022");
}

#[test]
fn test_search_metadata_nested_objects() {
    let context = TestContext::new();
    let memo_with_nested_metadata = r#"---
title: Complex Metadata Test
author:
  name: John Doe
  email: john@example.com
settings:
  public: false
  categories: ["work", "important"]
tags: ["@complex", "@metadata"]
---

This memo has nested metadata structures for testing."#;

    context.create_memo("2025-01/30/143022.md", memo_with_nested_metadata);

    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);

    // Search by title should work
    let output = context.run_command(&["search", "metadata.title:Complex"]);
    assert_command_success(&output);
    assert_output_contains(&output, "143022");

    // Search by content should work
    let output = context.run_command(&["search", "metadata.settings.public:false"]);
    assert_command_success(&output);
    assert_output_contains(&output, "143022");
}

#[test]
fn test_search_metadata_array_values() {
    let context = TestContext::new();
    let memo_with_array_metadata = r#"---
title: Array Metadata Test
tags: ["@project", "@meeting", "@urgent"]
categories: ["work", "planning", "review"]
---

This memo has array metadata for testing search functionality."#;

    context.create_memo("2025-01/30/143022.md", memo_with_array_metadata);

    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);

    // Search by title should work
    let output = context.run_command(&["search", "metadata.categories:work"]);
    assert_command_success(&output);
    assert_output_contains(&output, "143022");
}

// #1: 値が無いものを探す方法が今はない
#[ignore]
#[test]
fn test_search_metadata_missing_values() {
    let context = TestContext::new();

    // Create a memo with missing metadata fields
    let memo_with_missing_metadata = r#"---
title: ""
tags: []
description: null
empty_object: {}
---

This memo has empty metadata values."#;

    context.create_memo("2025-01/30/143022.md", memo_with_missing_metadata);

    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);

    // Search should handle empty values gracefully
    let output = context.run_command(&["search", "metadata.description:null"]);
    assert_command_success(&output);
    assert_output_contains(&output, "143022");
}

// 特殊文字には現在対応してない
#[ignore]
#[test]
fn test_search_metadata_special_characters() {
    let context = TestContext::new();

    // Create a memo with special characters in metadata
    let memo_with_special_chars = r#"---


// Memo with special characters in metadata
    let memo_with_special_chars = r#"---
title: "Special: !@#$%^&*()_+-=[]{}|;':\",./<>?"
tags: ["@special-chars", "@unicode-test"]
unicode_field: "αβγδε ñáéíóú 🚀📝✅"
---

This memo has special characters in metadata."#;

    context.create_memo("2025-01/30/151545.md", memo_with_special_chars);

    let index_output = context.run_command(&["index"]);
    assert_command_success(&index_output);

    // Search should handle special characters
    let output = context.run_command(&["search", "metadata.unicode_field:🚀"]);
    assert_command_success(&output);
    assert_output_contains(&output, "151545");
}
