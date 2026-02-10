//! Formatting and beautification with syntax highlighting

use crate::error::FormatError;
use crate::query::QueryOptions;
use crate::types::{Document, NodeRef, NodeType};
use std::io::Write;

/// Color mode for output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,   // Color if terminal detected
    Always, // Force color
    Never,  // No color
}

/// Indentation style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Indent {
    Spaces(u8), // 0-8 spaces
    Tab,
}

impl Indent {
    /// Get the indentation string for a given level
    pub fn as_str(&self, level: usize) -> String {
        match self {
            Indent::Spaces(n) => " ".repeat(*n as usize * level),
            Indent::Tab => "\t".repeat(level),
        }
    }

    /// Get single indentation unit
    pub fn unit(&self) -> &str {
        match self {
            Indent::Spaces(n) => match n {
                0 => "",
                1 => " ",
                2 => "  ",
                3 => "   ",
                4 => "    ",
                5 => "     ",
                6 => "      ",
                7 => "       ",
                8 => "        ",
                _ => "  ", // fallback
            },
            Indent::Tab => "\t",
        }
    }
}

/// Complete formatting configuration
#[derive(Clone)]
pub struct FormatOptions {
    pub indent: Indent,
    pub color: ColorMode,
    pub compact: bool, // For JSON: no indentation
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent: Indent::Spaces(2),
            color: ColorMode::Auto,
            compact: false,
        }
    }
}

impl FormatOptions {
    /// Check if colors should be enabled
    pub fn use_colors(&self) -> bool {
        match self.color {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => atty::is(atty::Stream::Stdout),
        }
    }
}

/// ANSI color codes
#[derive(Debug, Clone, Copy)]
pub struct ColorScheme {
    pub tag: &'static str,     // Yellow for tags
    pub attr: &'static str,    // Green for attributes
    pub comment: &'static str, // Blue for comments
    pub value: &'static str,   // Green for values
    pub reset: &'static str,
}

impl ColorScheme {
    pub const fn default() -> Self {
        Self {
            tag: "\x1b[33m",     // Yellow
            attr: "\x1b[32m",    // Green
            comment: "\x1b[94m", // Bright Blue
            value: "\x1b[32m",   // Green
            reset: "\x1b[0m",
        }
    }

    pub const fn none() -> Self {
        Self {
            tag: "",
            attr: "",
            comment: "",
            value: "",
            reset: "",
        }
    }
}

/// Format a document to a writer
pub trait Formatter {
    fn format<'input, W: Write>(
        &self,
        doc: &Document<'input>,
        writer: &mut W,
        options: &FormatOptions,
    ) -> Result<(), FormatError>;
}

/// XML/HTML formatter
pub struct XmlFormatter;

impl Formatter for XmlFormatter {
    fn format<'input, W: Write>(
        &self,
        doc: &Document<'input>,
        writer: &mut W,
        options: &FormatOptions,
    ) -> Result<(), FormatError> {
        let colors = if options.use_colors() {
            ColorScheme::default()
        } else {
            ColorScheme::none()
        };

        let vdom = doc.vdom();
        for handle in vdom.children() {
            let node = NodeRef::new(vdom, Some(*handle));
            // eprintln!("Visiting node: {:?} Type: {:?}", node.tag_name(), node.node_type());
            self.format_node(node, writer, options, &colors, 0)?;
        }
        Ok(())
    }
}

impl XmlFormatter {
    fn format_node<'a, 'input, W: Write>(
        &self,
        node: NodeRef<'a, 'input>,
        writer: &mut W,
        options: &FormatOptions,
        colors: &ColorScheme,
        level: usize,
    ) -> Result<(), FormatError> {
        match node.node_type() {
            NodeType::Element => self.format_element(node, writer, options, colors, level),
            NodeType::Text => self.format_text(node, writer),
            NodeType::Comment => self.format_comment(node, writer, options, colors, level),
            _ => Ok(()),
        }
    }

    fn format_element<'a, 'input, W: Write>(
        &self,
        node: NodeRef<'a, 'input>,
        writer: &mut W,
        options: &FormatOptions,
        colors: &ColorScheme,
        level: usize,
    ) -> Result<(), FormatError> {
        let tag_name = node.tag_name();
        let tag_name_str = tag_name.as_deref().unwrap_or("");
        let indent_str = options.indent.as_str(level);

        // Handle XML Processing Instructions (e.g. <?xml ... ?>) which tl parses as empty tag
        // Handle XML Processing Instructions (e.g. <?xml ... ?>) which tl parses as empty tag
        if tag_name_str.is_empty() {
            // Reconstruct XML declaration if possible, or print raw start
            // Assuming standard <?xml ... ?> if empty tag
            write!(writer, "<?xml")?;
            for (name, value) in node.attributes() {
                // tl parser hack: correct mangled "ersion" to "version"
                let name = if name == "ersion" { "version" } else { &name };
                write!(writer, " {}=\"{}\"", name, value)?;
            }
            writeln!(writer, "?>")?;

            // Recurse children
            for child in node.children() {
                self.format_node(child, writer, options, colors, level)?;
            }
            return Ok(());
        }

        // Opening tag
        write!(writer, "{}{}<{}", indent_str, colors.tag, tag_name_str)?;

        // Attributes
        for (name, value) in node.attributes() {
            write!(
                writer,
                " {}{}{}=\"{}\"{}",
                name, colors.attr, colors.reset, value, colors.reset
            )?;
        }

        if node.has_children() {
            // Check if children are only text
            let is_text_only = node
                .children()
                .all(|c| c.node_type() == NodeType::Text || c.node_type() == NodeType::Raw);

            if is_text_only {
                write!(writer, "{}>{}", colors.tag, colors.reset)?;
                // Format children inline
                for child in node.children() {
                    if let Some(text) = child.text() {
                        write!(writer, "{}", text.trim())?;
                    }
                }
                writeln!(writer, "{}</{}>{}", colors.tag, tag_name_str, colors.reset)?;
            } else {
                write!(writer, "{}>{}", colors.tag, colors.reset)?;
                writeln!(writer)?;

                // Format children block
                for child in node.children() {
                    self.format_node(child, writer, options, colors, level + 1)?;
                }

                // Closing tag
                writeln!(
                    writer,
                    "{}{}</{}>{}",
                    indent_str, colors.tag, tag_name_str, colors.reset
                )?;
            }
        } else {
            // Self-closing tag
            writeln!(writer, "{}/>{}", colors.tag, colors.reset)?;
        }

        Ok(())
    }

    fn format_text<'a, 'input, W: Write>(
        &self,
        node: NodeRef<'a, 'input>,
        writer: &mut W,
    ) -> Result<(), FormatError> {
        if let Some(text) = node.text() {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                writeln!(writer, "{}", trimmed)?;
            }
        }
        Ok(())
    }

    fn format_comment<'a, 'input, W: Write>(
        &self,
        node: NodeRef<'a, 'input>,
        writer: &mut W,
        options: &FormatOptions,
        colors: &ColorScheme,
        level: usize,
    ) -> Result<(), FormatError> {
        let indent_str = options.indent.as_str(level);
        if let Some(comment) = node.comment() {
            writeln!(
                writer,
                "{}{}{}{}",
                indent_str, colors.comment, comment, colors.reset
            )?;
        }
        Ok(())
    }
}

/// Format query results (streaming)
pub fn format_query_results<'doc, 'input: 'doc, W: Write>(
    results: impl Iterator<Item = NodeRef<'doc, 'input>>,
    writer: &mut W,
    options: &QueryOptions,
    _format_opts: &FormatOptions,
) -> Result<(), FormatError> {
    for node in results {
        if options.with_tags {
            // Format with full markup
            let outer = node.outer_html();
            writeln!(writer, "{}", outer)?;
        } else if let Some(attr) = &options.extract_attr {
            // Extract attribute
            if let Some(value) = node.attr(attr) {
                writeln!(writer, "{}", value)?;
            }
        } else {
            // Text content only
            if let Some(text) = node.text() {
                writeln!(writer, "{}", text.trim())?;
            }
        }
    }

    Ok(())
}

/// Simple text formatter (no processing)
pub struct TextFormatter;

impl Formatter for TextFormatter {
    fn format<'input, W: Write>(
        &self,
        doc: &Document<'input>,
        writer: &mut W,
        _options: &FormatOptions,
    ) -> Result<(), FormatError> {
        write!(writer, "{}", doc.source())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DocumentType;

    #[test]
    fn test_indent_as_str() {
        let spaces = Indent::Spaces(2);
        assert_eq!(spaces.as_str(0), "");
        assert_eq!(spaces.as_str(1), "  ");
        assert_eq!(spaces.as_str(2), "    ");

        let tab = Indent::Tab;
        assert_eq!(tab.as_str(0), "");
        assert_eq!(tab.as_str(1), "\t");
        assert_eq!(tab.as_str(2), "\t\t");
    }

    #[test]
    fn test_format_simple_xml() {
        let xml = "<root><child>value</child></root>";
        let doc = Document::parse(xml, DocumentType::Xml).unwrap();

        let formatter = XmlFormatter;
        let mut output = Vec::new();
        let options = FormatOptions {
            indent: Indent::Spaces(2),
            color: ColorMode::Never,
            compact: false,
        };

        formatter.format(&doc, &mut output, &options).unwrap();
        let result = String::from_utf8(output).unwrap();

        assert!(result.contains("<root>"));
        assert!(result.contains("</root>"));
    }

    #[test]
    fn test_color_scheme() {
        let colors = ColorScheme::default();
        assert!(!colors.tag.is_empty());
        assert!(!colors.reset.is_empty());

        let no_colors = ColorScheme::none();
        assert!(no_colors.tag.is_empty());
        assert!(no_colors.reset.is_empty());
    }
}
