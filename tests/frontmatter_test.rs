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
