use serde_yaml::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MemoContent {
    pub content: String,

    pub front_matter: Option<HashMap<String, Value>>,
    pub front_matter_error: Option<String>,
}

pub fn parse_memo_content(content: &str) -> MemoContent {
    let delimiter = "---\n";

    // Check if content starts with front matter delimiter
    if !content.starts_with(delimiter) {
        return MemoContent {
            front_matter: None,
            content: content.to_string(),
            front_matter_error: None,
        };
    }

    // Find the closing delimiter
    let content_after_first_delimiter = &content[delimiter.len()..]; // Skip "---\n"

    // Look for closing delimiter patterns
    let first_delimiter_end = content_after_first_delimiter.find(delimiter);
    match first_delimiter_end {
        None => MemoContent {
            front_matter: None,
            content: content.to_string(),
            front_matter_error: None,
        },
        Some(end_pos) => {
            let yaml_content = &content_after_first_delimiter[..end_pos];
            let remaining_content = &content_after_first_delimiter[end_pos + delimiter.len()..]; // Skip "\n---\n"

            // Parse YAML front matter
            let (front_matter, error) = if yaml_content.trim().is_empty() {
                (Some(HashMap::new()), None)
            } else {
                match serde_yaml::from_str::<HashMap<String, Value>>(yaml_content) {
                    Ok(fm) => (Some(fm), None),
                    Err(e) => (None, Some(format!("YAML parse error: {}", e))),
                }
            };

            MemoContent {
                front_matter,
                content: remaining_content.to_string(),
                front_matter_error: error,
            }
        }
    }
}

// Function used by tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_memo_content_with_front_matter() {
        let content = r#"---
title: Test
tags: ['@example', '@test']
---
Content here"#;

        let memo = parse_memo_content(content);

        assert!(memo.front_matter.is_some());

        let front_matter = memo.front_matter.unwrap();
        assert_eq!(
            front_matter.get("title").unwrap(),
            &Value::String("Test".to_string())
        );
        assert_eq!(
            front_matter.get("tags").unwrap(),
            &Value::Sequence(vec![
                Value::String("@example".to_string()),
                Value::String("@test".to_string())
            ])
        );
        assert_eq!(memo.content, "Content here");
        assert!(memo.front_matter_error.is_none());
    }

    #[test]
    fn test_parse_memo_content_empty_front_matter() {
        let content = r#"---
---
Content here"#;

        let memo = parse_memo_content(content);

        assert!(memo.front_matter.is_some());
        assert_eq!(memo.front_matter.unwrap().len(), 0);
        assert_eq!(memo.content, "Content here");
        assert!(memo.front_matter_error.is_none());
    }
}
