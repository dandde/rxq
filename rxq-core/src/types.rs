//! Core type definitions for zero-copy document representation

use crate::error::ParseError;
use std::borrow::Cow;
use tl::{HTMLTag, NodeHandle, ParserOptions, VDom};

/// Document type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentType {
    Xml,
    Html,
    Json,
}

/// Node type in the document tree
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Element,
    Text,
    Comment,
    Raw,
}

/// A parsed document with lifetime bound to source buffer.
/// All string data is borrowed from the original input.
pub struct Document<'input> {
    source: &'input str,
    vdom: VDom<'input>,
    doc_type: DocumentType,
}

impl<'input> Document<'input> {
    /// Parse a document from borrowed input (zero-copy).
    pub fn parse(source: &'input str, doc_type: DocumentType) -> Result<Self, ParseError> {
        let vdom = tl::parse(source, ParserOptions::default())
            .map_err(|e| ParseError::SyntaxError(format!("{:?}", e)))?;

        Ok(Self {
            source,
            vdom,
            doc_type,
        })
    }

    /// Auto-detect document type from content
    pub fn detect_type(source: &str) -> DocumentType {
        let trimmed = source.trim_start().to_lowercase();

        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            DocumentType::Json
        } else if trimmed.contains("<!doctype html") || trimmed.contains("<html") {
            DocumentType::Html
        } else {
            DocumentType::Xml
        }
    }

    /// Get the root node of the document
    pub fn root<'a>(&'a self) -> NodeRef<'a, 'input> {
        let parser = self.vdom.parser();
        let handle = self
            .vdom
            .children()
            .iter()
            .find(|h| h.get(parser).and_then(|node| node.as_tag()).is_some())
            .copied()
            .or_else(|| self.vdom.children().first().copied());

        let node = NodeRef {
            vdom: &self.vdom,
            handle,
        };

        // If root has empty tag name (e.g. <?xml ... ?> parsed by tl as container),
        // try to find the actual root element inside it.
        if let Some(name) = node.tag_name() {
            if name.is_empty() {
                if let Some(child) = node.children().find(|n| n.node_type() == NodeType::Element) {
                    return child;
                }
            }
        }

        node
    }

    /// Get the document type
    pub fn doc_type(&self) -> DocumentType {
        self.doc_type
    }

    /// Get reference to underlying VDom
    pub(crate) fn vdom(&self) -> &VDom<'input> {
        &self.vdom
    }

    /// Get the original source string
    pub fn source(&self) -> &'input str {
        self.source
    }
}

/// A reference to a node in the document tree.
#[derive(Clone, Copy)]
pub struct NodeRef<'a, 'input> {
    vdom: &'a VDom<'input>,
    handle: Option<NodeHandle>,
}

impl<'a, 'input> NodeRef<'a, 'input> {
    /// Create a new NodeRef
    pub(crate) fn new(vdom: &'a VDom<'input>, handle: Option<NodeHandle>) -> Self {
        Self { vdom, handle }
    }

    // Note: handle() getter removed as it was unused and caused warnings.
    // If needed in future, uncomment:
    // pub(crate) fn handle(&self) -> Option<NodeHandle> { self.handle }

    /// Get node type
    pub fn node_type(&self) -> NodeType {
        match self.handle.and_then(|h| h.get(self.vdom.parser())) {
            Some(node) if node.as_tag().is_some() => NodeType::Element,
            Some(node) if node.as_comment().is_some() => NodeType::Comment,
            Some(node) if node.as_raw().is_some() => NodeType::Text,
            _ => NodeType::Raw,
        }
    }

    /// Get the tag name (if this is an element node)
    pub fn tag_name(&self) -> Option<Cow<'a, str>> {
        self.handle
            .and_then(|h| h.get(self.vdom.parser()))
            .and_then(|node| node.as_tag())
            .map(|tag| tag.name().as_utf8_str())
    }

    /// Get inner HTML as String
    pub fn inner_html(&self) -> String {
        self.handle
            .and_then(|h| h.get(self.vdom.parser()))
            .and_then(|node| node.as_tag())
            .map(|tag| tag.inner_html(self.vdom.parser()).to_string())
            .unwrap_or_default()
    }

    /// Get outer HTML as String
    pub fn outer_html(&self) -> String {
        self.handle
            .and_then(|h| h.get(self.vdom.parser()))
            .and_then(|node| node.as_tag())
            .map(|tag| tag.outer_html(self.vdom.parser()).to_string())
            .unwrap_or_default()
    }

    /// Get text content (recursively collects all text nodes)
    pub fn text(&self) -> Option<String> {
        self.handle
            .and_then(|h| h.get(self.vdom.parser()))
            .map(|node| {
                if let Some(tag) = node.as_tag() {
                    self.collect_text_recursive(tag)
                } else if let Some(raw) = node.as_raw() {
                    raw.as_utf8_str().to_string()
                } else {
                    String::new()
                }
            })
    }

    /// Recursively collect text content
    fn collect_text_recursive(&self, tag: &HTMLTag) -> String {
        let mut result = String::new();
        let children = tag.children();
        for child_handle in children.top().iter() {
            if let Some(child) = child_handle.get(self.vdom.parser()) {
                if let Some(child_tag) = child.as_tag() {
                    result.push_str(&self.collect_text_recursive(child_tag));
                } else if let Some(raw) = child.as_raw() {
                    let text_cow = raw.as_utf8_str();
                    let text = text_cow.as_ref();
                    if !text.starts_with("<!--") {
                        result.push_str(text);
                    }
                }
            }
        }
        result
    }

    /// Get an attribute value by name (zero-copy)
    pub fn attr(&self, name: &'a str) -> Option<Cow<'a, str>> {
        self.handle
            .and_then(|h| h.get(self.vdom.parser()))
            .and_then(|node| node.as_tag())
            .and_then(|tag| {
                tag.attributes()
                    .get(name)
                    .flatten()
                    .map(|bytes| bytes.as_utf8_str())
            })
    }

    /// Get comment content
    pub fn comment(&self) -> Option<Cow<'a, str>> {
        self.handle
            .and_then(|h| h.get(self.vdom.parser()))
            .and_then(|node| node.as_comment())
            .map(|comment| comment.as_utf8_str())
    }

    /// Get all attributes as an iterator
    pub fn attributes(&self) -> impl Iterator<Item = (Cow<'a, str>, Cow<'a, str>)> + '_ {
        self.handle
            .and_then(|h| h.get(self.vdom.parser()))
            .and_then(|node| node.as_tag())
            .into_iter()
            .flat_map(|tag| {
                tag.attributes().iter().filter_map(|(k, v)| {
                    v.as_ref().map(|val| {
                        (
                            k.clone(),   // k is Cow, so clone to return
                            val.clone(), // val is &Cow, so clone to return
                        )
                    })
                })
            })
    }

    /// Iterate over child nodes
    pub fn children(&self) -> impl Iterator<Item = NodeRef<'a, 'input>> + '_ {
        self.handle
            .and_then(|h| h.get(self.vdom.parser()))
            .and_then(|node| node.as_tag())
            .into_iter()
            .flat_map(move |tag| {
                // Collect handles
                tag.children()
                    .top()
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(move |child_handle| NodeRef {
                        vdom: self.vdom,
                        handle: Some(child_handle),
                    })
            })
    }

    /// Check if this node has children
    pub fn has_children(&self) -> bool {
        self.handle
            .and_then(|h| h.get(self.vdom.parser()))
            .and_then(|node| node.as_tag())
            .map(|tag| tag.children().top().iter().next().is_some())
            .unwrap_or(false)
    }

    /// Get parent node if available
    pub fn parent(&self) -> Option<NodeRef<'a, 'input>> {
        None
    }
}

impl<'a, 'input> std::fmt::Debug for NodeRef<'a, 'input> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeRef")
            .field("tag_name", &self.tag_name())
            .field("node_type", &self.node_type())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_parse() {
        let xml = "<root><child>value</child></root>";
        let doc = Document::parse(xml, DocumentType::Xml).unwrap();
        assert_eq!(doc.doc_type(), DocumentType::Xml);
        assert_eq!(doc.source(), xml);
    }

    #[test]
    fn test_auto_detect_html() {
        let html = "<!DOCTYPE html><html><body>test</body></html>";
        assert_eq!(Document::detect_type(html), DocumentType::Html);

        // Case insensitivity
        assert_eq!(
            Document::detect_type("<!doctype html>"),
            DocumentType::Html
        );
        assert_eq!(
            Document::detect_type("<HTML><BODY></BODY></HTML>"),
            DocumentType::Html
        );

        // Leading whitespace
        assert_eq!(
            Document::detect_type("   <html></html>"),
            DocumentType::Html
        );
    }

    #[test]
    fn test_auto_detect_json() {
        assert_eq!(Document::detect_type("{}"), DocumentType::Json);
        assert_eq!(
            Document::detect_type("  {\"key\": \"value\"}"),
            DocumentType::Json
        );
        assert_eq!(Document::detect_type("[]"), DocumentType::Json);
        assert_eq!(Document::detect_type("\n [1, 2, 3]"), DocumentType::Json);
    }

    #[test]
    fn test_auto_detect_xml() {
        assert_eq!(Document::detect_type("<root></root>"), DocumentType::Xml);
        assert_eq!(
            Document::detect_type("<?xml version=\"1.0\"?><root></root>"),
            DocumentType::Xml
        );
        assert_eq!(Document::detect_type("  <root/>"), DocumentType::Xml);
        // Default case is XML
        assert_eq!(Document::detect_type("just some text"), DocumentType::Xml);
    }

    #[test]
    fn test_node_tag_name() {
        let xml = "<root><child>value</child></root>";
        let doc = Document::parse(xml, DocumentType::Xml).unwrap();
        let root = doc.root();
        assert_eq!(root.tag_name().as_deref(), Some("root"));
    }

    #[test]
    fn test_node_text() {
        let xml = "<root><child>value</child></root>";
        let doc = Document::parse(xml, DocumentType::Xml).unwrap();
        let root = doc.root();
        // Note: collect_text_recursive behaviour might include newlines depending on parsing
        let text = root.text().unwrap();
        assert_eq!(text.trim(), "value");
    }

    #[test]
    fn test_node_children() {
        // Use standard tags to avoid ambiguity in default parser mode
        let xml = "<div><p>1</p><p>2</p></div>";
        let doc = Document::parse(xml, DocumentType::Html).unwrap();
        let root = doc.root();

        let children: Vec<_> = root
            .children()
            .filter(|n| n.node_type() == NodeType::Element)
            .collect();

        assert_eq!(children.len(), 2);
        assert_eq!(children[0].tag_name().as_deref(), Some("p"));
        assert_eq!(children[1].tag_name().as_deref(), Some("p"));
    }
}
