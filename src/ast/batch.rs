use super::extractor::TranslatableNode;

#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub max_chars_per_batch: usize,
    pub max_items_per_batch: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_chars_per_batch: 5000,
            max_items_per_batch: 50,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlBatch {
    pub xml: String,
    pub ids: Vec<usize>,
}

/// Pack translatable nodes into XML batches.
///
/// Each node is emitted as `<seg id="N">escaped_text</seg>`.
/// Batches are split when adding another segment would exceed either
/// `max_items_per_batch` or `max_chars_per_batch`. A single segment
/// that is larger than `max_chars_per_batch` is placed in its own
/// batch rather than being truncated or split.
pub fn pack_nodes_into_xml_batches(
    nodes: &[TranslatableNode],
    config: &BatchConfig,
) -> Vec<XmlBatch> {
    let mut batches: Vec<XmlBatch> = Vec::new();
    let mut current = XmlBatch {
        xml: String::new(),
        ids: Vec::new(),
    };
    let mut current_chars: usize = 0;

    for node in nodes {
        let escaped = escape_xml(&node.text);
        let segment = format!(r#"<seg id="{}">{}</seg>"#, node.id, escaped);
        let seg_len = segment.len();

        let would_exceed_items =
            !current.ids.is_empty() && current.ids.len() >= config.max_items_per_batch;
        let would_exceed_chars =
            !current.ids.is_empty() && current_chars + seg_len > config.max_chars_per_batch;

        if would_exceed_items || would_exceed_chars {
            batches.push(current);
            current = XmlBatch {
                xml: String::new(),
                ids: Vec::new(),
            };
            current_chars = 0;
        }

        current.xml.push_str(&segment);
        current.ids.push(node.id);
        current_chars += seg_len;
    }

    if !current.ids.is_empty() {
        batches.push(current);
    }

    batches
}

fn escape_xml(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::extractor::TranslatableNodeKind;

    #[test]
    fn escape_xml_special_chars() {
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("a < b"), "a &lt; b");
        assert_eq!(escape_xml("a > b"), "a &gt; b");
        assert_eq!(escape_xml(r#""quote""#), "&quot;quote&quot;");
        assert_eq!(escape_xml("'apos'"), "&apos;apos&apos;");
    }

    #[test]
    fn batch_respects_item_limit() {
        let nodes = vec![
            TranslatableNode {
                id: 1,
                kind: TranslatableNodeKind::Paragraph,
                text: "a".into(),
            },
            TranslatableNode {
                id: 2,
                kind: TranslatableNodeKind::Paragraph,
                text: "b".into(),
            },
            TranslatableNode {
                id: 3,
                kind: TranslatableNodeKind::Paragraph,
                text: "c".into(),
            },
        ];
        let config = BatchConfig {
            max_chars_per_batch: 10_000,
            max_items_per_batch: 2,
        };
        let batches = pack_nodes_into_xml_batches(&nodes, &config);
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].ids, vec![1, 2]);
        assert_eq!(batches[1].ids, vec![3]);
    }

    #[test]
    fn batch_respects_char_limit() {
        let nodes = vec![
            TranslatableNode {
                id: 1,
                kind: TranslatableNodeKind::Paragraph,
                text: "hello".into(),
            },
            TranslatableNode {
                id: 2,
                kind: TranslatableNodeKind::Paragraph,
                text: "world".into(),
            },
        ];
        let config = BatchConfig {
            max_chars_per_batch: 30,
            max_items_per_batch: 10,
        };
        let batches = pack_nodes_into_xml_batches(&nodes, &config);
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].ids, vec![1]);
        assert_eq!(batches[1].ids, vec![2]);
    }

    #[test]
    fn empty_nodes_yields_no_batches() {
        let nodes: Vec<TranslatableNode> = vec![];
        let batches = pack_nodes_into_xml_batches(&nodes, &BatchConfig::default());
        assert!(batches.is_empty());
    }

    #[test]
    fn single_node_single_batch() {
        let nodes = vec![TranslatableNode {
            id: 1,
            kind: TranslatableNodeKind::Paragraph,
            text: "hello".into(),
        }];
        let batches = pack_nodes_into_xml_batches(&nodes, &BatchConfig::default());
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].xml, r#"<seg id="1">hello</seg>"#);
        assert_eq!(batches[0].ids, vec![1]);
    }

    #[test]
    fn oversized_segment_gets_own_batch() {
        let nodes = vec![TranslatableNode {
            id: 1,
            kind: TranslatableNodeKind::Paragraph,
            text: "x".repeat(100),
        }];
        let config = BatchConfig {
            max_chars_per_batch: 10,
            max_items_per_batch: 10,
        };
        let batches = pack_nodes_into_xml_batches(&nodes, &config);
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].ids, vec![1]);
        assert!(batches[0].xml.contains(&"x".repeat(100)));
    }
}
