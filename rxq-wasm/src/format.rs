use crate::RxqDocument;
use rxq_core::format::XmlFormatter;
use rxq_core::{ColorMode, DocumentType, FormatOptions, Formatter, Indent};
use serde::Serialize;
use wasm_bindgen::prelude::*;

// We need a helper to emulate what rxq-cli/src/format.rs does but for WASM
// Since rxq-core's formatters take a Writer, we'll write to a Vec<u8> (buffer).

#[wasm_bindgen]
impl RxqDocument {
    pub fn format(&self, options: JsValue) -> Result<String, JsValue> {
        let doc = self.get_doc();

        // Deserialize options from JS
        let opts: FormatConfig = serde_wasm_bindgen::from_value(options).unwrap_or_default();

        let indent = if opts.useTabs.unwrap_or(false) {
            Indent::Tab
        } else {
            Indent::Spaces(opts.indent.unwrap_or(2) as u8)
        };

        let format_opts = FormatOptions {
            indent,
            color: if opts.color.unwrap_or(false) {
                ColorMode::Always
            } else {
                ColorMode::Never
            },
            compact: opts.compact.unwrap_or(false),
        };

        let mut buffer = Vec::new();

        // Choose formatter based on document type
        match doc.doc_type() {
            DocumentType::Xml | DocumentType::Html => {
                // rxq-core uses XmlFormatter for both (generic)
                let formatter = XmlFormatter;
                formatter
                    .format(doc, &mut buffer, &format_opts)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
            }
            DocumentType::Json => {
                return Err(JsValue::from_str(
                    "JSON formatting not fully implemented in rxq-wasm yet",
                ));
            }
        }

        String::from_utf8(buffer)
            .map_err(|e| JsValue::from_str(&format!("Invalid UTF-8 output: {}", e)))
    }

    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        let doc = self.get_doc();

        // rxq_core::json::to_json returns a serde_json::Value
        // We need to access it. Note: 'to_json' is in rxq_core root or a module.
        // Checking rxq-core structure: src/json.rs -> pub fn to_json

        let root = doc.root();
        let value = rxq_core::to_json(root);

        // Convert serde_json::Value to JsValue
        let serializer = serde_wasm_bindgen::Serializer::json_compatible();
        value
            .serialize(&serializer)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[derive(serde::Deserialize, Default)]
#[allow(non_snake_case)]
struct FormatConfig {
    indent: Option<u8>,
    useTabs: Option<bool>,
    color: Option<bool>,
    compact: Option<bool>,
}
