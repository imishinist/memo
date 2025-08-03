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

#[test]
fn test_index_with_complex_metadata_structures() {
    let context = TestContext::new();

    // Create memo with complex nested metadata
    let complex_metadata_memo = r#"---
title: Complex Metadata Structure
author:
  name: John Doe
  email: john@example.com
  profile:
    department: Engineering
    level: Senior
project:
  name: Memo System
  status: active
  milestones:
    - name: "Phase 1"
      completed: true
      date: "2025-01-15"
    - name: "Phase 2"
      completed: false
      date: "2025-02-15"
tags: ["@project", "@engineering"]
metrics:
  priority: 1
  complexity: 8.5
  estimated_hours: 40
settings:
  public: false
  notifications: true
  categories: ["development", "planning"]
---

This memo contains deeply nested metadata structures to test indexing capabilities."#;

    context.create_memo("2025-01/30/143022.md", complex_metadata_memo);

    let output = context.run_command(&["index"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Indexing 1 memos");
    assert_output_contains(&output, "Search index built successfully");
}

#[test]
fn test_index_with_various_data_types() {
    let context = TestContext::new();

    // Create memos with different data types in metadata
    let numeric_metadata = r#"---
title: Numeric Data Test
integer_field: 42
float_field: 3.14159
negative_int: -100
large_number: 1234567890
zero_value: 0
---

Testing numeric data types in metadata."#;

    let boolean_metadata = r#"---
title: Boolean Data Test
active: true
completed: false
enabled: true
archived: false
---

Testing boolean data types in metadata."#;

    let array_metadata = r#"---
title: Array Data Test
tags: ["@test", "@array", "@metadata"]
numbers: [1, 2, 3, 4, 5]
mixed_array: ["string", 42, true, null]
empty_array: []
nested_arrays: [[1, 2], [3, 4]]
---

Testing array data types in metadata."#;

    context.create_memo("2025-01/30/143022.md", numeric_metadata);
    context.create_memo("2025-01/30/151545.md", boolean_metadata);
    context.create_memo("2025-01/30/160000.md", array_metadata);

    let output = context.run_command(&["index"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Indexing 3 memos");
    assert_output_contains(&output, "Search index built successfully");
}

#[test]
fn test_index_with_metadata_edge_cases() {
    let context = TestContext::new();

    // Memo with null and empty values
    let edge_case_metadata = r#"---
title: Edge Cases Test
null_field: null
empty_string: ""
empty_object: {}
empty_array: []
whitespace_string: "   "
special_chars: "!@#$%^&*()_+-=[]{}|;':\",./<>?"
unicode_text: "αβγδε ñáéíóú 🚀📝✅"
---

Testing edge cases in metadata values."#;

    // Memo with very large metadata
    let large_metadata = format!(
        r#"---
title: Large Metadata Test
description: "{}"
large_array: {}
tags: ["@large", "@test"]
---

Testing large metadata structures."#,
        "A".repeat(1000),
        serde_json::to_string(&(0..100).collect::<Vec<i32>>()).unwrap()
    );

    context.create_memo("2025-01/30/143022.md", &edge_case_metadata);
    context.create_memo("2025-01/30/151545.md", &large_metadata);

    let output = context.run_command(&["index"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Indexing 2 memos");
    assert_output_contains(&output, "Search index built successfully");
}

#[test]
fn test_index_with_malformed_metadata() {
    let context = TestContext::new();

    // Memo with broken YAML frontmatter
    let broken_frontmatter = r#"---
title: Broken Metadata Test
invalid_yaml: [unclosed array
another_field: "valid value"
---

This memo has broken frontmatter that should be handled gracefully."#;

    // Memo with valid frontmatter for comparison
    let valid_frontmatter = r#"---
title: Valid Metadata Test
tags: ["@valid", "@test"]
---

This memo has valid frontmatter."#;

    context.create_memo("2025-01/30/143022.md", broken_frontmatter);
    context.create_memo("2025-01/30/151545.md", valid_frontmatter);

    let output = context.run_command(&["index"]);

    // Indexing should succeed even with broken frontmatter
    assert_command_success(&output);
    assert_output_contains(&output, "Indexing 2 memos");
    assert_output_contains(&output, "Search index built successfully");
}

#[test]
fn test_index_metadata_consistency_across_rebuilds() {
    let context = TestContext::new();

    // Create memo with metadata
    let metadata_memo = TestMemoTemplates::with_custom_frontmatter(
        "Consistency Test",
        &["@consistency", "@test"],
        "Testing metadata consistency across index rebuilds.",
    );
    context.create_memo("2025-01/30/143022.md", &metadata_memo);

    // Build index first time
    let output1 = context.run_command(&["index"]);
    assert_command_success(&output1);

    // Verify search works
    let search_output1 = context.run_command(&["search", "Consistency Test"]);
    assert_command_success(&search_output1);
    assert_output_contains(&search_output1, "143022");

    // Rebuild index
    let output2 = context.run_command(&["index"]);
    assert_command_success(&output2);

    // Verify search still works after rebuild
    let search_output2 = context.run_command(&["search", "Consistency Test"]);
    assert_command_success(&search_output2);
    assert_output_contains(&search_output2, "143022");
}
