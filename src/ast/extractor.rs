use comrak::nodes::{AstNode, NodeValue};

use crate::types::MarkdownOptions;

use super::parser::MarkdownAst;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslatableNodeKind {
    Paragraph,
    Heading,
    ListItem,
    BlockQuote,
    LinkText,
    ImageAltText,
    FrontMatterValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslatableNode {
    pub id: usize,
    pub kind: TranslatableNodeKind,
    pub text: String,
}

pub fn extract_translatable_nodes(
    ast: &MarkdownAst,
    options: &MarkdownOptions,
) -> Vec<TranslatableNode> {
    ast.with_root(|root| extract_from_root(root, options))
}

fn extract_from_root<'a>(
    root: &'a AstNode<'a>,
    options: &MarkdownOptions,
) -> Vec<TranslatableNode> {
    let mut nodes = Vec::new();
    let mut next_id = 1;
    walk(root, options, &mut next_id, &mut nodes);
    nodes
}

fn walk<'a>(
    node: &'a AstNode<'a>,
    options: &MarkdownOptions,
    next_id: &mut usize,
    out: &mut Vec<TranslatableNode>,
) {
    match &node.data.borrow().value {
        NodeValue::Paragraph => {
            push_container(node, TranslatableNodeKind::Paragraph, options, next_id, out)
        }
        NodeValue::Heading(_) => {
            push_container(node, TranslatableNodeKind::Heading, options, next_id, out)
        }
        NodeValue::Item(_) => {
            push_container(node, TranslatableNodeKind::ListItem, options, next_id, out)
        }
        NodeValue::BlockQuote => push_container(
            node,
            TranslatableNodeKind::BlockQuote,
            options,
            next_id,
            out,
        ),
        NodeValue::Link(_) if options.translate_link_text => {
            push_leaf(node, TranslatableNodeKind::LinkText, options, next_id, out)
        }
        NodeValue::Image(_) => {
            if let Some(text) = collect_text(node, options) {
                push_node(TranslatableNodeKind::ImageAltText, text, next_id, out);
            }
        }
        NodeValue::FrontMatter(raw) if options.translate_frontmatter => {
            for value in extract_frontmatter_values(raw) {
                push_node(TranslatableNodeKind::FrontMatterValue, value, next_id, out);
            }
        }
        NodeValue::CodeBlock(_)
        | NodeValue::Code(_)
        | NodeValue::HtmlBlock(_)
        | NodeValue::HtmlInline(_)
        | NodeValue::Math(_) => {}
        _ => {
            for child in node.children() {
                walk(child, options, next_id, out);
            }
        }
    }
}

fn push_container<'a>(
    node: &'a AstNode<'a>,
    kind: TranslatableNodeKind,
    options: &MarkdownOptions,
    next_id: &mut usize,
    out: &mut Vec<TranslatableNode>,
) {
    if let Some(text) = collect_container_text(node, options) {
        push_node(kind, text, next_id, out);
    }

    for child in node.children() {
        walk(child, options, next_id, out);
    }
}

fn push_leaf<'a>(
    node: &'a AstNode<'a>,
    kind: TranslatableNodeKind,
    options: &MarkdownOptions,
    next_id: &mut usize,
    out: &mut Vec<TranslatableNode>,
) {
    if let Some(text) = collect_text(node, options) {
        push_node(kind, text, next_id, out);
    }

    for child in node.children() {
        walk(child, options, next_id, out);
    }
}

fn push_node(
    kind: TranslatableNodeKind,
    text: String,
    next_id: &mut usize,
    out: &mut Vec<TranslatableNode>,
) {
    out.push(TranslatableNode {
        id: *next_id,
        kind,
        text,
    });
    *next_id += 1;
}

fn collect_text<'a>(node: &'a AstNode<'a>, options: &MarkdownOptions) -> Option<String> {
    let mut text = String::new();
    collect_text_inner(node, options, false, &mut text);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn collect_container_text<'a>(node: &'a AstNode<'a>, options: &MarkdownOptions) -> Option<String> {
    let mut text = String::new();
    let mut html_inline_depth = 0usize;
    for child in node.children() {
        collect_container_child_text(child, options, &mut html_inline_depth, &mut text);
    }
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
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
        NodeValue::Image(_) => (),
        NodeValue::Link(_) if options.translate_link_text => (),
        _ => {
            for child in node.children() {
                collect_container_child_text(child, options, html_inline_depth, out);
            }
        }
    }
}

#[allow(clippy::only_used_in_recursion)]
fn collect_text_inner<'a>(
    node: &'a AstNode<'a>,
    options: &MarkdownOptions,
    skip_nested_special_nodes: bool,
    out: &mut String,
) {
    match &node.data.borrow().value {
        NodeValue::Text(text) => {
            if !is_probable_url_or_path(text.trim()) {
                out.push_str(text);
            }
        }
        NodeValue::Code(_) | NodeValue::CodeBlock(_) => {}
        NodeValue::HtmlInline(_) | NodeValue::HtmlBlock(_) => {}
        NodeValue::Math(_) => {}
        NodeValue::SoftBreak | NodeValue::LineBreak => out.push(' '),
        NodeValue::FrontMatter(_) => {}
        NodeValue::Link(_) if skip_nested_special_nodes => return,
        NodeValue::Image(_) if skip_nested_special_nodes => return,
        _ => {
            for child in node.children() {
                collect_text_inner(child, options, skip_nested_special_nodes, out);
            }
            return;
        }
    }

    for child in node.children() {
        collect_text_inner(child, options, skip_nested_special_nodes, out);
    }
}

fn extract_frontmatter_values(raw: &str) -> Vec<String> {
    raw.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }

            let (_, value) = trimmed.split_once(':')?;
            let value = value.trim();
            if value.is_empty() || is_probable_url_or_path(value) {
                None
            } else {
                Some(value.trim_matches('"').trim_matches('\'').to_string())
            }
        })
        .collect()
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
