use assert_cmd::cargo;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

// Helper to get path to test data
fn get_test_data_path(file: &str) -> PathBuf {
    // Tests are running from rxq-cli/ or root, handled by cargo
    // But we need to locate the workspace root tests/data directory
    // CARGO_MANIFEST_DIR points to rxq-cli/ location
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up to workspace root
    path.push("tests");
    path.push("data");
    path.push(file);
    path
}

fn rxq_cmd() -> Command {
    Command::new(cargo::cargo_bin!("rxq"))
}

#[test]
fn test_version() {
    rxq_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("rxq"));
}

#[test]
fn test_format_xml_stdout() {
    let input = get_test_data_path("xml/unformatted.xml");
    let expected = get_test_data_path("xml/formatted.xml");

    // Read expected output
    let expected_str = fs::read_to_string(&expected).unwrap();

    rxq_cmd()
        .arg(&input)
        .arg("--no-color") // Ensure no color codes for comparison
        .arg("--indent")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::diff(expected_str)); // diff checks equality
}

#[test]
fn test_format_xml_stdin() {
    let input = get_test_data_path("xml/unformatted.xml");
    let input_content = fs::read_to_string(&input).unwrap();
    let expected = get_test_data_path("xml/formatted.xml");
    let expected_str = fs::read_to_string(&expected).unwrap();

    rxq_cmd()
        .write_stdin(input_content)
        .arg("--no-color")
        .assert()
        .success()
        .stdout(predicate::str::diff(expected_str));
}

#[test]
fn test_xpath_extract() {
    let input = get_test_data_path("xml/unformatted.xml");

    rxq_cmd()
        .arg(&input)
        .arg("-x")
        .arg("//first_name")
        .assert()
        .success()
        .stdout(predicate::str::contains("John"));
}

#[test]
fn test_html_format() {
    let input = get_test_data_path("html/unformatted.html");
    // Note: formatted.html format might differ, check basic valid output
    rxq_cmd()
        .arg(&input)
        .arg("-m") // HTML mode
        .arg("--no-color")
        .assert()
        .success()
        .stdout(predicate::str::contains("<html>"));
}

#[test]
fn test_json_output() {
    let input = get_test_data_path("xml/unformatted.xml");

    rxq_cmd()
        .arg(&input)
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"first_name\": \"John\""));
}

#[test]
fn test_json_compact() {
    let input = get_test_data_path("xml/unformatted.xml");

    rxq_cmd()
        .arg(&input)
        .arg("--json")
        .arg("--compact")
        .assert()
        .success()
        .stdout(predicate::str::contains("{\"user\":"));
}

#[test]
fn test_depth_limit() {
    let input = get_test_data_path("xml/unformatted.xml");

    rxq_cmd()
        .arg(&input)
        .arg("--json")
        .arg("--depth")
        .arg("1")
        .assert()
        .success();
}
