use std::collections::HashMap;

use md_translator_rs::ast::{
    TranslatableNodeKind, extract_translatable_nodes, markdown_parse_options,
    pack_nodes_into_xml_batches, parse_extract_translate_replace_render,
    parse_markdown_with_options, parse_xml_response, render_markdown,
};
use md_translator_rs::types::MarkdownOptions;

#[test]
fn integration_ast_full_pipeline_with_frontmatter_and_code_blocks() {
    let input = r#"---
title: Getting Started
description: A simple guide
---

# Introduction

Welcome to the project. This is a paragraph with [a link](https://example.com).

```rust
let x = 42;
```

> A blockquote with important info.

- First item
- Second item

![alt text](image.png)
"#;

    let parse_options = markdown_parse_options();
    let markdown_options = MarkdownOptions {
        translate_frontmatter: true,
        translate_multiline_code: false,
        translate_latex: false,
        translate_link_text: true,
    };

    let ast = parse_markdown_with_options(input, parse_options.clone());
    let nodes = extract_translatable_nodes(&ast, &markdown_options);
    assert!(!nodes.is_empty());
    assert!(
        nodes
            .iter()
            .any(|n| n.kind == TranslatableNodeKind::Heading)
    );
    assert!(
        nodes
            .iter()
            .any(|n| n.kind == TranslatableNodeKind::Paragraph)
    );
    assert!(
        nodes
            .iter()
            .any(|n| n.kind == TranslatableNodeKind::BlockQuote)
    );
    assert!(
        nodes
            .iter()
            .any(|n| n.kind == TranslatableNodeKind::ListItem)
    );
    assert!(
        nodes
            .iter()
            .any(|n| n.kind == TranslatableNodeKind::LinkText)
    );
    assert!(
        nodes
            .iter()
            .any(|n| n.kind == TranslatableNodeKind::ImageAltText)
    );
    assert!(
        nodes
            .iter()
            .any(|n| n.kind == TranslatableNodeKind::FrontMatterValue)
    );

    let config = md_translator_rs::ast::BatchConfig {
        max_chars_per_batch: 5000,
        max_items_per_batch: 20,
    };
    let batches = pack_nodes_into_xml_batches(&nodes, &config);
    assert!(!batches.is_empty());

    let mut translations = HashMap::new();
    for batch in &batches {
        let parsed = parse_xml_response(&batch.xml, &batch.ids).unwrap();
        for (id, text) in parsed {
            translations.insert(id, format!("tr {}", text));
        }
    }

    let rendered = parse_extract_translate_replace_render(
        input,
        parse_options,
        &markdown_options,
        &translations,
    )
    .unwrap();

    assert!(rendered.contains("title: tr Getting Started"));
    assert!(rendered.contains("description: tr A simple guide"));
    assert!(rendered.contains("# tr Introduction"));
    assert!(
        rendered.contains("tr Welcome to the project. This is a paragraph with"),
        "unexpected rendered output: {rendered}"
    );
    assert!(
        rendered.contains("[tr a link](https://example.com)"),
        "link URL should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("> tr A blockquote with important info."),
        "blockquote should be translated: {rendered}"
    );
    assert!(
        rendered.contains("- tr First item"),
        "list item should be translated: {rendered}"
    );
    assert!(
        rendered.contains("- tr Second item"),
        "list item should be translated: {rendered}"
    );
    assert!(
        rendered.contains("![tr alt text](image.png)"),
        "image alt should be translated: {rendered}"
    );
}

#[test]
fn integration_ast_round_trip_without_translation() {
    let input = r#"---
title: Hello
---

# Heading

Paragraph with [link](https://example.com) and ![alt](img.png).

```rust
let x = 1;
```
"#;

    let parse_options = markdown_parse_options();
    let ast = parse_markdown_with_options(input, parse_options);
    let rendered = render_markdown(&ast).unwrap();

    assert!(
        rendered.contains("title: Hello"),
        "frontmatter should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("# Heading"),
        "heading should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("[link](https://example.com)"),
        "link should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("![alt](img.png)"),
        "image should be preserved: {rendered}"
    );
    assert!(
        rendered.contains("``` rust\nlet x = 1;\n```"),
        "code block should be preserved: {rendered}"
    );
}

#[test]
fn integration_ast_pipeline_rejects_placeholder_mutation() {
    let input = "Paragraph __MDT_KEEP_1__ text";
    let options = MarkdownOptions::default();
    let mut translations = HashMap::new();
    translations.insert(1, "Paragraphe __MDT_BROKEN_1__ texte".to_string());

    let result = parse_extract_translate_replace_render(
        input,
        markdown_parse_options(),
        &options,
        &translations,
    );

    assert!(
        result.is_err(),
        "expected placeholder mismatch error, got: {result:?}"
    );
}

#[test]
fn integration_ast_batch_pack_and_parse_roundtrip() {
    let input = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
    let parse_options = markdown_parse_options();
    let markdown_options = MarkdownOptions::default();

    let ast = parse_markdown_with_options(input, parse_options);
    let nodes = extract_translatable_nodes(&ast, &markdown_options);
    assert_eq!(nodes.len(), 3, "expected 3 paragraph nodes");

    let config = md_translator_rs::ast::BatchConfig {
        max_chars_per_batch: 5000,
        max_items_per_batch: 10,
    };
    let batches = pack_nodes_into_xml_batches(&nodes, &config);
    assert_eq!(batches.len(), 1);

    let xml = batches[0].xml.clone();
    let translated_xml = xml
        .replace("First paragraph.", "FIRST PARAGRAPH.")
        .replace("Second paragraph.", "SECOND PARAGRAPH.")
        .replace("Third paragraph.", "THIRD PARAGRAPH.");

    let parsed = parse_xml_response(&translated_xml, &batches[0].ids).unwrap();
    assert_eq!(parsed.len(), 3);

    let mut translations = HashMap::new();
    for (id, text) in parsed {
        translations.insert(id, text);
    }

    let rendered = parse_extract_translate_replace_render(
        input,
        markdown_parse_options(),
        &markdown_options,
        &translations,
    )
    .unwrap();

    assert!(rendered.contains("FIRST PARAGRAPH."));
    assert!(rendered.contains("SECOND PARAGRAPH."));
    assert!(rendered.contains("THIRD PARAGRAPH."));
}
