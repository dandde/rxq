//! XML/HTML to JSON conversion

use crate::types::{NodeRef, NodeType};
use serde_json::{Map, Value};

/// Convert a node and its descendants to a JSON Value.
/// Follows conventions:
/// - Elements -> Objects
/// - Attributes -> Keys prefixed with @
/// - Text content -> "#text" key if attributes exist, else string value
/// - Children -> Keys matching tag name
/// - Multiple children with same tag -> Array
pub fn to_json<'a, 'input>(node: NodeRef<'a, 'input>) -> Value {
    match node.node_type() {
        NodeType::Element => element_to_json(node),
        NodeType::Text => text_to_json(node),
        _ => Value::Null,
    }
}

fn element_to_json<'a, 'input>(node: NodeRef<'a, 'input>) -> Value {
    let mut map = Map::new();

    // 1. Process Attributes
    for (name, value) in node.attributes() {
        map.insert(format!("@{}", name), Value::String(value.into_owned()));
    }

    // 2. Process Children
    // We need to group children by tag name to handle arrays
    let mut text_content = String::new();
    let mut has_element_children = false;

    for child in node.children() {
        match child.node_type() {
            NodeType::Text => {
                if let Some(text) = child.text() {
                    text_content.push_str(&text);
                }
            }
            NodeType::Element => {
                has_element_children = true;
                if let Some(tag_cow) = child.tag_name() {
                    let tag = tag_cow.into_owned();
                    // Skip empty tags (e.g. processing instructions parsed by tl)
                    if tag.is_empty() {
                        continue;
                    }

                    let child_json = to_json(child);

                    if let Some(existing) = map.get_mut(&tag) {
                        if let Some(arr) = existing.as_array_mut() {
                            arr.push(child_json);
                        } else {
                            // Convert to array
                            let old_val = existing.clone();
                            map.insert(tag, Value::Array(vec![old_val, child_json]));
                        }
                    } else {
                        map.insert(tag, child_json);
                    }
                }
            }
            _ => {}
        }
    }

    // 3. Handle Text Content
    let trimmed_text = text_content.trim();
    if !trimmed_text.is_empty() {
        if map.is_empty() && !has_element_children {
            // Simple case: <tag>text</tag> -> "text"
            return Value::String(trimmed_text.to_string());
        } else {
            // Mixed content or attributes present: <tag attr="val">text</tag> -> {"@attr": "val", "#text": "text"}
            map.insert("#text".to_string(), Value::String(trimmed_text.to_string()));
        }
    } else if map.is_empty() && !has_element_children {
        // Empty element <tag/> -> null or ""? xq usually null
        return Value::Null;
    }

    Value::Object(map)
}

fn text_to_json<'a, 'input>(node: NodeRef<'a, 'input>) -> Value {
    node.text()
        .map(|t| Value::String(t.trim().to_string()))
        .unwrap_or(Value::Null)
}
