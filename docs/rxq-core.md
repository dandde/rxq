# rxq-core API Documentation

`rxq-core` is a zero-copy XML and HTML processing library for Rust. It is designed for high performance and minimal memory overhead by borrowing data from the input string whenever possible.

## Core Concepts

- **Zero-Copy**: The library avoids allocating new strings for node names, attributes, and text content. Instead, it returns `Cow<'a, str>` referencing the original input.
- **Dual Lifetimes**: `NodeRef<'a, 'input>` uses two lifetimes:
  - `'a`: The lifetime of the borrow from the `VDom`.
  - `'input`: The lifetime of the original input string.

## Main Types

### `Document`
The entry point for parsing.

```rust
use rxq_core::{Document, DocumentType};

let xml = "<root><child>value</child></root>";
let doc = Document::parse(xml, DocumentType::Xml).unwrap();
```

### `NodeRef`
A light-weight reference to a node in the DOM.

```rust
let root = doc.root();
assert_eq!(root.tag_name().as_deref(), Some("root"));
```

### `Query`
Execute XPath-like or CSS selector queries.

```rust
use rxq_core::{Query, execute_query, QueryOptions};

let query = Query::XPath("//child");
let results = execute_query(&doc, query, &QueryOptions::default()).unwrap();
```

## Modules

- **types**: Core data structures (`Document`, `NodeRef`, `NodeType`).
- **parser**: Parsing utilities and options.
- **query**: Query engine implementation (XPath subset, CSS selectors).
- **format**: Beautification and specific formatters (`XmlFormatter`).
- **error**: Error definitions (`ParseError`, `QueryError`, `FormatError`).

## Examples

See `src/lib.rs` and `examples/` (if available) for more usage examples.
