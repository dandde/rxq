//! # rxq-core: Zero-Copy XML/HTML Processing
//!
//! This crate provides zero-copy parsing and querying of XML and HTML documents.
//! All parsed data references the original input buffer, eliminating unnecessary
//! allocations and improving performance.
//!
//! ## Example
//!
//! ```
//! use rxq_core::{Document, DocumentType, Query, execute_query, QueryOptions};
//!
//! let xml = r#"
//! <users>
//!     <user status="active">
//!         <name>Alice</name>
//!         <email>alice@example.com</email>
//!     </user>
//! </users>
//! "#;
//!
//! // Parse (zero-copy)
//! let doc = Document::parse(xml, DocumentType::Xml).unwrap();
//!
//! // Query (returns iterator of borrowed references)
//! let query = Query::XPath("//user[@status='active']/name");
//! let results = execute_query(&doc, query, &QueryOptions::default()).unwrap();
//!
//! for node in results {
//!     println!("Name: {}", node.text().unwrap());
//! }
//! ```

pub mod types;
pub mod parser;
pub mod query;
pub mod format;
pub mod error;

// Re-export main types
pub use types::{Document, DocumentType, NodeRef, NodeType};
pub use query::{Query, QueryOptions, QueryIter, execute_query};
pub use format::{Formatter, FormatOptions, ColorMode, Indent};
pub use error::{ParseError, QueryError, FormatError};

#[cfg(feature = "json-output")]
pub mod json;

#[cfg(feature = "json-output")]
pub use json::to_json;
