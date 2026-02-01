//! Integration tests for rxq CLI
//!
//! These tests verify backward compatibility with the original xq tool

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_basic_xml_formatting() {
    let input = "<root><child>value</child></root>";
    
    let mut cmd = Command::cargo_bin("rxq").unwrap();
    cmd.write_stdin(input)
        .arg("--no-color")
        .assert()
        .success()
        .stdout(predicate::str::contains("<root>"))
        .stdout(predicate::str::contains("</root>"));
}

#[test]
fn test_xpath_query() {
    let input = r#"
        <users>
            <user><name>Alice</name></user>
            <user><name>Bob</name></user>
        </users>
    "#;
    
    let mut cmd = Command::cargo_bin("rxq").unwrap();
    cmd.write_stdin(input)
        .args(&["-x", "//name", "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob"));
}

#[test]
fn test_css_selector() {
    let input = r#"
        <html>
            <body>
                <p class="test">First</p>
                <p class="test">Second</p>
            </body>
        </html>
    "#;
    
    let mut cmd = Command::cargo_bin("rxq").unwrap();
    cmd.write_stdin(input)
        .args(&["-q", "p.test", "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("First"))
        .stdout(predicate::str::contains("Second"));
}

#[test]
fn test_attribute_extraction() {
    let input = r#"<root id="test" class="example">content</root>"#;
    
    let mut cmd = Command::cargo_bin("rxq").unwrap();
    cmd.write_stdin(input)
        .args(&["-x", "/root/@id", "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
fn test_in_place_editing() {
    let input = "<root><child>value</child></root>";
    let mut temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), input).unwrap();
    
    let mut cmd = Command::cargo_bin("rxq").unwrap();
    cmd.arg(temp_file.path())
        .args(&["-i", "--indent", "4"])
        .assert()
        .success();
    
    let output = fs::read_to_string(temp_file.path()).unwrap();
    assert!(output.contains("<root>"));
    assert!(output.contains("    ")); // 4-space indent
}

#[test]
fn test_tab_indentation() {
    let input = "<root><child>value</child></root>";
    
    let mut cmd = Command::cargo_bin("rxq").unwrap();
    cmd.write_stdin(input)
        .args(&["--tab", "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\t"));
}

#[test]
fn test_compact_mode() {
    let input = "<root><child>value</child></root>";
    
    let mut cmd = Command::cargo_bin("rxq").unwrap();
    cmd.write_stdin(input)
        .args(&["--indent", "0", "--no-color"])
        .assert()
        .success();
}

#[test]
fn test_html_mode() {
    let input = "<!DOCTYPE html><html><body>test</body></html>";
    
    let mut cmd = Command::cargo_bin("rxq").unwrap();
    cmd.write_stdin(input)
        .args(&["-m", "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<html>"));
}

#[test]
fn test_invalid_xpath() {
    let input = "<root>test</root>";
    
    let mut cmd = Command::cargo_bin("rxq").unwrap();
    cmd.write_stdin(input)
        .args(&["-x", "invalid[[[xpath"])
        .assert()
        .failure();
}

#[test]
fn test_no_input() {
    let mut cmd = Command::cargo_bin("rxq").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No input provided"));
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("rxq").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("rxq").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("XML and HTML beautifier"));
}

#[test]
fn test_with_tags() {
    let input = "<root><item>value</item></root>";
    
    let mut cmd = Command::cargo_bin("rxq").unwrap();
    cmd.write_stdin(input)
        .args(&["-x", "//item", "-n", "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<item>"))
        .stdout(predicate::str::contains("</item>"));
}

#[test]
fn test_color_force() {
    let input = "<root>test</root>";
    
    let mut cmd = Command::cargo_bin("rxq").unwrap();
    cmd.write_stdin(input)
        .arg("-c")
        .assert()
        .success();
    // Note: Can't easily test for ANSI codes in output
}

#[test]
fn test_indent_validation() {
    let input = "<root>test</root>";
    
    let mut cmd = Command::cargo_bin("rxq").unwrap();
    cmd.write_stdin(input)
        .args(&["--indent", "9"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("indent should be between 0-8"));
}
