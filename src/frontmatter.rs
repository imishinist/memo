use serde_yaml::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MemoContent {
    pub frontmatter: Option<HashMap<String, Value>>,
    pub content: String,
    pub frontmatter_error: Option<String>,
}

pub fn parse_memo_content(content: &str) -> MemoContent {
    // Check if content starts with frontmatter delimiter
    if !content.starts_with("---\n") {
        return MemoContent {
            frontmatter: None,
            content: content.to_string(),
            frontmatter_error: None,
        };
    }

    // Find the closing delimiter
    let content_after_first_delimiter = &content[4..]; // Skip "---\n"

    // Look for closing delimiter patterns
    if let Some(end_pos) = content_after_first_delimiter.find("\n---\n") {
        let yaml_content = &content_after_first_delimiter[..end_pos];
        let remaining_content = &content_after_first_delimiter[end_pos + 5..]; // Skip "\n---\n"

        // Parse YAML frontmatter
        let (frontmatter, error) = if yaml_content.trim().is_empty() {
            (Some(HashMap::new()), None)
        } else {
            match serde_yaml::from_str::<HashMap<String, Value>>(yaml_content) {
                Ok(fm) => (Some(fm), None),
                Err(e) => (None, Some(format!("YAML parse error: {}", e))),
            }
        };

        MemoContent {
            frontmatter,
            content: remaining_content.to_string(),
            frontmatter_error: error,
        }
    } else if content_after_first_delimiter.starts_with("---\n") {
        // Handle case where frontmatter is empty: ---\n---\n
        let remaining_content = &content_after_first_delimiter[4..]; // Skip "---\n"
        MemoContent {
            frontmatter: Some(HashMap::new()),
            content: remaining_content.to_string(),
            frontmatter_error: None,
        }
    } else {
        // No closing delimiter found, treat as regular content
        MemoContent {
            frontmatter: None,
            content: content.to_string(),
            frontmatter_error: None,
        }
    }
}

// Function used by tests
#[cfg(test)]
pub fn format_memo_content(memo: &MemoContent) -> String {
    match &memo.frontmatter {
        Some(frontmatter) if !frontmatter.is_empty() => {
            let yaml_str = serde_yaml::to_string(frontmatter).unwrap_or_default();
            format!("---\n{}---\n{}", yaml_str, memo.content)
        }
        _ => memo.content.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_memo_content_with_frontmatter() {
        let mut frontmatter = HashMap::new();
        frontmatter.insert("title".to_string(), Value::String("Test".to_string()));

        let memo = MemoContent {
            frontmatter: Some(frontmatter),
            content: "Content here".to_string(),
            frontmatter_error: None,
        };

        let formatted = format_memo_content(&memo);
        assert!(formatted.starts_with("---\n"));
        assert!(formatted.contains("title: Test"));
        assert!(formatted.ends_with("---\nContent here"));
    }

    #[test]
    fn test_format_memo_content_without_frontmatter() {
        let memo = MemoContent {
            frontmatter: None,
            content: "Just content".to_string(),
            frontmatter_error: None,
        };

        let formatted = format_memo_content(&memo);
        assert_eq!(formatted, "Just content");
    }
}
