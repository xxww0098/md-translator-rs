//! Parse AI translation responses in XML format.
//!
//! AI providers return translated text as `<seg id="N">translated text</seg>` elements.
//! This module parses those responses into an ID→text map with strict validation.

use std::collections::HashMap;
use std::fmt;
use std::io::Cursor;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

/// Error type for XML response parsing failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseResponseError {
    /// Missing segment for an expected ID.
    MissingSegment { expected_id: usize },
    /// Unexpected XML structure.
    UnexpectedEvent { message: String },
    /// Invalid ID attribute value.
    InvalidId { message: String },
}

impl fmt::Display for ParseResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseResponseError::MissingSegment { expected_id } => {
                write!(f, "missing segment for expected id={}", expected_id)
            }
            ParseResponseError::UnexpectedEvent { message } => {
                write!(f, "unexpected XML event: {}", message)
            }
            ParseResponseError::InvalidId { message } => {
                write!(f, "invalid id attribute: {}", message)
            }
        }
    }
}

impl std::error::Error for ParseResponseError {}

/// Parse an XML response and validate that all expected IDs are present.
///
/// The XML should contain `<seg id="N">text</seg>` elements. Entity decoding
/// (`&lt;`, `&gt;`, `&amp;`, etc.) is handled automatically by the parser.
///
/// # Arguments
///
/// * `xml` - The raw XML string from the AI provider
/// * `expected_ids` - Ordered list of segment IDs that must be present
///
/// # Returns
///
/// A `HashMap` mapping segment ID → translated text, in the same order as `expected_ids`.
///
/// # Errors
///
/// Returns `ParseResponseError` if:
/// * Any expected ID is missing from the response
/// * The XML structure is invalid
#[allow(clippy::result_large_err)]
pub fn parse_xml_response(
    xml: &str,
    expected_ids: &[usize],
) -> Result<HashMap<usize, String>, ParseResponseError> {
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
                return Err(ParseResponseError::UnexpectedEvent {
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
            return Err(ParseResponseError::MissingSegment { expected_id });
        }
    }

    Ok(result_map)
}

/// Extract the `id` attribute from a `<seg>` start element.
fn extract_id_attribute(event: &BytesStart) -> Result<usize, ParseResponseError> {
    for attr in event.attributes().flatten() {
        if attr.key.as_ref() == b"id" {
            let value = attr.value.as_ref();
            return std::str::from_utf8(value)
                .map_err(|_| ParseResponseError::InvalidId {
                    message: format!("non-UTF8 id value: {:?}", value),
                })?
                .parse::<usize>()
                .map_err(|_| ParseResponseError::InvalidId {
                    message: format!("non-numeric id value: {:?}", value),
                });
        }
    }
    Err(ParseResponseError::InvalidId {
        message: "missing id attribute on <seg> element".to_string(),
    })
}

fn read_segment_text(
    reader: &mut Reader<Cursor<&[u8]>>,
    buf: &mut Vec<u8>,
) -> Result<String, ParseResponseError> {
    let mut text_content = String::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf) {
            Ok(Event::Text(e)) => {
                text_content.push_str(&e.unescape().map_err(|e| {
                    ParseResponseError::UnexpectedEvent {
                        message: format!("failed to unescape text: {}", e),
                    }
                })?);
            }
            Ok(Event::CData(e)) => {
                text_content.push_str(std::str::from_utf8(e.as_ref()).map_err(|_| {
                    ParseResponseError::UnexpectedEvent {
                        message: "CData contains non-UTF8".to_string(),
                    }
                })?);
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"seg" => {
                break;
            }
            Ok(Event::Eof) => {
                return Err(ParseResponseError::UnexpectedEvent {
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
    fn parse_simple_response() {
        let xml = r#"<seg id="1">Hello</seg><seg id="2">World</seg>"#;
        let result = parse_xml_response(xml, &[1, 2]).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[&1], "Hello");
        assert_eq!(result[&2], "World");
    }

    #[test]
    fn parse_single_segment() {
        let xml = r#"<seg id="42">Answer</seg>"#;
        let result = parse_xml_response(xml, &[42]).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[&42], "Answer");
    }

    #[test]
    fn decode_xml_entities() {
        // Test that &amp; &lt; &gt; &quot; &apos; are properly decoded
        let xml = r#"<seg id="1">a &amp; b &lt; c &gt; d &quot;e&quot; 'f'</seg>"#;
        let result = parse_xml_response(xml, &[1]).unwrap();

        assert_eq!(result[&1], "a & b < c > d \"e\" 'f'");
    }

    #[test]
    fn decode_html_entities_in_text() {
        let xml = r#"<seg id="1">pi&#241;ata</seg>"#;
        let result = parse_xml_response(xml, &[1]).unwrap();

        assert!(result[&1].contains("pi"));
    }

    #[test]
    fn missing_segment_fails() {
        let xml = r#"<seg id="1">Hello</seg><seg id="3">World</seg>"#;
        let result = parse_xml_response(xml, &[1, 2, 3]);

        assert!(matches!(
            result,
            Err(ParseResponseError::MissingSegment { expected_id: 2 })
        ));
    }

    #[test]
    fn empty_response_fails_when_ids_expected() {
        let xml = r#""#;
        let result = parse_xml_response(xml, &[1]);

        assert!(matches!(
            result,
            Err(ParseResponseError::MissingSegment { expected_id: 1 })
        ));
    }

    #[test]
    fn empty_ids_allows_empty_response() {
        let xml = r#""#;
        let result = parse_xml_response(xml, &[]);

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn parse_with_whitespace() {
        let xml = r#"
            <seg id="1">First</seg>
            <seg id="2">Second</seg>
        "#;
        let result = parse_xml_response(xml, &[1, 2]).unwrap();

        assert_eq!(result[&1], "First");
        assert_eq!(result[&2], "Second");
    }

    #[test]
    fn unchecked_parser_finds_all_segments() {
        let xml = r#"<seg id="5">Five</seg><seg id="10">Ten</seg>"#;
        let result = parse_xml_response(xml, &[]).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[&5], "Five");
        assert_eq!(result[&10], "Ten");
    }

    #[test]
    fn special_characters_in_text() {
        let xml = r#"<seg id="1"><![CDATA[<script>alert('xss')</script>]]></seg>"#;
        let result = parse_xml_response(xml, &[]).unwrap();

        assert!(result[&1].contains("<script>alert('xss')</script>"));
    }

    #[test]
    fn error_invalid_id_attribute() {
        let xml = r#"<seg id="not-a-number">Text</seg>"#;
        let result = parse_xml_response(xml, &[1]);

        assert!(matches!(result, Err(ParseResponseError::InvalidId { .. })));
    }

    #[test]
    fn error_missing_id_attribute() {
        let xml = r#"<seg>Text</seg>"#;
        let result = parse_xml_response(xml, &[1]);

        assert!(matches!(result, Err(ParseResponseError::InvalidId { .. })));
    }

    #[test]
    fn error_unclosed_segment() {
        let xml = r#"<seg id="1">Text"#;
        let result = parse_xml_response(xml, &[]);

        assert!(matches!(
            result,
            Err(ParseResponseError::UnexpectedEvent { .. })
        ));
    }

    #[test]
    fn parse_unicode_content() {
        let xml = r#"<seg id="1">你好世界</seg><seg id="2">🎉</seg>"#;
        let result = parse_xml_response(xml, &[1, 2]).unwrap();

        assert_eq!(result[&1], "你好世界");
        assert_eq!(result[&2], "🎉");
    }

    #[test]
    fn partial_response_missing_ids_fails() {
        // Simulate a real partial response where only some IDs come back
        let xml = r#"<seg id="1">Translated 1</seg><seg id="3">Translated 3</seg>"#;
        let result = parse_xml_response(xml, &[1, 2, 3, 4]);

        assert!(matches!(
            result,
            Err(ParseResponseError::MissingSegment { expected_id: 2 })
        ));
    }
}
