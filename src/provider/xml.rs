use std::collections::HashMap;
use std::fmt;
use std::io::Cursor;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::types::BatchItem;

#[derive(Debug, Clone)]
pub struct ProviderXmlBatchConfig {
    pub max_chars_per_batch: usize,
    pub max_items_per_batch: usize,
}

impl Default for ProviderXmlBatchConfig {
    fn default() -> Self {
        Self {
            max_chars_per_batch: 5000,
            max_items_per_batch: 50,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderXmlBatch {
    pub xml: String,
    pub ids: Vec<usize>,
}

/// Error type for provider XML response parsing failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderXmlParseError {
    MissingSegment { expected_id: usize },
    UnexpectedEvent { message: String },
    InvalidId { message: String },
}

impl fmt::Display for ProviderXmlParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderXmlParseError::MissingSegment { expected_id } => {
                write!(f, "missing segment for expected id={}", expected_id)
            }
            ProviderXmlParseError::UnexpectedEvent { message } => {
                write!(f, "unexpected XML event: {}", message)
            }
            ProviderXmlParseError::InvalidId { message } => {
                write!(f, "invalid id attribute: {}", message)
            }
        }
    }
}

impl std::error::Error for ProviderXmlParseError {}

pub fn pack_batch_items_into_xml_batches(
    items: &[BatchItem],
    config: &ProviderXmlBatchConfig,
) -> Vec<ProviderXmlBatch> {
    let mut batches: Vec<ProviderXmlBatch> = Vec::new();
    let mut current = ProviderXmlBatch {
        xml: String::new(),
        ids: Vec::new(),
    };
    let mut current_chars: usize = 0;

    for item in items {
        let escaped = escape_xml(&item.text);
        let segment = format!(r#"<seg id="{}">{}</seg>"#, item.id, escaped);
        let seg_len = segment.len();

        let would_exceed_items =
            !current.ids.is_empty() && current.ids.len() >= config.max_items_per_batch;
        let would_exceed_chars =
            !current.ids.is_empty() && current_chars + seg_len > config.max_chars_per_batch;

        if would_exceed_items || would_exceed_chars {
            batches.push(current);
            current = ProviderXmlBatch {
                xml: String::new(),
                ids: Vec::new(),
            };
            current_chars = 0;
        }

        current.xml.push_str(&segment);
        current.ids.push(item.id);
        current_chars += seg_len;
    }

    if !current.ids.is_empty() {
        batches.push(current);
    }

    batches
}

#[allow(clippy::result_large_err)]
pub fn parse_provider_xml_response(
    xml: &str,
    expected_ids: &[usize],
) -> Result<HashMap<usize, String>, ProviderXmlParseError> {
    let mut reader = Reader::from_reader(Cursor::new(xml.as_bytes()));
    reader.config_mut().check_end_names = true;
    reader.config_mut().check_comments = true;

    let mut result_map: HashMap<usize, String> = HashMap::new();
    let mut buf = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"seg" => {
                let id = extract_id_attribute(&e)?;
                let text = read_segment_text(&mut reader, &mut buf)?;
                result_map.insert(id, text);
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(ProviderXmlParseError::UnexpectedEvent {
                    message: format!(
                        "XML parsing error at position {}: {}",
                        reader.error_position(),
                        e
                    ),
                });
            }
            _ => {}
        }
    }

    for &expected_id in expected_ids {
        if !result_map.contains_key(&expected_id) {
            return Err(ProviderXmlParseError::MissingSegment { expected_id });
        }
    }

    Ok(result_map)
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

fn extract_id_attribute(event: &BytesStart) -> Result<usize, ProviderXmlParseError> {
    for attr in event.attributes().flatten() {
        if attr.key.as_ref() == b"id" {
            let value = attr.value.as_ref();
            return std::str::from_utf8(value)
                .map_err(|_| ProviderXmlParseError::InvalidId {
                    message: format!("non-UTF8 id value: {:?}", value),
                })?
                .parse::<usize>()
                .map_err(|_| ProviderXmlParseError::InvalidId {
                    message: format!("non-numeric id value: {:?}", value),
                });
        }
    }

    Err(ProviderXmlParseError::InvalidId {
        message: "missing id attribute on <seg> element".to_string(),
    })
}

fn read_segment_text(
    reader: &mut Reader<Cursor<&[u8]>>,
    buf: &mut Vec<u8>,
) -> Result<String, ProviderXmlParseError> {
    let mut text_content = String::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf) {
            Ok(Event::Text(e)) => {
                text_content.push_str(&e.unescape().map_err(|e| {
                    ProviderXmlParseError::UnexpectedEvent {
                        message: format!("failed to unescape text: {}", e),
                    }
                })?);
            }
            Ok(Event::CData(e)) => {
                text_content.push_str(std::str::from_utf8(e.as_ref()).map_err(|_| {
                    ProviderXmlParseError::UnexpectedEvent {
                        message: "CData contains non-UTF8".to_string(),
                    }
                })?);
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"seg" => break,
            Ok(Event::Eof) => {
                return Err(ProviderXmlParseError::UnexpectedEvent {
                    message: "unexpected EOF while reading segment text".to_string(),
                });
            }
            _ => {}
        }
    }

    Ok(text_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_xml_batches_follow_ast_batching_rules() {
        let items = vec![
            BatchItem {
                id: 1,
                text: "Hello & welcome".into(),
                context_before: vec![],
                context_after: vec![],
            },
            BatchItem {
                id: 2,
                text: "Section <1>".into(),
                context_before: vec![],
                context_after: vec![],
            },
        ];

        let batches = pack_batch_items_into_xml_batches(
            &items,
            &ProviderXmlBatchConfig {
                max_chars_per_batch: 1000,
                max_items_per_batch: 10,
            },
        );

        assert_eq!(batches.len(), 1);
        assert_eq!(
            batches[0].xml,
            r#"<seg id="1">Hello &amp; welcome</seg><seg id="2">Section &lt;1&gt;</seg>"#
        );
    }

    #[test]
    fn provider_xml_response_parses_segments() {
        let parsed = parse_provider_xml_response(
            r#"<seg id="1">First</seg><seg id="2">Second</seg>"#,
            &[1, 2],
        )
        .unwrap();

        assert_eq!(parsed[&1], "First");
        assert_eq!(parsed[&2], "Second");
    }

    #[test]
    fn provider_xml_response_requires_all_expected_ids() {
        let err = parse_provider_xml_response(r#"<seg id="1">First</seg>"#, &[1, 2]).unwrap_err();

        assert_eq!(
            err,
            ProviderXmlParseError::MissingSegment { expected_id: 2 }
        );
    }
}
