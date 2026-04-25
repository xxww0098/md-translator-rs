pub mod batch;
pub mod extractor;
pub mod parse_response;
pub mod parser;
pub mod pipeline;
pub mod render;

pub use batch::{BatchConfig, XmlBatch, pack_nodes_into_xml_batches};
pub use extractor::{TranslatableNode, TranslatableNodeKind, extract_translatable_nodes};
pub use parse_response::{ParseResponseError, parse_xml_response};
pub use parser::{
    MarkdownAst, markdown_parse_options, parse_markdown, parse_markdown_with_options,
};
pub use pipeline::{AstTranslationPipeline, parse_extract_translate_replace_render};
pub use render::{RenderError, render_markdown, replace_translated_nodes_and_render};

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use comrak::nodes::NodeValue;

    use super::extractor::{TranslatableNode, TranslatableNodeKind, extract_translatable_nodes};
    use super::parser::{markdown_parse_options, parse_markdown, parse_markdown_with_options};
    use super::pipeline::parse_extract_translate_replace_render;
    use super::render::{RenderError, render_markdown};
    use crate::types::MarkdownOptions;

    #[test]
    fn parse_simple_markdown() {
        let ast = parse_markdown(
            "# Hello\n\n> Quote\n\n- item\n\n| a | b |\n| --- | --- |\n| c | d |\n\nA [link](https://example.com) and ![image](img.png)",
        );
        assert!(ast.contains_value(|v| matches!(v, NodeValue::Document)));
        assert!(ast.contains_value(|v| matches!(v, NodeValue::Heading(_))));
        assert!(ast.contains_value(|v| matches!(v, NodeValue::BlockQuote)));
        assert!(ast.contains_value(|v| matches!(v, NodeValue::List(_))));
        assert!(ast.contains_value(|v| matches!(v, NodeValue::Table(_))));
        assert!(ast.contains_value(|v| matches!(v, NodeValue::Link(_))));
        assert!(ast.contains_value(|v| matches!(v, NodeValue::Image(_))));
    }

    #[test]
    fn parse_with_code_blocks() {
        let ast = parse_markdown_with_options(
            "```rust\nlet x = 1;\n```\n\nInline `code` and $$x^2$$",
            markdown_parse_options(),
        );
        assert!(ast.contains_value(|v| matches!(v, NodeValue::CodeBlock(_))));
        assert!(ast.contains_value(|v| matches!(v, NodeValue::Code(_))));
        assert!(ast.contains_value(|v| matches!(v, NodeValue::Math(_))));
    }

    #[test]
    fn parse_with_frontmatter() {
        let ast = parse_markdown_with_options(
            "---\ntitle: Hello\ndescription: Example\n---\n\nBody",
            markdown_parse_options(),
        );
        assert!(ast.contains_value(|v| matches!(v, NodeValue::FrontMatter(_))));
        assert!(ast.contains_value(|v| matches!(v, NodeValue::Paragraph)));
    }

    #[test]
    fn extract_nodes() {
        let options = MarkdownOptions {
            translate_frontmatter: true,
            translate_multiline_code: false,
            translate_latex: false,
            translate_link_text: true,
        };

        let ast = parse_markdown_with_options(
            r#"---
title: Hello
description: Example
path: /docs/guide.md
---

# Heading

Paragraph with [link](https://example.com), ![alt](img.png), `code`, <span>html</span>, and $$math$$.

> Quote
>
> - item
"#,
            markdown_parse_options(),
        );

        let nodes = extract_translatable_nodes(&ast, &options);
        let ids: Vec<_> = nodes.iter().map(|node| node.id).collect();
        assert_eq!(ids, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let kinds: Vec<_> = nodes.iter().map(|node| &node.kind).collect();
        assert_eq!(
            kinds,
            vec![
                &TranslatableNodeKind::FrontMatterValue,
                &TranslatableNodeKind::FrontMatterValue,
                &TranslatableNodeKind::Heading,
                &TranslatableNodeKind::Paragraph,
                &TranslatableNodeKind::LinkText,
                &TranslatableNodeKind::ImageAltText,
                &TranslatableNodeKind::BlockQuote,
                &TranslatableNodeKind::Paragraph,
                &TranslatableNodeKind::ListItem,
                &TranslatableNodeKind::Paragraph,
            ]
        );

        assert!(nodes.iter().all(|node| !node.text.contains("code")));
        assert!(nodes.iter().all(|node| !node.text.contains("html")));
        assert!(
            nodes
                .iter()
                .all(|node| !node.text.contains("https://example.com"))
        );
        assert!(
            nodes
                .iter()
                .all(|node| !node.text.contains("/docs/guide.md"))
        );
    }

    #[test]
    fn extract_nodes_skips_link_text_when_disabled() {
        let ast = parse_markdown_with_options(
            "A [link](https://example.com) and ![alt](img.png)",
            markdown_parse_options(),
        );
        let options = MarkdownOptions {
            translate_frontmatter: false,
            translate_multiline_code: false,
            translate_latex: false,
            translate_link_text: false,
        };

        let nodes = extract_translatable_nodes(&ast, &options);
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].kind, TranslatableNodeKind::Paragraph);
        assert_eq!(nodes[0].text, "A link and");
        assert_eq!(nodes[1].kind, TranslatableNodeKind::ImageAltText);
        assert_eq!(nodes[1].text, "alt");
        assert!(
            nodes
                .iter()
                .all(|node| node.kind != TranslatableNodeKind::LinkText)
        );
    }

    #[test]
    fn extract_nodes_skips_url_only_paragraphs_and_frontmatter_paths() {
        let ast = parse_markdown_with_options(
            "---\npath: ./docs/file.md\nurl: https://example.com\ntitle: Keep me\n---\n\nhttps://example.com/path\n\n./docs/file.md\n\nKeep me",
            markdown_parse_options(),
        );
        let options = MarkdownOptions {
            translate_frontmatter: true,
            translate_multiline_code: false,
            translate_latex: false,
            translate_link_text: true,
        };

        let nodes = extract_translatable_nodes(&ast, &options);
        let texts: Vec<_> = nodes.iter().map(|node| node.text.as_str()).collect();

        assert_eq!(texts, vec!["Keep me", "Keep me"]);
        assert_eq!(nodes[0].kind, TranslatableNodeKind::FrontMatterValue);
        assert_eq!(nodes[1].kind, TranslatableNodeKind::Paragraph);
    }

    #[test]
    fn pack_xml_batch() {
        let nodes = vec![
            TranslatableNode {
                id: 1,
                kind: TranslatableNodeKind::Paragraph,
                text: "Hello & welcome".into(),
            },
            TranslatableNode {
                id: 2,
                kind: TranslatableNodeKind::Heading,
                text: "Section <1>".into(),
            },
            TranslatableNode {
                id: 3,
                kind: TranslatableNodeKind::Paragraph,
                text: "Third".into(),
            },
        ];
        let config = super::batch::BatchConfig {
            max_chars_per_batch: 1000,
            max_items_per_batch: 10,
        };
        let batches = super::batch::pack_nodes_into_xml_batches(&nodes, &config);
        assert_eq!(batches.len(), 1);
        assert_eq!(
            batches[0].xml,
            r#"<seg id="1">Hello &amp; welcome</seg><seg id="2">Section &lt;1&gt;</seg><seg id="3">Third</seg>"#
        );
        assert_eq!(batches[0].ids, vec![1, 2, 3]);
    }

    #[test]
    fn parse_xml_response() {
        let xml = r#"<seg id="1">Hello &amp; welcome</seg><seg id="2">Section &lt;1&gt;</seg><seg id="3">Third</seg>"#;
        let result = super::parse_response::parse_xml_response(xml, &[1, 2, 3]).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[&1], "Hello & welcome");
        assert_eq!(result[&2], "Section <1>");
        assert_eq!(result[&3], "Third");
    }

    #[test]
    fn parse_xml_response_missing_id_fails() {
        let xml = r#"<seg id="1">First</seg><seg id="3">Third</seg>"#;
        let result = super::parse_response::parse_xml_response(xml, &[1, 2, 3]);

        assert!(matches!(
            result,
            Err(super::parse_response::ParseResponseError::MissingSegment { expected_id: 2 })
        ));
    }

    #[test]
    fn parse_xml_response_roundtrip() {
        let nodes = vec![
            TranslatableNode {
                id: 5,
                kind: TranslatableNodeKind::Paragraph,
                text: "Hello & farewell".into(),
            },
            TranslatableNode {
                id: 10,
                kind: TranslatableNodeKind::Heading,
                text: "Test <tag> & \"quotes\"".into(),
            },
        ];
        let config = super::batch::BatchConfig {
            max_chars_per_batch: 1000,
            max_items_per_batch: 10,
        };
        let batches = super::batch::pack_nodes_into_xml_batches(&nodes, &config);
        assert_eq!(batches.len(), 1);

        let result =
            super::parse_response::parse_xml_response(&batches[0].xml, &batches[0].ids).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[&5], "Hello & farewell");
        assert_eq!(result[&10], "Test <tag> & \"quotes\"");
    }

    #[test]
    fn round_trip() {
        let input = r#"---
title: Hello
---

# Heading

Paragraph with [link](https://example.com) and ![alt](img.png).

```rust
let x = 1;
```
"#;

        let ast = parse_markdown_with_options(input, markdown_parse_options());
        let rendered = render_markdown(&ast).unwrap();

        assert!(rendered.contains("title: Hello"));
        assert!(rendered.contains("# Heading"));
        assert!(rendered.contains("[link](https://example.com)"));
        assert!(rendered.contains("![alt](img.png)"));
        assert!(rendered.contains("``` rust\nlet x = 1;\n```"));
    }

    #[test]
    fn translate_pipeline() {
        let input = r#"---
title: Hello
description: __MDT_KEEP_0__
---

# Hello

Simple paragraph for translation.

```rust
let x = 1;
```
"#;
        let options = MarkdownOptions {
            translate_frontmatter: true,
            translate_multiline_code: false,
            translate_latex: false,
            translate_link_text: true,
        };

        let mut translated = HashMap::new();
        translated.insert(1, "Bonjour".to_string());
        translated.insert(2, "Texte __MDT_KEEP_0__".to_string());
        translated.insert(3, "Salut".to_string());
        translated.insert(4, "Paragraphe simple pour la traduction.".to_string());

        let rendered = parse_extract_translate_replace_render(
            input,
            markdown_parse_options(),
            &options,
            &translated,
        )
        .unwrap();

        assert!(rendered.contains("title: Bonjour"));
        assert!(rendered.contains("description: Texte __MDT_KEEP_0__"));
        assert!(rendered.contains("# Salut"));
        assert!(rendered.contains("Paragraphe simple pour la traduction."));
        assert!(rendered.contains("``` rust\nlet x = 1;\n```"));
    }

    #[test]
    fn translate_pipeline_rejects_placeholder_mutation() {
        let input = "Paragraph __MDT_KEEP_1__";
        let options = MarkdownOptions::default();
        let mut translated = HashMap::new();
        translated.insert(1, "Paragraphe __MDT_BROKEN_1__".to_string());

        let result = parse_extract_translate_replace_render(
            input,
            markdown_parse_options(),
            &options,
            &translated,
        );

        assert!(matches!(
            result,
            Err(RenderError::PlaceholderMismatch { id: 1 })
        ));
    }
}
