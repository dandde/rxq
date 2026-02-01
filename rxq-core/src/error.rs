//! Error types for rxq-core

use thiserror::Error;

/// Errors that can occur during document parsing
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("XML/HTML syntax error: {0}")]
    SyntaxError(String),
    
    #[error("unsupported document type")]
    UnsupportedType,
    
    #[error("character encoding error: {0}")]
    EncodingError(String),
    
    #[error("empty or invalid input")]
    EmptyInput,
}

/// Errors that can occur during query execution
#[derive(Error, Debug)]
pub enum QueryError {
    #[error("invalid XPath expression: {0}")]
    InvalidXPath(String),
    
    #[error("invalid CSS selector: {0}")]
    InvalidSelector(String),
    
    #[error("query execution failed: {0}")]
    ExecutionError(String),
    
    #[error("node not found")]
    NodeNotFound,
    
    #[error("attribute '{0}' not found")]
    AttributeNotFound(String),
}

/// Errors that can occur during formatting
#[derive(Error, Debug)]
pub enum FormatError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("formatting failed: {0}")]
    FormatFailed(String),
    
    #[error("invalid indentation: {0}")]
    InvalidIndent(String),
    
    #[error("color output not supported")]
    ColorNotSupported,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ParseError::SyntaxError("unexpected token".to_string());
        assert_eq!(err.to_string(), "XML/HTML syntax error: unexpected token");
    }

    #[test]
    fn test_error_from() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let format_err = FormatError::from(io_err);
        assert!(matches!(format_err, FormatError::IoError(_)));
    }
}
