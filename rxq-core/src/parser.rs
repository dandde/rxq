//! Parser utilities and helpers
//!
//! This module provides additional parsing utilities beyond the basic
//! Document::parse() function. Future extensions may include:
//! - Custom encoding detection
//! - Streaming parsers
//! - Fragment parsing
//! - XML namespace handling

use crate::types::{Document, DocumentType};
use crate::error::ParseError;

/// Parse options for fine-grained control
#[derive(Default, Clone)]
pub struct ParseOptions {
    /// Detect document type automatically
    pub auto_detect: bool,
    
    /// Strict mode (fail on any error)
    pub strict: bool,
    
    /// Maximum document size (bytes)
    pub max_size: Option<usize>,
}

/// Parse with options
pub fn parse_with_options<'input>(
    source: &'input str,
    doc_type: DocumentType,
    options: &ParseOptions,
) -> Result<Document<'input>, ParseError> {
    // Check size limit
    if let Some(max_size) = options.max_size {
        if source.len() > max_size {
            return Err(ParseError::SyntaxError(
                format!("Document size {} exceeds maximum {}", source.len(), max_size)
            ));
        }
    }
    
    // Auto-detect if requested
    let doc_type = if options.auto_detect {
        Document::detect_type(source)
    } else {
        doc_type
    };
    
    Document::parse(source, doc_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_size_limit() {
        let xml = "<root><child>value</child></root>";
        let options = ParseOptions {
            max_size: Some(10),
            ..Default::default()
        };
        
        let result = parse_with_options(xml, DocumentType::Xml, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_auto_detect() {
        let html = "<!DOCTYPE html><html><body>test</body></html>";
        let options = ParseOptions {
            auto_detect: true,
            ..Default::default()
        };
        
        let doc = parse_with_options(html, DocumentType::Xml, &options).unwrap();
        assert_eq!(doc.doc_type(), DocumentType::Html);
    }
}
