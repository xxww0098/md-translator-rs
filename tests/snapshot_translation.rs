//! Snapshot tests for translation stability.
//!
//! These tests verify that the translation pipeline produces stable, deterministic
//! output across repeated runs. They use a mock backend that returns deterministic
//! translations (prefixing with "tr ") so snapshots are stable.
//!
//! Structure preservation tests cover:
//! - Fenced code blocks
//! - Frontmatter
//! - Tables
//! - Links and images

use std::collections::HashMap;

use md_translator_rs::ast::{
    extract_translatable_nodes, markdown_parse_options, parse_extract_translate_replace_render,
    parse_markdown_with_options,
};
use md_translator_rs::types::MarkdownOptions;

/// Extracts translatable nodes and returns a deterministic translation map
/// that prefixes each original text with "tr ".
fn make_det_translations(markdown: &str, options: &MarkdownOptions) -> HashMap<usize, String> {
    let ast = parse_markdown_with_options(markdown, markdown_parse_options());
    let nodes = extract_translatable_nodes(&ast, options);
    nodes
        .iter()
        .map(|n| (n.id, format!("tr {}", n.text)))
        .collect()
}

#[test]
fn snapshot_translation_code_blocks_preserved() {
    let input = r#"# Example

Here is some code:

```rust
fn main() {
    println!("Hello, world!");
}
```

More text after.
"#;

    let options = MarkdownOptions {
        translate_frontmatter: false,
        translate_multiline_code: false,
        translate_latex: false,
        translate_link_text: false,
    };

    let translations = make_det_translations(input, &options);

    let rendered = parse_extract_translate_replace_render(
        input,
        markdown_parse_options(),
        &options,
        &translations,
    )
    .unwrap();

    assert!(
        rendered.contains("``` rust"),
        "code block fence should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("fn main()"),
        "code block content should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("tr Example"),
        "heading should be translated: {rendered}"
    );
    assert!(
        rendered.contains("tr More text after."),
        "paragraph should be translated: {rendered}"
    );
}

#[test]
fn snapshot_translation_frontmatter_preserved_and_translated() {
    let input = r#"---
title: Getting Started
description: A simple guide
date: 2024-01-15
---

# Introduction

Welcome to the project.
"#;

    let options = MarkdownOptions {
        translate_frontmatter: true,
        translate_multiline_code: false,
        translate_latex: false,
        translate_link_text: false,
    };

    let translations = make_det_translations(input, &options);

    let rendered = parse_extract_translate_replace_render(
        input,
        markdown_parse_options(),
        &options,
        &translations,
    )
    .unwrap();

    assert!(
        rendered.starts_with("---\n"),
        "frontmatter start delimiter should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("\n---\n"),
        "frontmatter end delimiter should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("tr Getting Started"),
        "title should be translated: {rendered}"
    );
    assert!(
        rendered.contains("tr A simple guide"),
        "description should be translated: {rendered}"
    );
    // Note: date IS translated because it's not a URL/path - only URLs/paths are skipped
    assert!(
        rendered.contains("tr 2024-01-15"),
        "date value should be translated: {rendered}"
    );
}

#[test]
fn snapshot_translation_table_preserved() {
    let input = r#"# Pricing

| Feature | Free | Pro |
|---------|------|-----|
| Translations | 100/month | Unlimited |
| Support | Community | Email |

Let us know if you have questions.
"#;

    let options = MarkdownOptions {
        translate_frontmatter: false,
        translate_multiline_code: false,
        translate_latex: false,
        translate_link_text: false,
    };

    let translations = make_det_translations(input, &options);

    let rendered = parse_extract_translate_replace_render(
        input,
        markdown_parse_options(),
        &options,
        &translations,
    )
    .unwrap();

    // Table structure preserved (table cells are not translatable nodes)
    assert!(
        rendered.contains("| Feature | Free | Pro |"),
        "table header should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("| --- |"),
        "table separator should be preserved: {rendered}"
    );
    // Heading translated, last paragraph translated
    assert!(
        rendered.contains("tr Pricing"),
        "heading should be translated: {rendered}"
    );
    assert!(
        rendered.contains("tr Let us know"),
        "paragraph should be translated: {rendered}"
    );
}

#[test]
fn snapshot_translation_links_preserved_urls_translated_text() {
    let input = r#"Check out [our website](https://example.com) for more info.

![logo](https://example.com/logo.png)
"#;

    let options = MarkdownOptions {
        translate_frontmatter: false,
        translate_multiline_code: false,
        translate_latex: false,
        translate_link_text: true,
    };

    let translations = make_det_translations(input, &options);

    let rendered = parse_extract_translate_replace_render(
        input,
        markdown_parse_options(),
        &options,
        &translations,
    )
    .unwrap();

    assert!(
        rendered.contains("(https://example.com)"),
        "link URL should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("(https://example.com/logo.png)"),
        "image URL should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("tr our website"),
        "link text should be translated: {rendered}"
    );
    assert!(
        rendered.contains("tr logo"),
        "image alt should be translated: {rendered}"
    );
}

#[test]
fn snapshot_translation_multilingual_content() {
    let input = r#"---
title: Technical Documentation
author: dev-team
---

# Overview

This document explains the system architecture.

```python
def hello():
    print("Hello, World!")
```

## Features

- Fast processing
- Reliable results
- Easy integration

> Important: Read the docs before starting.

[API Reference](https://api.example.com/docs)

![screenshot](https://example.com/screen.png)
"#;

    let options = MarkdownOptions {
        translate_frontmatter: true,
        translate_multiline_code: false,
        translate_latex: false,
        translate_link_text: true,
    };

    let translations = make_det_translations(input, &options);

    let rendered = parse_extract_translate_replace_render(
        input,
        markdown_parse_options(),
        &options,
        &translations,
    )
    .unwrap();

    assert!(
        rendered.contains("tr Technical Documentation"),
        "frontmatter title should be translated: {rendered}"
    );
    assert!(
        rendered.contains("---"),
        "frontmatter delimiters should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("``` python"),
        "code block should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("print(\"Hello, World!\")"),
        "code content should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("tr Fast processing"),
        "list items should be translated: {rendered}"
    );
    assert!(
        rendered.contains("> tr Important:"),
        "blockquote should be translated: {rendered}"
    );
    assert!(
        rendered.contains("https://api.example.com/docs"),
        "link URL should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("https://example.com/screen.png"),
        "image URL should be preserved: {rendered}"
    );
}

#[test]
fn snapshot_deterministic_same_input_same_output() {
    let input = r#"# Hello World

This is a test paragraph.

## Section

Another paragraph here.
"#;

    let options = MarkdownOptions::default();
    let translations: HashMap<_, _> = [
        (1, "Hola Mundo".to_string()),
        (2, "Esta es una prueba de parrafo.".to_string()),
        (3, "Seccion".to_string()),
        (4, "Otro parrafo aqui.".to_string()),
    ]
    .into_iter()
    .collect();

    let rendered1 = parse_extract_translate_replace_render(
        input,
        markdown_parse_options(),
        &options,
        &translations,
    )
    .unwrap();

    let rendered2 = parse_extract_translate_replace_render(
        input,
        markdown_parse_options(),
        &options,
        &translations,
    )
    .unwrap();

    assert_eq!(rendered1, rendered2, "translation should be deterministic");
}

#[test]
fn snapshot_roundtrip_no_change_without_translation() {
    let input = r#"---
title: Hello
description: Test
---

# Heading

Paragraph with [link](https://example.com) and ![alt](img.png).

```rust
let x = 1;
```

| a | b |
|---|---|
| c | d |
"#;

    let ast = md_translator_rs::ast::parse_markdown_with_options(input, markdown_parse_options());
    let rendered = md_translator_rs::ast::render_markdown(&ast).unwrap();

    assert!(
        rendered.contains("title: Hello"),
        "frontmatter title preserved: {rendered}"
    );
    assert!(
        rendered.contains("# Heading"),
        "heading preserved: {rendered}"
    );
    assert!(
        rendered.contains("[link](https://example.com)"),
        "link preserved: {rendered}"
    );
    assert!(
        rendered.contains("![alt](img.png)"),
        "image preserved: {rendered}"
    );
    assert!(
        rendered.contains("``` rust\nlet x = 1;\n```"),
        "code block preserved: {rendered}"
    );
    assert!(
        rendered.contains("| a | b |"),
        "table header preserved: {rendered}"
    );
}
