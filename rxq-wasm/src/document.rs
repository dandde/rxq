use rxq_core::{Document, DocumentType};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RxqDocument {
    // We keep the source string alive here.
    // The Document uses references to this string.
    #[allow(dead_code)]
    source: String,

    // Self-referential pointer to the Document.
    // We treat it as 'static because we manually ensure 'source is alive
    // as long as this pointer is used.
    doc_ptr: *mut Document<'static>,
}

#[wasm_bindgen]
impl RxqDocument {
    #[wasm_bindgen(constructor)]
    pub fn new(source: String, doc_type: Option<String>) -> Result<RxqDocument, JsValue> {
        // 1. Determine document type
        let dt = if let Some(t) = doc_type {
            match t.to_lowercase().as_str() {
                "xml" => DocumentType::Xml,
                "html" => DocumentType::Html,
                "json" => DocumentType::Json,
                _ => return Err(JsValue::from_str("Invalid document type")),
            }
        } else {
            Document::detect_type(&source)
        };

        // 2. Transmute source to 'static to trick the compiler.
        // SAFETY: We own 'source' in the struct, so it won't move or drop
        // while the struct exists.
        let source_static: &'static str = unsafe { std::mem::transmute(source.as_str()) };

        // 3. Parse document with 'static lifetime
        let doc =
            Document::parse(source_static, dt).map_err(|e| JsValue::from_str(&e.to_string()))?;

        // 4. Box it and leak to get a raw pointer
        let doc_boxed = Box::new(doc);
        let doc_ptr = Box::into_raw(doc_boxed);

        Ok(RxqDocument { source, doc_ptr })
    }

    pub fn free(mut self) {
        // Re-construct the box to drop it
        if !self.doc_ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(self.doc_ptr);
            }
            self.doc_ptr = std::ptr::null_mut();
        }
        // 'source' will be dropped naturally when struct is dropped
    }
}

// Internal helper to get reference to document
impl RxqDocument {
    pub(crate) fn get_doc(&self) -> &Document<'static> {
        unsafe { &*self.doc_ptr }
    }
}

impl Drop for RxqDocument {
    fn drop(&mut self) {
        // Ensure we clean up if free() wasn't called mostly
        if !self.doc_ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(self.doc_ptr);
            }
        }
    }
}
