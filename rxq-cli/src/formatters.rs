//! Formatters for different document types

use rxq_core::{Document, DocumentType, FormatError, FormatOptions, Formatter};
use std::io::Write;

/// Enum wrapper for formatters to avoid object safety issues
pub enum DocFormatter {
    Xml(XmlHtmlFormatter),
    Json(JsonFormatter),
}

impl Formatter for DocFormatter {
    fn format<'input, W: Write>(
        &self,
        doc: &Document<'input>,
        writer: &mut W,
        options: &FormatOptions,
    ) -> Result<(), FormatError> {
        match self {
            DocFormatter::Xml(f) => f.format(doc, writer, options),
            DocFormatter::Json(f) => f.format(doc, writer, options),
        }
    }
}

/// Get the appropriate formatter for a document type
pub fn get_formatter(doc_type: DocumentType) -> DocFormatter {
    match doc_type {
        DocumentType::Xml | DocumentType::Html => DocFormatter::Xml(XmlHtmlFormatter),
        DocumentType::Json => DocFormatter::Json(JsonFormatter),
    }
}

/// XML/HTML formatter implementation
pub struct XmlHtmlFormatter;

impl Formatter for XmlHtmlFormatter {
    fn format<'input, W: Write>(
        &self,
        doc: &Document<'input>,
        writer: &mut W,
        options: &FormatOptions,
    ) -> Result<(), FormatError> {
        // Use the core formatter
        let formatter = rxq_core::format::XmlFormatter;
        formatter.format(doc, writer, options)
    }
}

/// JSON formatter
pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn format<'input, W: Write>(
        &self,
        doc: &Document<'input>,
        writer: &mut W,
        options: &FormatOptions,
    ) -> Result<(), FormatError> {
        // Prepare JSON value
        let value: serde_json::Value = match doc.doc_type() {
            DocumentType::Xml | DocumentType::Html => {
                // Convert XML/HTML to JSON
                let root = doc.root();
                let json = rxq_core::to_json(root);

                if let Some(tag) = root.tag_name() {
                    let mut map = serde_json::Map::new();
                    map.insert(tag.into_owned(), json);
                    serde_json::Value::Object(map)
                } else {
                    json
                }
            }
            DocumentType::Json => {
                let source = doc.source();
                serde_json::from_str(source).map_err(|e| {
                    FormatError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                })?
            }
        };

        if options.compact {
            serde_json::to_writer(&mut *writer, &value)
                .map_err(|e| FormatError::IoError(e.into()))?;
        } else {
            // Configure indentation
            let indent_str = if let rxq_core::format::Indent::Spaces(n) = options.indent {
                " ".repeat(n as usize)
            } else {
                "\t".to_string()
            };

            let formatter = serde_json::ser::PrettyFormatter::with_indent(indent_str.as_bytes());
            let mut serializer = serde_json::Serializer::with_formatter(&mut *writer, formatter);

            use serde::Serialize;
            value
                .serialize(&mut serializer)
                .map_err(|e| FormatError::IoError(e.into()))?;
        }

        // Add trailing newline if pretty printing
        if !options.compact {
            writeln!(writer)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_formatter_pretty() {
        let json = r#"{"key":"value"}"#;
        let doc = Document::parse(json, DocumentType::Json).unwrap();
        let options = FormatOptions::default(); // default Indent::Spaces(2)
        let formatter = JsonFormatter;

        let mut output = Vec::new();
        formatter.format(&doc, &mut output, &options).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("{\n  \"key\": \"value\"\n}"));
    }

    #[test]
    fn test_json_formatter_compact() {
        let json = r#"{
            "key": "value"
        }"#;
        let doc = Document::parse(json, DocumentType::Json).unwrap();
        let options = FormatOptions {
            compact: true,
            ..FormatOptions::default()
        };
        let formatter = JsonFormatter;

        let mut output = Vec::new();
        formatter.format(&doc, &mut output, &options).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, r#"{"key":"value"}"#);
    }
}
