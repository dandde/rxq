//! rxq: Zero-copy XML/HTML beautifier and content extractor
//!
//! Command-line interface providing full backward compatibility with xq

use anyhow::{Context, Result};
use clap::builder::styling;
use clap::builder::Styles;
use clap::Parser;
use std::fs::File;
use std::io::{stdin, stdout, BufWriter, Read, Write};
use std::path::PathBuf;

use rxq_core::format::format_query_results;
use rxq_core::{
    execute_query, ColorMode, Document, DocumentType, FormatOptions, Formatter, Indent, Query,
    QueryOptions,
};

mod formatters;
use formatters::get_formatter;

fn my_styles() -> Styles {
    styling::Styles::styled()
        .header(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
        .usage(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
        .literal(styling::AnsiColor::Blue.on_default() | styling::Effects::BOLD)
        .placeholder(styling::AnsiColor::Cyan.on_default())
}

#[derive(Parser, Debug)]
#[command(name = "rxq")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Command-line XML and HTML beautifier and content extractor")]
#[command(long_about = None, styles = my_styles())]
struct Cli {
    /// Input file (stdin if not provided)
    pub file: Option<PathBuf>,

    /// XPath query (multiple results)
    #[arg(short = 'x', long = "xpath")]
    pub xpath: Option<String>,

    /// XPath query (single result)
    #[arg(short = 'e', long = "extract")]
    pub extract: Option<String>,

    /// CSS selector query
    #[arg(short = 'q', long = "query")]
    pub css_query: Option<String>,

    /// Extract attribute for CSS query
    #[arg(short = 'a', long = "attr", requires = "css_query")]
    pub css_attr: Option<String>,

    /// Return node content with tags
    #[arg(short = 'n', long = "node")]
    pub with_tags: bool,

    /// Use HTML formatter
    #[arg(short = 'm', long = "html")]
    pub html: bool,

    /// Output as JSON
    #[arg(short = 'j', long = "json")]
    pub json: bool,

    /// Indent spaces (0-8)
    #[arg(long = "indent", default_value = "2", value_parser = validate_indent)]
    pub indent: u8,

    /// Use tabs for indentation
    #[arg(long = "tab", conflicts_with = "indent")]
    pub use_tabs: bool,

    /// Force color output
    #[arg(short = 'c', long = "color", conflicts_with = "no_color")]
    pub force_color: bool,

    /// Disable color output
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Format file in place
    #[arg(short = 'i', long = "in-place", requires = "file")]
    pub in_place: bool,

    /// Compact JSON output
    #[arg(long = "compact")]
    pub compact: bool,

    /// Maximum nesting depth for JSON output
    #[arg(short = 'd', long = "depth", default_value = "-1")]
    pub depth: i32,
}

fn validate_indent(s: &str) -> Result<u8, String> {
    let val: u8 = s.parse().map_err(|_| "must be a number")?;
    if val > 8 {
        Err("indent should be between 0-8 spaces".to_string())
    } else {
        Ok(val)
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Read input (either from file or stdin)
    let input = read_input(&cli)?;

    // Detect or use specified document type
    let doc_type = determine_doc_type(&cli, &input);

    // Parse document (zero-copy)
    let doc = Document::parse(&input, doc_type).context("Failed to parse document")?;

    // Build query if specified
    let query = build_query(&cli)?;

    // Format options
    let format_opts = FormatOptions {
        indent: if cli.use_tabs {
            Indent::Tab
        } else {
            Indent::Spaces(cli.indent)
        },
        color: color_mode(&cli),
        compact: cli.compact,
    };

    // Prepare output writer
    let mut output: Box<dyn Write> = if cli.in_place {
        let path = cli.file.as_ref().unwrap(); // Safe: requires="file"
        Box::new(BufWriter::new(
            File::create(path).context("Failed to open file for writing")?,
        ))
    } else {
        Box::new(stdout().lock())
    };

    // Execute query or format entire document
    if let Some(query) = query {
        let query_opts = QueryOptions {
            with_tags: cli.with_tags,
            extract_attr: cli.css_attr.clone(),
        };

        let results = execute_query(&doc, query, &query_opts).context("Query execution failed")?;

        // Use generic writer (Box<dyn Write> implements Write)
        format_query_results(results, &mut output, &query_opts, &format_opts)
            .context("Failed to format query results")?;
    } else {
        // Format entire document
        let formatter = if cli.json {
            use formatters::JsonFormatter;
            formatters::DocFormatter::Json(JsonFormatter)
        } else {
            get_formatter(doc_type)
        };

        formatter
            .format(&doc, &mut output, &format_opts)
            .context("Failed to format document")?;
    }

    output.flush()?;
    Ok(())
}

/// Read input from file or stdin
fn read_input(cli: &Cli) -> Result<String> {
    let mut input = String::new();

    if let Some(path) = &cli.file {
        File::open(path)
            .with_context(|| format!("Failed to open file: {}", path.display()))?
            .read_to_string(&mut input)
            .context("Failed to read file")?;
    } else {
        // Check if stdin is a terminal (no piped input)
        if atty::is(atty::Stream::Stdin) {
            anyhow::bail!("No input provided. Use --help for usage information.");
        }

        stdin()
            .read_to_string(&mut input)
            .context("Failed to read from stdin")?;
    }

    if input.is_empty() {
        anyhow::bail!("Input is empty");
    }

    Ok(input)
}

/// Determine document type based on CLI flags and content
fn determine_doc_type(cli: &Cli, input: &str) -> DocumentType {
    if cli.html {
        DocumentType::Html
    } else {
        Document::detect_type(input)
    }
}

/// Build query from CLI arguments
fn build_query(cli: &Cli) -> Result<Option<Query<'static>>> {
    if let Some(xpath) = &cli.xpath {
        // Leak the string to get 'static lifetime
        // This is acceptable for CLI use since it lives for program duration
        let leaked: &'static str = Box::leak(xpath.clone().into_boxed_str());
        Ok(Some(Query::XPath(leaked)))
    } else if let Some(extract) = &cli.extract {
        let leaked: &'static str = Box::leak(extract.clone().into_boxed_str());
        Ok(Some(Query::Extract(leaked)))
    } else if let Some(css) = &cli.css_query {
        let leaked: &'static str = Box::leak(css.clone().into_boxed_str());
        Ok(Some(Query::CssSelector(leaked)))
    } else {
        Ok(None)
    }
}

/// Determine color mode from CLI flags
fn color_mode(cli: &Cli) -> ColorMode {
    if cli.in_place {
        // Never use colors for in-place editing
        ColorMode::Never
    } else if cli.force_color {
        ColorMode::Always
    } else if cli.no_color {
        ColorMode::Never
    } else {
        ColorMode::Auto
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_indent() {
        assert!(validate_indent("2").is_ok());
        assert!(validate_indent("8").is_ok());
        assert!(validate_indent("0").is_ok());
        assert!(validate_indent("9").is_err());
        assert!(validate_indent("abc").is_err());
    }

    #[test]
    fn test_determine_doc_type() {
        let cli = Cli::parse_from(["rxq"]);

        let html = "<!DOCTYPE html><html></html>";
        assert_eq!(determine_doc_type(&cli, html), DocumentType::Html);

        let json = r#"{"key": "value"}"#;
        assert_eq!(determine_doc_type(&cli, json), DocumentType::Json);

        let xml = "<root></root>";
        assert_eq!(determine_doc_type(&cli, xml), DocumentType::Xml);
    }

    #[test]
    fn test_color_mode_in_place() {
        let cli = Cli::parse_from(["rxq", "-i", "test.xml"]);
        assert_eq!(color_mode(&cli), ColorMode::Never);
    }

    #[test]
    fn test_color_mode_forced() {
        let cli = Cli::parse_from(["rxq", "-c"]);
        assert_eq!(color_mode(&cli), ColorMode::Always);
    }
}
