//! Query execution engine for CSS selectors and XPath-like expressions

use crate::error::QueryError;
use crate::types::{Document, NodeRef};
use tl::ParserOptions;

/// Query specification (type-safe)
#[derive(Debug, Clone)]
pub enum Query<'q> {
    /// XPath-like expression (multiple results)
    /// Currently supports simple patterns like //tag, //tag[@attr='value'], /root/child
    XPath(&'q str),

    /// XPath-like expression (single result only)
    Extract(&'q str),

    /// CSS selector (uses tl's query selector)
    CssSelector(&'q str),
}

/// Options for query execution
#[derive(Default, Clone, Debug)]
pub struct QueryOptions {
    /// Return full node content (with tags) vs text only
    pub with_tags: bool,

    /// For CSS queries: attribute to extract
    pub extract_attr: Option<String>,
}

/// Lazy iterator over query results
pub struct QueryIter<'doc, 'input> {
    inner: Box<dyn Iterator<Item = NodeRef<'doc, 'input>> + 'doc>,
}

impl<'doc, 'input> Iterator for QueryIter<'doc, 'input> {
    type Item = NodeRef<'doc, 'input>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// Execute a query on a document
pub fn execute_query<'doc, 'input>(
    doc: &'doc Document<'input>,
    query: Query<'_>,
    _options: &QueryOptions,
) -> Result<QueryIter<'doc, 'input>, QueryError> {
    match query {
        Query::XPath(expr) | Query::Extract(expr) => execute_xpath_like(doc, expr),
        Query::CssSelector(selector) => execute_css_selector(doc, selector),
    }
}

/// Execute CSS selector query
fn execute_css_selector<'doc, 'input>(
    doc: &'doc Document<'input>,
    selector: &str,
) -> Result<QueryIter<'doc, 'input>, QueryError> {
    // Use tl's query selector
    let vdom = doc.vdom();
    let _parser_options = ParserOptions::default();

    // Query all matching nodes
    let results: Vec<NodeRef<'doc, 'input>> = vdom
        .query_selector(selector)
        .ok_or_else(|| QueryError::InvalidSelector(selector.to_string()))?
        .map(|handle| NodeRef::new(vdom, Some(handle.clone())))
        .collect();

    Ok(QueryIter {
        inner: Box::new(results.into_iter()),
    })
}

/// Execute XPath-like query
/// This is a simplified implementation that supports common patterns
fn execute_xpath_like<'doc, 'input>(
    doc: &'doc Document<'input>,
    expr: &str,
) -> Result<QueryIter<'doc, 'input>, QueryError> {
    let vdom = doc.vdom();

    // Parse the expression
    let pattern = parse_xpath_pattern(expr)?;

    // Execute based on pattern type
    let results: Vec<NodeRef<'doc, 'input>> = match pattern {
        XPathPattern::DescendantTag(tag) => {
            // //tag pattern - find all descendants with this tag
            find_descendants_by_tag(vdom, doc.root(), &tag)
        }
        XPathPattern::DescendantAttr(tag, attr, value) => {
            // //tag[@attr='value'] pattern
            find_descendants_by_attr(vdom, doc.root(), &tag, &attr, &value)
        }
        XPathPattern::AttributeValue(path, attr) => {
            // /path/to/tag/@attr pattern
            let root = doc.root();
            if !path.is_empty() && root.tag_name().as_deref() == Some(path[0].as_str()) {
                find_attribute_value(vdom, root, &path[1..], &attr)
            } else {
                vec![]
            }
        }
        XPathPattern::AbsolutePath(path) => {
            // /root/child/grandchild pattern
            let root = doc.root();
            if !path.is_empty() && root.tag_name().as_deref() == Some(path[0].as_str()) {
                find_by_absolute_path(vdom, root, &path[1..])
            } else {
                vec![]
            }
        }
    };

    Ok(QueryIter {
        inner: Box::new(results.into_iter()),
    })
}

/// Simplified XPath patterns we support
#[derive(Debug)]
enum XPathPattern {
    /// //tag
    DescendantTag(String),
    /// //tag[@attr='value']
    DescendantAttr(String, String, String),
    /// /path/@attr
    AttributeValue(Vec<String>, String),
    /// /root/child/grandchild
    AbsolutePath(Vec<String>),
}

/// Parse XPath expression into a pattern
fn parse_xpath_pattern(expr: &str) -> Result<XPathPattern, QueryError> {
    let expr = expr.trim();

    if expr.starts_with("//") {
        // Descendant pattern
        let rest = &expr[2..];

        if let Some(bracket_pos) = rest.find('[') {
            // Has predicate: //tag[@attr='value']
            let tag = rest[..bracket_pos].to_string();
            let predicate = &rest[bracket_pos + 1..];

            if let Some(end) = predicate.find(']') {
                let pred_inner = &predicate[..end];
                if let Some((attr, value)) = parse_attribute_predicate(pred_inner) {
                    return Ok(XPathPattern::DescendantAttr(tag, attr, value));
                }
            }

            Err(QueryError::InvalidXPath(expr.to_string()))
        } else {
            // Simple tag: //tag
            Ok(XPathPattern::DescendantTag(rest.to_string()))
        }
    } else if expr.starts_with('/') && expr.contains("/@") {
        // Attribute extraction: /path/@attr
        let parts: Vec<&str> = expr.split("/@").collect();
        if parts.len() == 2 {
            let path = parts[0][1..].split('/').map(String::from).collect();
            let attr = parts[1].to_string();
            Ok(XPathPattern::AttributeValue(path, attr))
        } else {
            Err(QueryError::InvalidXPath(expr.to_string()))
        }
    } else if expr.starts_with('/') {
        // Absolute path: /root/child
        let path: Vec<String> = expr[1..].split('/').map(String::from).collect();
        Ok(XPathPattern::AbsolutePath(path))
    } else {
        Err(QueryError::InvalidXPath(expr.to_string()))
    }
}

/// Parse attribute predicate like @attr='value' or @attr="value"
fn parse_attribute_predicate(pred: &str) -> Option<(String, String)> {
    if !pred.starts_with('@') {
        return None;
    }

    let pred = pred[1..].trim();

    // Try both ' and " quotes
    for quote in &['\'', '"'] {
        if let Some(eq_pos) = pred.find('=') {
            let attr = pred[..eq_pos].trim().to_string();
            let value_part = pred[eq_pos + 1..].trim();

            if value_part.starts_with(*quote) && value_part.ends_with(*quote) {
                let value = value_part[1..value_part.len() - 1].to_string();
                return Some((attr, value));
            }
        }
    }

    None
}

/// Find all descendants with a given tag name
fn find_descendants_by_tag<'a, 'input>(
    vdom: &'a tl::VDom<'input>,
    node: NodeRef<'a, 'input>,
    tag: &str,
) -> Vec<NodeRef<'a, 'input>> {
    let mut results = Vec::new();
    find_descendants_by_tag_impl(vdom, node, tag, &mut results);
    results
}

fn find_descendants_by_tag_impl<'a, 'input>(
    vdom: &'a tl::VDom<'input>,
    node: NodeRef<'a, 'input>,
    tag: &str,
    results: &mut Vec<NodeRef<'a, 'input>>,
) {
    // node.tag_name() returns Option<Cow>, use as_deref to compare with &str
    if node.tag_name().as_deref() == Some(tag) {
        results.push(node);
    }

    for child in node.children() {
        find_descendants_by_tag_impl(vdom, child, tag, results);
    }
}

/// Find all descendants with a tag and specific attribute value
fn find_descendants_by_attr<'a, 'input>(
    vdom: &'a tl::VDom<'input>,
    node: NodeRef<'a, 'input>,
    tag: &str,
    attr: &str,
    value: &str,
) -> Vec<NodeRef<'a, 'input>> {
    let mut results = Vec::new();
    find_descendants_by_attr_impl(vdom, node, tag, attr, value, &mut results);
    results
}

fn find_descendants_by_attr_impl<'a, 'input>(
    vdom: &'a tl::VDom<'input>,
    node: NodeRef<'a, 'input>,
    tag: &str,
    attr: &str,
    value: &str,
    results: &mut Vec<NodeRef<'a, 'input>>,
) {
    if node.tag_name().as_deref() == Some(tag) && node.attr(attr).as_deref() == Some(value) {
        results.push(node);
    }

    for child in node.children() {
        find_descendants_by_attr_impl(vdom, child, tag, attr, value, results);
    }
}

/// Find nodes by absolute path and extract attribute values
fn find_attribute_value<'a, 'input>(
    vdom: &'a tl::VDom<'input>,
    node: NodeRef<'a, 'input>,
    path: &[String],
    attr: &str,
) -> Vec<NodeRef<'a, 'input>> {
    let mut results = Vec::new();
    find_attribute_value_impl(vdom, node, path, attr, &mut results);
    results
}

fn find_attribute_value_impl<'a, 'input>(
    vdom: &'a tl::VDom<'input>,
    node: NodeRef<'a, 'input>,
    path: &[String],
    attr: &str,
    results: &mut Vec<NodeRef<'a, 'input>>,
) {
    if path.is_empty() {
        // We're at the target node, create a pseudo-node with the attribute value
        if node.attr(attr).is_some() {
            // For attribute values, we return the node itself
            // The caller will extract the attribute value
            results.push(node);
        }
        return;
    }

    // Match the first path component
    let target = &path[0];
    for child in node.children() {
        if child.tag_name().as_deref() == Some(target.as_str()) {
            find_attribute_value_impl(vdom, child, &path[1..], attr, results);
        }
    }
}

/// Find nodes by absolute path
fn find_by_absolute_path<'a, 'input>(
    vdom: &'a tl::VDom<'input>,
    node: NodeRef<'a, 'input>,
    path: &[String],
) -> Vec<NodeRef<'a, 'input>> {
    let mut results = Vec::new();
    find_by_absolute_path_impl(vdom, node, path, &mut results);
    results
}

fn find_by_absolute_path_impl<'a, 'input>(
    vdom: &'a tl::VDom<'input>,
    node: NodeRef<'a, 'input>,
    path: &[String],
    results: &mut Vec<NodeRef<'a, 'input>>,
) {
    if path.is_empty() {
        results.push(node);
        return;
    }

    let target = &path[0];
    for child in node.children() {
        if child.tag_name().as_deref() == Some(target.as_str()) {
            find_by_absolute_path_impl(vdom, child, &path[1..], results);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DocumentType;

    #[test]
    fn test_css_selector() {
        let html = r#"
            <html>
                <body>
                    <p class="test">First</p>
                    <p class="test">Second</p>
                    <div>Other</div>
                </body>
            </html>
        "#;

        let doc = Document::parse(html, DocumentType::Html).unwrap();
        let query = Query::CssSelector("p.test");
        let results: Vec<_> = execute_query(&doc, query, &QueryOptions::default())
            .unwrap()
            .collect();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_xpath_descendant_tag() {
        let xml = r#"
            <root>
                <item>1</item>
                <nested>
                    <item>2</item>
                </nested>
            </root>
        "#;

        let doc = Document::parse(xml, DocumentType::Xml).unwrap();
        let query = Query::XPath("//item");
        let results: Vec<_> = execute_query(&doc, query, &QueryOptions::default())
            .unwrap()
            .collect();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_xpath_attribute_predicate() {
        let xml = r#"
            <root>
                <user status="active">Alice</user>
                <user status="inactive">Bob</user>
                <user status="active">Charlie</user>
            </root>
        "#;

        let doc = Document::parse(xml, DocumentType::Xml).unwrap();
        let query = Query::XPath("//user[@status='active']");
        let results: Vec<_> = execute_query(&doc, query, &QueryOptions::default())
            .unwrap()
            .collect();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_parse_xpath_patterns() {
        let pattern = parse_xpath_pattern("//tag").unwrap();
        assert!(matches!(pattern, XPathPattern::DescendantTag(_)));

        let pattern = parse_xpath_pattern("//tag[@attr='value']").unwrap();
        assert!(matches!(pattern, XPathPattern::DescendantAttr(_, _, _)));

        let pattern = parse_xpath_pattern("/root/child/@attr").unwrap();
        assert!(matches!(pattern, XPathPattern::AttributeValue(_, _)));
    }
}
