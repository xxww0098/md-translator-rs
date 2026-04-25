use std::collections::HashMap;
use std::fmt;

use comrak::format_commonmark;
use comrak::nodes::{AstNode, NodeValue};

use crate::types::MarkdownOptions;

use super::extractor::TranslatableNodeKind;
use super::parser::MarkdownAst;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    MissingTranslation {
        id: usize,
    },
    PlaceholderMismatch {
        id: usize,
    },
    UnsupportedStructure {
        id: usize,
        kind: TranslatableNodeKind,
    },
    RenderIo {
        message: String,
    },
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::MissingTranslation { id } => {
                write!(f, "missing translated content for AST node id={id}")
            }
            RenderError::PlaceholderMismatch { id } => {
                write!(
                    f,
                    "translated content mutated protected placeholders for AST node id={id}"
                )
            }
            RenderError::UnsupportedStructure { id, kind } => {
                write!(
                    f,
                    "AST node id={id} with kind {kind:?} cannot be safely replaced"
                )
            }
            RenderError::RenderIo { message } => write!(f, "failed to render markdown: {message}"),
        }
    }
}

impl std::error::Error for RenderError {}

#[allow(clippy::result_large_err)]
pub fn render_markdown(ast: &MarkdownAst) -> Result<String, RenderError> {
    ast.with_root(|root| {
        let mut out = Vec::new();
        format_commonmark(root, &ast.options, &mut out).map_err(|err| RenderError::RenderIo {
            message: err.to_string(),
        })?;
        String::from_utf8(out).map_err(|err| RenderError::RenderIo {
            message: err.to_string(),
        })
    })
}

#[allow(clippy::result_large_err)]
pub fn replace_translated_nodes_and_render(
    ast: &MarkdownAst,
    options: &MarkdownOptions,
    translated: &HashMap<usize, String>,
) -> Result<String, RenderError> {
    ast.with_root(|root| {
        let mut next_id = 1usize;
        replace_walk(root, options, translated, &mut next_id)?;

        let mut out = Vec::new();
        format_commonmark(root, &ast.options, &mut out).map_err(|err| RenderError::RenderIo {
            message: err.to_string(),
        })?;
        String::from_utf8(out).map_err(|err| RenderError::RenderIo {
            message: err.to_string(),
        })
    })
}

fn replace_walk<'a>(
    node: &'a AstNode<'a>,
    options: &MarkdownOptions,
    translated: &HashMap<usize, String>,
    next_id: &mut usize,
) -> Result<(), RenderError> {
    let node_value = node.data.borrow().value.clone();
    match node_value {
        NodeValue::Paragraph if container_has_replaceable_text(node, options) => {
            replace_container(
                node,
                TranslatableNodeKind::Paragraph,
                options,
                translated,
                next_id,
            )?;
        }
        NodeValue::Heading(_) if container_has_replaceable_text(node, options) => {
            replace_container(
                node,
                TranslatableNodeKind::Heading,
                options,
                translated,
                next_id,
            )?;
        }
        NodeValue::Item(_) if container_has_replaceable_text(node, options) => {
            replace_container(
                node,
                TranslatableNodeKind::ListItem,
                options,
                translated,
                next_id,
            )?;
        }
        NodeValue::BlockQuote if container_has_replaceable_text(node, options) => {
            replace_container(
                node,
                TranslatableNodeKind::BlockQuote,
                options,
                translated,
                next_id,
            )?;
        }
        NodeValue::Link(_) if options.translate_link_text && node_has_replaceable_text(node) => {
            replace_leaf(node, TranslatableNodeKind::LinkText, translated, next_id)?;
        }
        NodeValue::Image(_) if node_has_replaceable_text(node) => {
            replace_leaf(
                node,
                TranslatableNodeKind::ImageAltText,
                translated,
                next_id,
            )?;
        }
        NodeValue::FrontMatter(raw) if options.translate_frontmatter => {
            let (updated, consumed) = replace_frontmatter(&raw, translated, *next_id)?;
            if consumed > 0 {
                node.data.borrow_mut().value = NodeValue::FrontMatter(updated);
                *next_id += consumed;
            }
            return Ok(());
        }
        NodeValue::CodeBlock(_)
        | NodeValue::Code(_)
        | NodeValue::HtmlBlock(_)
        | NodeValue::HtmlInline(_)
        | NodeValue::Math(_) => return Ok(()),
        _ => {}
    }

    for child in node.children() {
        replace_walk(child, options, translated, next_id)?;
    }

    Ok(())
}

fn replace_container<'a>(
    node: &'a AstNode<'a>,
    kind: TranslatableNodeKind,
    options: &MarkdownOptions,
    translated: &HashMap<usize, String>,
    next_id: &mut usize,
) -> Result<(), RenderError> {
    let id = *next_id;
    let replacement = translated
        .get(&id)
        .ok_or(RenderError::MissingTranslation { id })?;
    validate_placeholders(
        id,
        &collect_container_placeholder_text(node, options),
        replacement,
    )?;

    if !replace_first_text_descendant(node, options, replacement, true) {
        return Err(RenderError::UnsupportedStructure { id, kind });
    }

    *next_id += 1;
    Ok(())
}

fn replace_leaf<'a>(
    node: &'a AstNode<'a>,
    kind: TranslatableNodeKind,
    translated: &HashMap<usize, String>,
    next_id: &mut usize,
) -> Result<(), RenderError> {
    let id = *next_id;
    let replacement = translated
        .get(&id)
        .ok_or(RenderError::MissingTranslation { id })?;
    let source = collect_all_text(node);
    validate_placeholders(id, &source, replacement)?;

    if !replace_first_text_descendant(node, &MarkdownOptions::default(), replacement, false) {
        return Err(RenderError::UnsupportedStructure { id, kind });
    }

    *next_id += 1;
    Ok(())
}

fn replace_frontmatter(
    raw: &str,
    translated: &HashMap<usize, String>,
    mut next_id: usize,
) -> Result<(String, usize), RenderError> {
    let mut consumed = 0usize;
    let mut updated_lines = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            updated_lines.push(line.to_string());
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            updated_lines.push(line.to_string());
            continue;
        };

        let value_trimmed = value.trim();
        if value_trimmed.is_empty() || is_probable_url_or_path(value_trimmed) {
            updated_lines.push(line.to_string());
            continue;
        }

        let id = next_id;
        let replacement = translated
            .get(&id)
            .ok_or(RenderError::MissingTranslation { id })?;
        validate_placeholders(id, value_trimmed, replacement)?;
        updated_lines.push(format!("{key}: {replacement}"));
        next_id += 1;
        consumed += 1;
    }

    Ok((updated_lines.join("\n"), consumed))
}

fn container_has_replaceable_text<'a>(node: &'a AstNode<'a>, options: &MarkdownOptions) -> bool {
    !collect_container_placeholder_text(node, options)
        .trim()
        .is_empty()
}

fn node_has_replaceable_text<'a>(node: &'a AstNode<'a>) -> bool {
    !collect_all_text(node).trim().is_empty()
}

fn replace_first_text_descendant<'a>(
    node: &'a AstNode<'a>,
    options: &MarkdownOptions,
    replacement: &str,
    skip_nested_special_nodes: bool,
) -> bool {
    let mut remaining = Some(replacement.to_string());
    let replaced =
        replace_text_descendants(node, options, &mut remaining, skip_nested_special_nodes);
    replaced && remaining.as_deref().unwrap_or_default().is_empty()
}

fn replace_text_descendants<'a>(
    node: &'a AstNode<'a>,
    options: &MarkdownOptions,
    remaining: &mut Option<String>,
    skip_nested_special_nodes: bool,
) -> bool {
    let value = node.data.borrow().value.clone();
    match value {
        NodeValue::Text(_) => {
            if let Some(text) = remaining.take() {
                node.data.borrow_mut().value = NodeValue::Text(text);
                *remaining = Some(String::new());
                return true;
            }
            return false;
        }
        NodeValue::Code(_)
        | NodeValue::CodeBlock(_)
        | NodeValue::HtmlBlock(_)
        | NodeValue::HtmlInline(_)
        | NodeValue::Math(_) => return false,
        NodeValue::Image(_) if skip_nested_special_nodes => return false,
        NodeValue::Link(_) if skip_nested_special_nodes && options.translate_link_text => {
            return false;
        }
        _ => {}
    }

    for child in node.children() {
        if replace_text_descendants(child, options, remaining, skip_nested_special_nodes) {
            clear_following_text_descendants(node, child, options, skip_nested_special_nodes);
            return true;
        }
    }

    false
}

fn clear_following_text_descendants<'a>(
    node: &'a AstNode<'a>,
    replaced_child: &'a AstNode<'a>,
    options: &MarkdownOptions,
    skip_nested_special_nodes: bool,
) {
    let mut seen_replaced_child = false;
    for child in node.children() {
        if std::ptr::eq(child, replaced_child) {
            seen_replaced_child = true;
            continue;
        }

        if seen_replaced_child {
            clear_text_descendants(child, options, skip_nested_special_nodes);
        }
    }
}

fn clear_text_descendants<'a>(
    node: &'a AstNode<'a>,
    options: &MarkdownOptions,
    skip_nested_special_nodes: bool,
) {
    let value = node.data.borrow().value.clone();
    match value {
        NodeValue::Text(_) => {
            node.data.borrow_mut().value = NodeValue::Text(String::new());
        }
        NodeValue::Code(_)
        | NodeValue::CodeBlock(_)
        | NodeValue::HtmlBlock(_)
        | NodeValue::HtmlInline(_)
        | NodeValue::Math(_) => {}
        NodeValue::Image(_) if skip_nested_special_nodes => {}
        NodeValue::Link(_) if skip_nested_special_nodes && options.translate_link_text => {}
        _ => {
            for child in node.children() {
                clear_text_descendants(child, options, skip_nested_special_nodes);
            }
        }
    }
}

fn collect_container_placeholder_text<'a>(
    node: &'a AstNode<'a>,
    options: &MarkdownOptions,
) -> String {
    let mut text = String::new();
    let mut html_inline_depth = 0usize;
    for child in node.children() {
        collect_container_child_text(child, options, &mut html_inline_depth, &mut text);
    }
    text.trim().to_string()
}

fn collect_container_child_text<'a>(
    node: &'a AstNode<'a>,
    options: &MarkdownOptions,
    html_inline_depth: &mut usize,
    out: &mut String,
) {
    match &node.data.borrow().value {
        NodeValue::Text(text) => {
            if *html_inline_depth == 0 && !is_probable_url_or_path(text.trim()) {
                out.push_str(text);
            }
        }
        NodeValue::Code(_) | NodeValue::CodeBlock(_) => {}
        NodeValue::HtmlBlock(_) => {}
        NodeValue::HtmlInline(raw) => update_html_inline_depth(raw, html_inline_depth),
        NodeValue::Math(_) => {}
        NodeValue::SoftBreak | NodeValue::LineBreak => {
            if *html_inline_depth == 0 {
                out.push(' ');
            }
        }
        NodeValue::FrontMatter(_) => {}
        NodeValue::Image(_) => {}
        NodeValue::Link(_) if options.translate_link_text => {}
        _ => {
            for child in node.children() {
                collect_container_child_text(child, options, html_inline_depth, out);
            }
        }
    }
}

fn collect_all_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut out = String::new();
    collect_all_text_inner(node, &mut out);
    out.trim().to_string()
}

fn collect_all_text_inner<'a>(node: &'a AstNode<'a>, out: &mut String) {
    match &node.data.borrow().value {
        NodeValue::Text(text) => out.push_str(text),
        NodeValue::SoftBreak | NodeValue::LineBreak => out.push(' '),
        NodeValue::Code(_)
        | NodeValue::CodeBlock(_)
        | NodeValue::HtmlBlock(_)
        | NodeValue::HtmlInline(_)
        | NodeValue::Math(_)
        | NodeValue::FrontMatter(_) => {}
        _ => {
            for child in node.children() {
                collect_all_text_inner(child, out);
            }
        }
    }
}

fn validate_placeholders(id: usize, source: &str, replacement: &str) -> Result<(), RenderError> {
    let source_tokens = collect_placeholder_tokens(source);
    let replacement_tokens = collect_placeholder_tokens(replacement);
    if source_tokens == replacement_tokens {
        Ok(())
    } else {
        Err(RenderError::PlaceholderMismatch { id })
    }
}

fn collect_placeholder_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut remainder = text;

    while let Some(start) = remainder.find("__MDT_") {
        let after_prefix = &remainder[start + 6..];
        let Some(end_offset) = after_prefix.find("__") else {
            break;
        };
        let token = format!("__MDT_{}__", &after_prefix[..end_offset]);
        tokens.push(token);
        remainder = &after_prefix[end_offset + 2..];
    }

    tokens
}

fn is_probable_url_or_path(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return true;
    }

    trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("mailto:")
        || trimmed.starts_with('/')
        || trimmed.starts_with("./")
        || trimmed.starts_with("../")
        || (!trimmed.contains(char::is_whitespace)
            && (trimmed.contains('/') || trimmed.contains('\\')))
}

fn update_html_inline_depth(raw: &str, html_inline_depth: &mut usize) {
    let trimmed = raw.trim();
    if trimmed.starts_with("</") {
        *html_inline_depth = html_inline_depth.saturating_sub(1);
    } else if trimmed.starts_with('<') && !trimmed.ends_with("/>") && !trimmed.starts_with("<!--") {
        *html_inline_depth += 1;
    }
}
