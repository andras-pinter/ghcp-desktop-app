//! Parser for SKILL.md files (YAML frontmatter + markdown body).
//!
//! The SKILL.md standard uses YAML frontmatter delimited by `---` lines,
//! followed by a markdown body containing the skill instructions.
//! Spec: <https://agentskills.io/specification>

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parsed representation of a SKILL.md file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedSkillMd {
    /// Skill name (required, 1–64 chars, lowercase + hyphens).
    pub name: String,
    /// Short description (required, 1–1024 chars).
    pub description: String,
    /// SPDX license identifier.
    pub license: Option<String>,
    /// Compatibility hints (e.g., target platforms or tools).
    pub compatibility: Option<Vec<String>>,
    /// Arbitrary metadata key-value pairs.
    pub metadata: Option<HashMap<String, serde_norway::Value>>,
    /// Tools the skill is allowed to use.
    pub allowed_tools: Option<Vec<String>>,
    /// The markdown body (instructions).
    pub instructions: String,
}

/// YAML frontmatter structure (internal, for deserialization).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Frontmatter {
    name: String,
    description: String,
    license: Option<String>,
    compatibility: Option<Vec<String>>,
    metadata: Option<HashMap<String, serde_norway::Value>>,
    allowed_tools: Option<Vec<String>>,
}

/// Errors that can occur when parsing a SKILL.md file.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("missing YAML frontmatter (must start with ---)")]
    MissingFrontmatter,
    #[error("unclosed YAML frontmatter (missing closing ---)")]
    UnclosedFrontmatter,
    #[error("invalid YAML frontmatter: {0}")]
    InvalidYaml(#[from] serde_norway::Error),
    #[error("missing required field: {0}")]
    MissingField(String),
    #[error("invalid name: must be 1-64 characters, lowercase alphanumeric and hyphens")]
    InvalidName,
    #[error("description too long: max 1024 characters")]
    DescriptionTooLong,
}

/// Parse a SKILL.md string into a structured `ParsedSkillMd`.
///
/// The input must start with `---` followed by YAML frontmatter, a closing
/// `---`, and then the markdown body (instructions).
pub fn parse(input: &str) -> Result<ParsedSkillMd, ParseError> {
    let trimmed = input.trim_start();

    // Find frontmatter delimiters
    if !trimmed.starts_with("---") {
        return Err(ParseError::MissingFrontmatter);
    }

    let after_first = &trimmed[3..];
    let after_first = after_first.strip_prefix('\n').unwrap_or(
        after_first
            .strip_prefix("\r\n")
            .unwrap_or(after_first),
    );

    let closing = after_first.find("\n---");
    let closing_idx = closing.ok_or(ParseError::UnclosedFrontmatter)?;

    let yaml_str = &after_first[..closing_idx];
    let body_start = closing_idx + 4; // skip "\n---"
    let body = if body_start < after_first.len() {
        after_first[body_start..]
            .strip_prefix('\n')
            .unwrap_or(
                after_first[body_start..]
                    .strip_prefix("\r\n")
                    .unwrap_or(&after_first[body_start..]),
            )
    } else {
        ""
    };

    let fm: Frontmatter = serde_norway::from_str(yaml_str)?;

    // Validate required fields
    if fm.name.is_empty() || fm.name.len() > 64 {
        return Err(ParseError::InvalidName);
    }
    if !fm
        .name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(ParseError::InvalidName);
    }
    if fm.description.is_empty() {
        return Err(ParseError::MissingField("description".to_string()));
    }
    if fm.description.len() > 1024 {
        return Err(ParseError::DescriptionTooLong);
    }

    Ok(ParsedSkillMd {
        name: fm.name,
        description: fm.description,
        license: fm.license,
        compatibility: fm.compatibility,
        metadata: fm.metadata,
        allowed_tools: fm.allowed_tools,
        instructions: body.trim().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_SKILL: &str = r#"---
name: code-review
description: Reviews code for quality and best practices
license: MIT
compatibility:
  - github-copilot
  - cursor
allowed-tools:
  - web_search
  - file_read
metadata:
  author: test-author
  version: "1.0"
---

You are a code review assistant. When reviewing code:

1. Check for bugs and logic errors
2. Suggest improvements for readability
3. Verify error handling is complete

Always be constructive and explain your reasoning.
"#;

    #[test]
    fn test_parse_valid_skill() {
        let result = parse(VALID_SKILL).unwrap();
        assert_eq!(result.name, "code-review");
        assert_eq!(result.description, "Reviews code for quality and best practices");
        assert_eq!(result.license, Some("MIT".to_string()));
        assert_eq!(
            result.compatibility,
            Some(vec![
                "github-copilot".to_string(),
                "cursor".to_string()
            ])
        );
        assert_eq!(
            result.allowed_tools,
            Some(vec!["web_search".to_string(), "file_read".to_string()])
        );
        assert!(result.instructions.contains("code review assistant"));
        assert!(result.instructions.contains("Always be constructive"));

        let metadata = result.metadata.unwrap();
        assert_eq!(
            metadata.get("author"),
            Some(&serde_norway::Value::String("test-author".to_string()))
        );
    }

    #[test]
    fn test_parse_minimal_skill() {
        let input = "---\nname: my-skill\ndescription: A minimal skill\n---\nDo the thing.\n";
        let result = parse(input).unwrap();
        assert_eq!(result.name, "my-skill");
        assert_eq!(result.description, "A minimal skill");
        assert_eq!(result.instructions, "Do the thing.");
        assert!(result.license.is_none());
        assert!(result.compatibility.is_none());
        assert!(result.allowed_tools.is_none());
        assert!(result.metadata.is_none());
    }

    #[test]
    fn test_parse_empty_body() {
        let input = "---\nname: empty-body\ndescription: No instructions\n---\n";
        let result = parse(input).unwrap();
        assert_eq!(result.instructions, "");
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let input = "# Just a markdown file\nNo frontmatter here.";
        let err = parse(input).unwrap_err();
        assert!(matches!(err, ParseError::MissingFrontmatter));
    }

    #[test]
    fn test_parse_unclosed_frontmatter() {
        let input = "---\nname: broken\ndescription: Missing closing delimiter\n";
        let err = parse(input).unwrap_err();
        assert!(matches!(err, ParseError::UnclosedFrontmatter));
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let input = "---\n: invalid yaml: [broken\n---\nBody\n";
        let err = parse(input).unwrap_err();
        assert!(matches!(err, ParseError::InvalidYaml(_)));
    }

    #[test]
    fn test_parse_invalid_name_uppercase() {
        let input = "---\nname: MySkill\ndescription: Bad name\n---\nBody\n";
        let err = parse(input).unwrap_err();
        assert!(matches!(err, ParseError::InvalidName));
    }

    #[test]
    fn test_parse_invalid_name_spaces() {
        let input = "---\nname: my skill\ndescription: Bad name\n---\nBody\n";
        let err = parse(input).unwrap_err();
        assert!(matches!(err, ParseError::InvalidName));
    }

    #[test]
    fn test_parse_empty_name() {
        let input = "---\nname: \"\"\ndescription: Empty name\n---\nBody\n";
        let err = parse(input).unwrap_err();
        assert!(matches!(err, ParseError::InvalidName));
    }

    #[test]
    fn test_parse_name_too_long() {
        let long_name = "a".repeat(65);
        let input = format!("---\nname: {long_name}\ndescription: Too long\n---\nBody\n");
        let err = parse(&input).unwrap_err();
        assert!(matches!(err, ParseError::InvalidName));
    }

    #[test]
    fn test_parse_description_too_long() {
        let long_desc = "a".repeat(1025);
        let input = format!("---\nname: valid-name\ndescription: {long_desc}\n---\nBody\n");
        let err = parse(&input).unwrap_err();
        assert!(matches!(err, ParseError::DescriptionTooLong));
    }

    #[test]
    fn test_parse_missing_description() {
        let input = "---\nname: no-desc\n---\nBody\n";
        let err = parse(input).unwrap_err();
        // serde_norway will report missing field
        assert!(matches!(err, ParseError::InvalidYaml(_)));
    }

    #[test]
    fn test_parse_multiline_instructions() {
        let input = "---\nname: multi-line\ndescription: Has multiple lines\n---\n\nLine one.\n\nLine two.\n\n```rust\nfn main() {}\n```\n\nLine three.\n";
        let result = parse(input).unwrap();
        assert!(result.instructions.contains("Line one."));
        assert!(result.instructions.contains("Line two."));
        assert!(result.instructions.contains("```rust"));
        assert!(result.instructions.contains("Line three."));
    }

    #[test]
    fn test_parse_with_leading_whitespace() {
        let input = "  \n---\nname: whitespace\ndescription: Leading whitespace\n---\nBody\n";
        let result = parse(input).unwrap();
        assert_eq!(result.name, "whitespace");
    }

    #[test]
    fn test_parse_preserves_code_blocks_in_body() {
        let input = "---\nname: code-skill\ndescription: Has code\n---\n\nUse this pattern:\n\n```python\ndef hello():\n    print(\"hello\")\n```\n";
        let result = parse(input).unwrap();
        assert!(result.instructions.contains("```python"));
        assert!(result.instructions.contains("def hello():"));
    }
}
