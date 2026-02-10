use assert_cmd::cargo;
use assert_cmd::Command;
use std::path::PathBuf;

fn get_data_path(file: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // workspace root
    path.push("tests");
    path.push("data");
    path.push(file);
    path
}

fn rxq_cmd() -> Command {
    Command::new(cargo::cargo_bin!("rxq"))
}

#[test]
fn test_count_multiple_items() {
    let input = get_data_path("xml/multiple.xml");

    rxq_cmd()
       .arg(input)
       .arg("-x")
       .arg("//item")
       .arg("--count")
       .assert()
       .success()
       .stdout("4\n");
}

#[test]
fn test_count_nested_items() {
    let input = get_data_path("xml/multiple.xml");

    rxq_cmd()
       .arg(input)
       .arg("-x")
       .arg("/root/group/item")
       .arg("--count")
       .assert()
       .success()
       .stdout("1\n");
}

#[test]
fn test_count_no_match() {
    let input = get_data_path("xml/multiple.xml");

    rxq_cmd()
       .arg(input)
       .arg("-x")
       .arg("//nonexistent")
       .arg("--count")
       .assert()
       .success()
       .stdout("0\n");
}
