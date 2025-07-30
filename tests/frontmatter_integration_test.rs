use memo::frontmatter::parse_memo_content;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_frontmatter_parsing_consistency() {
    let original_content = r#"---
title: Test Memo
tags: ["@tag1", "@tag2"]
priority: 1
---

This is the memo content.

Some more content here."#;

    // Parse the content multiple times to verify consistency
    let parsed1 = parse_memo_content(original_content).unwrap();
    let parsed2 = parse_memo_content(original_content).unwrap();

    assert!(parsed1.frontmatter.is_some());
    assert!(parsed2.frontmatter.is_some());
    assert_eq!(parsed1.content.trim(), parsed2.content.trim());

    let frontmatter = parsed1.frontmatter.unwrap();
    assert_eq!(
        frontmatter.get("title").unwrap().as_str().unwrap(),
        "Test Memo"
    );
    assert_eq!(frontmatter.get("priority").unwrap().as_i64().unwrap(), 1);
}

#[test]
fn test_memo_file_operations_with_frontmatter() {
    let temp_dir = TempDir::new().unwrap();
    let memo_file = temp_dir.path().join("test_memo.md");

    let content_with_frontmatter = r#"---
title: File Test Memo
author: Test User
---

This is a test memo saved to file."#;

    // Write memo with frontmatter to file
    fs::write(&memo_file, content_with_frontmatter).unwrap();

    // Read and parse the file
    let file_content = fs::read_to_string(&memo_file).unwrap();
    let parsed = parse_memo_content(&file_content).unwrap();

    assert!(parsed.frontmatter.is_some());
    let frontmatter = parsed.frontmatter.unwrap();
    assert_eq!(
        frontmatter.get("title").unwrap().as_str().unwrap(),
        "File Test Memo"
    );
    assert_eq!(
        frontmatter.get("author").unwrap().as_str().unwrap(),
        "Test User"
    );
    assert_eq!(parsed.content.trim(), "This is a test memo saved to file.");
}

#[test]
fn test_memo_without_frontmatter_compatibility() {
    let temp_dir = TempDir::new().unwrap();
    let memo_file = temp_dir.path().join("simple_memo.md");

    let simple_content = "This is a simple memo without any frontmatter.

Just plain text content.";

    // Write simple memo to file
    fs::write(&memo_file, simple_content).unwrap();

    // Read and parse the file
    let file_content = fs::read_to_string(&memo_file).unwrap();
    let parsed = parse_memo_content(&file_content).unwrap();

    assert!(parsed.frontmatter.is_none());
    assert_eq!(parsed.content, simple_content);
}
