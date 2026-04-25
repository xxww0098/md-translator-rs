use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::types::MarkdownOptions;

static FRONTMATTER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)^---\n.*?\n---").expect("valid regex"));
static CODE_FENCE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)```.*?```").expect("valid regex"));
static LATEX_BLOCK_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)\$\$.*?\$\$").expect("valid regex"));
static INLINE_CODE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"`([^`]+?)`").expect("valid regex"));
static INLINE_LATEX_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$([^\$]+?)\$").expect("valid regex"));
static COMMENT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<!--(?s:.*?)-->").expect("valid regex"));
static SELF_CLOSING_HTML_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<([a-zA-Z][a-zA-Z0-9-]*)\s*[^>]*?/>").expect("valid regex"));
static HTML_END_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"</([a-zA-Z][a-zA-Z0-9-]*)>").expect("valid regex"));
static HTML_START_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<([a-zA-Z][a-zA-Z0-9-]*)(?:\s+[^>]*)?>").expect("valid regex"));
static IMAGE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(!\[)(.*?)(\]\(.*?\))").expect("valid regex"));
static LINK_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\[)(.*?)(\]\(.*?\))").expect("valid regex"));
static HEADING_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(#{1,6}\s)(.*)$").expect("valid regex"));
static LIST_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\s*(?:[-*]|\d+\.)\s+)(.*)$").expect("valid regex"));
static BLOCKQUOTE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(>\s)(.*)$").expect("valid regex"));

#[derive(Debug, Clone)]
pub struct ProtectedMarkdown {
    pub text: String,
    pub placeholders: HashMap<String, String>,
}

impl ProtectedMarkdown {
    pub fn has_placeholders(&self) -> bool {
        !self.placeholders.is_empty()
    }
}

pub fn protect_plain_text(input: &str) -> ProtectedMarkdown {
    ProtectedMarkdown {
        text: input.to_string(),
        placeholders: HashMap::new(),
    }
}

pub fn protect_markdown(input: &str, options: &MarkdownOptions) -> ProtectedMarkdown {
    let mut placeholders = HashMap::new();
    let mut counter: usize = 100;
    let mut full_text = input.to_string();

    if !options.translate_frontmatter {
        full_text = replace_with_placeholders(
            &full_text,
            &FRONTMATTER_RE,
            "FRONTMATTER",
            &mut counter,
            &mut placeholders,
        );
    }
    if !options.translate_multiline_code {
        full_text = replace_with_placeholders(
            &full_text,
            &CODE_FENCE_RE,
            "MULTILINE_CODE",
            &mut counter,
            &mut placeholders,
        );
    }
    if !options.translate_latex {
        full_text = replace_with_placeholders(
            &full_text,
            &LATEX_BLOCK_RE,
            "LATEX_BLOCK",
            &mut counter,
            &mut placeholders,
        );
    }

    let mut lines_out = Vec::new();
    for line in full_text.split('\n') {
        let mut modified = line.to_string();

        modified = replace_with_placeholders(
            &modified,
            &INLINE_CODE_RE,
            "CODE",
            &mut counter,
            &mut placeholders,
        );

        if !options.translate_latex {
            modified = INLINE_LATEX_RE
                .replace_all(&modified, |caps: &regex::Captures<'_>| {
                    let content = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                    if is_probable_currency(content) {
                        caps.get(0)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default()
                    } else {
                        allocate_placeholder(
                            caps.get(0).map(|m| m.as_str()).unwrap_or_default(),
                            "LATEX_INLINE",
                            &mut counter,
                            &mut placeholders,
                        )
                    }
                })
                .to_string();
        }

        modified = replace_with_placeholders(
            &modified,
            &COMMENT_RE,
            "HTML",
            &mut counter,
            &mut placeholders,
        );
        modified = replace_with_placeholders(
            &modified,
            &SELF_CLOSING_HTML_RE,
            "HTML",
            &mut counter,
            &mut placeholders,
        );
        modified = replace_with_placeholders(
            &modified,
            &HTML_END_RE,
            "HTML",
            &mut counter,
            &mut placeholders,
        );
        modified = replace_with_placeholders(
            &modified,
            &HTML_START_RE,
            "HTML",
            &mut counter,
            &mut placeholders,
        );

        modified = IMAGE_RE
            .replace_all(&modified, |caps: &regex::Captures<'_>| {
                let full = caps.get(0).map(|m| m.as_str()).unwrap_or_default();
                let prefix = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                let content = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
                let suffix = caps.get(3).map(|m| m.as_str()).unwrap_or_default();

                if content.trim().is_empty() {
                    return allocate_placeholder(full, "LINK", &mut counter, &mut placeholders);
                }

                let pre = allocate_placeholder(prefix, "LINK_PRE", &mut counter, &mut placeholders);
                let suf = allocate_placeholder(suffix, "LINK_SUF", &mut counter, &mut placeholders);
                format!("{pre}{content}{suf}")
            })
            .to_string();

        modified = LINK_RE
            .replace_all(&modified, |caps: &regex::Captures<'_>| {
                let full = caps.get(0).map(|m| m.as_str()).unwrap_or_default();
                let prefix = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                let content = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
                let suffix = caps.get(3).map(|m| m.as_str()).unwrap_or_default();

                if options.translate_link_text {
                    let pre =
                        allocate_placeholder(prefix, "LINK_PRE", &mut counter, &mut placeholders);
                    let suf =
                        allocate_placeholder(suffix, "LINK_SUF", &mut counter, &mut placeholders);
                    format!("{pre}{content}{suf}")
                } else {
                    allocate_placeholder(full, "LINK", &mut counter, &mut placeholders)
                }
            })
            .to_string();

        modified = HEADING_RE
            .replace(&modified, |caps: &regex::Captures<'_>| {
                let prefix = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                let content = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
                let head = allocate_placeholder(prefix, "HEADING", &mut counter, &mut placeholders);
                format!("{head}{content}")
            })
            .to_string();

        modified = LIST_RE
            .replace(&modified, |caps: &regex::Captures<'_>| {
                let prefix = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                let content = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
                let list = allocate_placeholder(prefix, "LIST", &mut counter, &mut placeholders);
                format!("{list}{content}")
            })
            .to_string();

        modified = BLOCKQUOTE_RE
            .replace(&modified, |caps: &regex::Captures<'_>| {
                let prefix = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                let content = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
                let quote =
                    allocate_placeholder(prefix, "BLOCKQUOTE", &mut counter, &mut placeholders);
                format!("{quote}{content}")
            })
            .to_string();

        lines_out.push(modified);
    }

    ProtectedMarkdown {
        text: lines_out.join("\n"),
        placeholders,
    }
}

pub fn restore_markdown(translated: &str, protected: &ProtectedMarkdown) -> String {
    let mut output = translated.to_string();
    for (placeholder, value) in &protected.placeholders {
        output = output.replace(placeholder, value);
    }
    output
}

fn replace_with_placeholders(
    input: &str,
    regex: &Regex,
    tag: &str,
    counter: &mut usize,
    placeholders: &mut HashMap<String, String>,
) -> String {
    regex
        .replace_all(input, |caps: &regex::Captures<'_>| {
            let raw = caps.get(0).map(|m| m.as_str()).unwrap_or_default();
            allocate_placeholder(raw, tag, counter, placeholders)
        })
        .to_string()
}

fn allocate_placeholder(
    raw: &str,
    tag: &str,
    counter: &mut usize,
    placeholders: &mut HashMap<String, String>,
) -> String {
    let key = format!("__MDT_{tag}_{counter}_KEEP__");
    *counter += 1;
    placeholders.insert(key.clone(), raw.to_string());
    key
}

pub fn validate_placeholders(translated: &str, protected: &ProtectedMarkdown) -> bool {
    protected.placeholders.keys().all(|placeholder| {
        translated.matches(placeholder).count() == protected.text.matches(placeholder).count()
    })
}

fn is_probable_currency(content: &str) -> bool {
    content
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == ',' || c.is_ascii_whitespace())
        && !content.contains('\\')
}

#[cfg(test)]
mod tests {
    use crate::types::MarkdownOptions;

    use super::{protect_markdown, restore_markdown, validate_placeholders};

    #[test]
    fn round_trip_markdown_preserves_code_fence() {
        let input = "```rust\nlet x = 1;\n```\n# Hello";
        let opts = MarkdownOptions::default();
        let protected = protect_markdown(input, &opts);
        let restored = restore_markdown(&protected.text, &protected);
        assert_eq!(restored, input);
    }

    #[test]
    fn placeholder_validation_detects_missing_tokens() {
        let input = "# Hello\n```rs\nlet x = 1;\n```";
        let opts = MarkdownOptions::default();
        let protected = protect_markdown(input, &opts);
        assert!(validate_placeholders(&protected.text, &protected));

        let invalid = protected
            .text
            .replace("__MDT_MULTILINE_CODE_", "__MDT_MUTATED_CODE_");
        assert!(!validate_placeholders(&invalid, &protected));
    }
}
