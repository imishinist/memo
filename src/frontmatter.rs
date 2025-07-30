use serde_yaml::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MemoContent {
    pub frontmatter: Option<HashMap<String, Value>>,
    pub content: String,
}

pub fn parse_memo_content(content: &str) -> Result<MemoContent, Box<dyn std::error::Error>> {
    // Check if content starts with frontmatter delimiter
    if !content.starts_with("---\n") {
        return Ok(MemoContent {
            frontmatter: None,
            content: content.to_string(),
        });
    }

    // Find the closing delimiter
    let content_after_first_delimiter = &content[4..]; // Skip "---\n"

    // Look for closing delimiter patterns
    if let Some(end_pos) = content_after_first_delimiter.find("\n---\n") {
        let yaml_content = &content_after_first_delimiter[..end_pos];
        let remaining_content = &content_after_first_delimiter[end_pos + 5..]; // Skip "\n---\n"

        // Parse YAML frontmatter
        let frontmatter: HashMap<String, Value> = if yaml_content.trim().is_empty() {
            HashMap::new()
        } else {
            serde_yaml::from_str(yaml_content)?
        };

        Ok(MemoContent {
            frontmatter: Some(frontmatter),
            content: remaining_content.to_string(),
        })
    } else if content_after_first_delimiter.starts_with("---\n") {
        // Handle case where frontmatter is empty: ---\n---\n
        let remaining_content = &content_after_first_delimiter[4..]; // Skip "---\n"
        Ok(MemoContent {
            frontmatter: Some(HashMap::new()),
            content: remaining_content.to_string(),
        })
    } else {
        // No closing delimiter found, treat as regular content
        Ok(MemoContent {
            frontmatter: None,
            content: content.to_string(),
        })
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
        };

        let formatted = format_memo_content(&memo);
        assert_eq!(formatted, "Just content");
    }
}
