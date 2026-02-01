use crate::RxqDocument;
use rxq_core::{execute_query, Query, QueryOptions};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl RxqDocument {
    pub fn query(&self, expression: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let doc = self.get_doc();

        // Deserialize options
        let opts: QueryConfig = serde_wasm_bindgen::from_value(options).unwrap_or_default();

        // Determine query type
        let q_obj = match opts.type_.as_deref() {
            Some("xpath") => Query::XPath(expression),
            Some("css") => Query::CssSelector(expression),
            _ => {
                // Auto-detect or default to XPath?
                // Let's assume XPath if starts with /, else CSS?
                // Simple heuristic for now or demand explicit type.
                // Defaulting to XPath as per CLI habit if unstated, or CSS if no slashes.
                if expression.starts_with('/') {
                    Query::XPath(expression)
                } else {
                    Query::CssSelector(expression)
                }
            }
        };

        let result_opts = QueryOptions {
            with_tags: opts.withTags.unwrap_or(false),
            extract_attr: opts.attribute, // Option<String>
        };

        let results = execute_query(doc, q_obj, &result_opts)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Collect results into a Vec<String>
        let output: Vec<String> = results
            .filter_map(|node| {
                if let Some(attr) = result_opts.extract_attr.as_ref() {
                    // Extract attribute
                    node.attr(attr).map(|c| c.into_owned())
                } else if result_opts.with_tags {
                    // Outer HTML
                    Some(node.outer_html())
                } else {
                    // Text content
                    node.text()
                }
            })
            .collect();

        Ok(serde_wasm_bindgen::to_value(&output).unwrap())
    }
}

#[derive(serde::Deserialize, Default)]
#[allow(non_snake_case)]
struct QueryConfig {
    #[serde(rename = "type")]
    type_: Option<String>, // "xpath" or "css"
    withTags: Option<bool>,
    attribute: Option<String>,
}
