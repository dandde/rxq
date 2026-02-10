# rxq

A high-performance XML/HTML beautifier and content extractor, written in Rust.

> **Note**: `rxq` was heavily inspired by [`xq`](https://github.com/sibprogrammer/xq). It aims to provide a faster, high-performance alternative while maintaining a similar CLI experience.

## Features

- **Rust**: Built with Rust and zero-copy parsing for performance.
- **Versatile Formatting**:
    - **XML**: Syntax highlighting, auto-indentation, and inline text preservation.
    - **HTML**: Graceful handling of HTML5 documents.
    - **JSON**: Convert XML/HTML structure to JSON instantly (~38x faster than existing tools).
- **Querying**:
    - **XPath**: Extract data using standard XPath syntax (e.g., `//user/name`).
    - **CSS Selectors**: Query elements using familiar CSS selectors (e.g., `div.content`).
- **Compatibility**: Supports standard flags for colorization, indentation control, and compact output.

## Usage

```bash
# Example 
cargo run --example zero_copy_demo -p rxq-core
```

### Basic Formatting

Format an XML file from standard input or file:

```bash
# From file
rxq input.xml

# From stdin
cat input.xml | rxq
```

### Data Extraction (XPath)

Extract specific data using XPath queries:

```bash
# Extract all names
rxq -x //user/name input.xml

# Extract attribute values
rxq -x "//@status" input.xml
```

### JSON Conversion

Convert XML or HTML to JSON:

```bash
rxq --json input.xml
```

### Benchmarks

`rxq` is significantly faster than its Go-based predecessor `xq`:

- **JSON Conversion**: ~38x faster
- **Formatting**: ~11x faster
- **Extraction**: ~3.7x faster

(Benchmarks run on a 480KB XML file)

## License

MIT License. See [LICENSE](LICENSE) for details.
