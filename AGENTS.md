# AGENTS.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

`rxq` is a high-performance XML/HTML beautifier and content extractor written in Rust. It's designed as a faster alternative to `xq` (Go-based), using zero-copy parsing via the `tl` crate for optimal performance.

## Workspace Structure

This is a Cargo workspace with 4 crates:

- **rxq-core**: Core library providing zero-copy XML/HTML parsing, querying (XPath-like and CSS selectors), and formatting. All parsed data references the original input buffer to eliminate allocations.
- **rxq-cli**: Command-line interface binary (`rxq`). Provides backward compatibility with `xq` CLI.
- **rxq-wasm**: WebAssembly bindings for browser usage. Compiled to `cdylib` and consumed by the web frontend.
- **rxq-server**: Axum-based HTTP server that serves the React frontend and provides a CORS proxy endpoint for fetching external URLs.

The `www/` directory contains a React + TypeScript + Vite frontend that uses the WASM bindings.

## Core Architecture

### Zero-Copy Design
All document parsing is zero-copy. The `Document<'input>` type and `NodeRef<'a, 'input>` borrows from the original input string throughout the parsing and querying lifecycle. Never assume data is owned unless explicitly cloned.

### Query Execution
The query engine supports:
- XPath-like patterns: `//tag`, `//tag[@attr='value']`, `/root/child`, `/path/@attr`
- CSS selectors (delegated to `tl::VDom::query_selector`)

Query results are returned as lazy iterators (`QueryIter`) yielding `NodeRef` instances.

### Formatting
Formatters are document-type specific. The `format` module provides syntax highlighting and auto-indentation for XML/HTML, with JSON conversion available via the `json-output` feature.

## Common Development Commands

### Build and Run
```bash
# Build all workspace crates
cargo build

# Build release version
cargo build --release

# Run CLI (from workspace root)
cargo run -p rxq-cli -- <args>

# Example: format XML file
cargo run -p rxq-cli -- examples/sample.xml

# Example: XPath query
cargo run -p rxq-cli -- -x "//user/name" examples/sample.xml
```

### Testing
```bash
# Run all tests in workspace
cargo test

# Run tests for specific crate
cargo test -p rxq-core

# Run integration tests
cargo test -p rxq-cli --test integration

# Run with output
cargo test -- --nocapture
```

### Benchmarks
```bash
# Run criterion benchmarks (in rxq-core)
cargo bench -p rxq-core

# Run benchmarks for specific suite
cargo bench -p rxq-core --bench parsing
cargo bench -p rxq-core --bench querying

# Run custom benchmark script comparing to xq
./benchmark.sh
```

### WASM Development
```bash
# Build WASM package (from rxq-wasm directory)
cd rxq-wasm
wasm-pack build --target web --out-dir pkg

# From workspace root
cargo build -p rxq-wasm --target wasm32-unknown-unknown
```

### Web Frontend Development
```bash
# Install dependencies
cd www
npm install

# Development mode (Vite dev server)
npm run dev

# Build for production
npm run build

# Lint
npm run lint

# Preview production build
npm run preview
```

### Server Development
```bash
# Run the server (serves www/dist + provides proxy API)
cargo run -p rxq-server

# Or use the convenience script (builds frontend first)
./serve.sh
```

The server runs on `http://localhost:3000` and serves the React app with a `/api/proxy?url=<url>` endpoint for CORS-free XML/HTML fetching.

## Testing Strategy

- **Unit tests**: Inline with modules using `#[cfg(test)]`
- **Integration tests**: CLI integration tests in `rxq-cli/tests/integration.rs` using `assert_cmd` and `predicates`
- **Benchmarks**: Criterion benchmarks in `rxq-core/benches/` for parsing and querying performance

When adding new features, always add corresponding tests. For CLI changes, add integration tests.

## Feature Flags

The `json-output` feature in `rxq-core` enables JSON conversion functionality. Both `rxq-cli` and `rxq-wasm` enable this feature by default.

## Important Patterns

### Lifetimes
Pay close attention to lifetime parameters:
- `'input`: Lifetime of the original source string
- `'doc` or `'a`: Lifetime of the parsed document/VDom
- Query strings in CLI are leaked (`Box::leak`) to obtain `'static` lifetime, which is acceptable for short-lived CLI programs

### Error Handling
- Core library uses custom error types: `ParseError`, `QueryError`, `FormatError`
- CLI uses `anyhow::Result` for ergonomic error propagation with context

### Parser Details
The underlying `tl` crate parser handles both HTML and XML. The `DocumentType` enum guides formatting behavior, not parsing strictness.
