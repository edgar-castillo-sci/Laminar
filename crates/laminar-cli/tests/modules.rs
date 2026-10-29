//! Integration tests for the `modules` command.

use assert_cmd::Command;

#[test]
fn modules_text() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args(["modules"])
        .assert()
        .success()
        .stdout(predicates::str::contains("honeycomb"))
        .stdout(predicates::str::contains("jam"))
        .stdout(predicates::str::contains("cif"))
        .stdout(predicates::str::contains("stdout"));
}

#[test]
fn modules_json() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    let output = cmd.args(["modules", "--json"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["symmetry_modules"][0]["name"], "honeycomb");
    assert_eq!(json["notation_modules"][0]["name"], "jam");
    assert!(json["export_formats"].as_array().unwrap().len() >= 3);
}
