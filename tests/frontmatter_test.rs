use memo::frontmatter::parse_memo_content;
use serde_yaml::Value;

#[test]
fn test_parse_memo_with_frontmatter() {
    let content = r#"---
title: Test Memo
tags: ["@tag1", "@tag2"]
created_at: "2025-07-30 13:00:00"
priority: 1
---

This is the memo content.

Some more content here."#;

    let result = parse_memo_content(content);

    assert!(result.frontmatter.is_some());
    assert!(result.frontmatter_error.is_none());
    let frontmatter = result.frontmatter.unwrap();

    assert_eq!(
        frontmatter.get("title").unwrap(),
        &Value::String("Test Memo".to_string())
    );
    assert_eq!(
        frontmatter.get("created_at").unwrap(),
        &Value::String("2025-07-30 13:00:00".to_string())
    );
    assert_eq!(
        frontmatter.get("priority").unwrap(),
        &Value::Number(serde_yaml::Number::from(1))
    );

    if let Value::Sequence(tags) = frontmatter.get("tags").unwrap() {
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0], Value::String("@tag1".to_string()));
        assert_eq!(tags[1], Value::String("@tag2".to_string()));
    } else {
        panic!("tags should be a sequence");
    }

    assert_eq!(
        result.content.trim(),
        "This is the memo content.\n\nSome more content here."
    );
}

#[test]
fn test_parse_memo_without_frontmatter() {
    let content = r#"This is a simple memo without frontmatter.

Just plain text content."#;

    let result = parse_memo_content(content);

    assert!(result.frontmatter.is_none());
    assert!(result.frontmatter_error.is_none());
    assert_eq!(result.content, content);
}

#[test]
fn test_parse_memo_with_empty_frontmatter() {
    let content = r#"---
---

This memo has empty frontmatter."#;

    let result = parse_memo_content(content);

    assert!(result.frontmatter.is_some());
    assert!(result.frontmatter_error.is_none());
    let frontmatter = result.frontmatter.unwrap();
    assert!(frontmatter.is_empty());
    assert_eq!(result.content.trim(), "This memo has empty frontmatter.");
}

#[test]
fn test_parse_memo_with_invalid_yaml() {
    let content = r#"---
title: Test Memo
invalid: [unclosed array
---

This memo has invalid YAML."#;

    let result = parse_memo_content(content);
    assert!(result.frontmatter_error.is_some());
    assert!(result.frontmatter.is_none());
}

#[test]
fn test_parse_memo_with_arbitrary_fields() {
    let content = r#"---
custom_field: "custom value"
number_field: 42
boolean_field: true
nested:
  sub_field: "nested value"
  sub_number: 123
---

Memo with arbitrary fields."#;

    let result = parse_memo_content(content);

    assert!(result.frontmatter.is_some());
    assert!(result.frontmatter_error.is_none());
    let frontmatter = result.frontmatter.unwrap();

    assert_eq!(
        frontmatter.get("custom_field").unwrap(),
        &Value::String("custom value".to_string())
    );
    assert_eq!(
        frontmatter.get("number_field").unwrap(),
        &Value::Number(serde_yaml::Number::from(42))
    );
    assert_eq!(
        frontmatter.get("boolean_field").unwrap(),
        &Value::Bool(true)
    );

    if let Value::Mapping(nested) = frontmatter.get("nested").unwrap() {
        assert_eq!(
            nested.get(&Value::String("sub_field".to_string())).unwrap(),
            &Value::String("nested value".to_string())
        );
        assert_eq!(
            nested
                .get(&Value::String("sub_number".to_string()))
                .unwrap(),
            &Value::Number(serde_yaml::Number::from(123))
        );
    } else {
        panic!("nested should be a mapping");
    }
}

#[test]
fn test_parse_memo_frontmatter_only_at_start() {
    let content = r#"This is content before frontmatter.

---
title: This should not be parsed as frontmatter
---

More content."#;

    let result = parse_memo_content(content);

    assert!(result.frontmatter.is_none());
    assert!(result.frontmatter_error.is_none());
    assert_eq!(result.content, content);
}

// 新しい構造に対応したテスト
#[test]
fn test_memo_file_with_frontmatter() {
    use memo::{MemoContext, MemoFile};
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let memo_dir = temp_dir.path().join("memo");
    fs::create_dir_all(&memo_dir).unwrap();

    let _context = MemoContext {
        memo_dir: memo_dir.clone(),
        editor: "echo".to_string(),
    };

    let content = r#"---
title: Test Memo
priority: 1
---

This is a test memo."#;

    let memo_path = memo_dir.join("2025-01/30/143022.md");
    let memo = MemoFile::create(&memo_path, content.to_string()).unwrap();

    assert_eq!(memo.id, "2025-01/30/143022");
    assert!(memo.frontmatter.is_some());
    assert!(memo.frontmatter_error.is_none());

    let frontmatter = memo.frontmatter.unwrap();
    assert_eq!(
        frontmatter.get("title").unwrap(),
        &Value::String("Test Memo".to_string())
    );
    assert_eq!(
        frontmatter.get("priority").unwrap(),
        &Value::Number(serde_yaml::Number::from(1))
    );
}
