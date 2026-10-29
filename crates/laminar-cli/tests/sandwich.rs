//! Integration tests for the `sandwich` command.

use assert_cmd::Command;

#[test]
fn list_options() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args(["sandwich", "--list-options"])
        .assert()
        .success()
        .stdout(predicates::str::contains("PL"))
        .stdout(predicates::str::contains("OneH"))
        .stdout(predicates::str::contains("Janus1T"));
}

#[test]
fn count_pl_n6() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args(["sandwich", "-n", "6", "--option", "PL", "-q"])
        .assert()
        .success()
        .stdout("70\n");
}

#[test]
fn count_oneh_n4() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args(["sandwich", "-n", "4", "--option", "OneH", "-q"])
        .assert()
        .success()
        .stdout("120\n");
}

#[test]
fn count_janus1t_n3() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args(["sandwich", "-n", "3", "--option", "Janus1T", "-q"])
        .assert()
        .success()
        .stdout("144\n");
}

#[test]
fn stdout_pl_n2() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    let output = cmd
        .args(["sandwich", "-n", "2", "--option", "PL", "-e", "stdout"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 2, "expected 2 classes at n=2");
}

#[test]
fn missing_export_fails() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args(["sandwich", "-n", "2", "--option", "PL"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("--export is required"));
}

#[test]
fn missing_option_fails() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args(["sandwich", "-n", "2", "-q"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("--option is required"));
}

#[test]
fn unknown_option_fails() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args(["sandwich", "-n", "2", "--option", "XYZ", "-q"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown option"));
}

#[test]
fn cif_without_reference_cell_fails() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args([
        "sandwich", "-n", "2", "--option", "PL",
        "-e", "cif", "--output", "/tmp/laminar_test",
    ])
    .assert()
    .failure()
    .stderr(predicates::str::contains("--reference-cell is required"));
}

#[test]
fn cif_export_end_to_end() {
    use std::io::Write;
    let mut cell = tempfile::NamedTempFile::new().unwrap();
    writeln!(cell, "a = 3.16").unwrap();
    writeln!(cell, "c = 12.30").unwrap();
    writeln!(cell, "z_up = 1.56").unwrap();
    writeln!(cell, "z_down = -1.56").unwrap();
    writeln!(cell, "[species_map]").unwrap();
    writeln!(cell, "j = \"Mo\"").unwrap();
    writeln!(cell, "k = \"S\"").unwrap();

    let out_dir = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args([
        "sandwich", "-n", "2", "--option", "OneH",
        "-e", "cif",
        "--output", out_dir.path().to_str().unwrap(),
        "--reference-cell", cell.path().to_str().unwrap(),
    ])
    .assert()
    .success();

    // OneH n=2 has 5 classes → 5 CIF files.
    let entries: Vec<_> = std::fs::read_dir(out_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|s| s == "cif").unwrap_or(false))
        .collect();
    assert_eq!(entries.len(), 5, "expected 5 CIF files, got {}", entries.len());
}

#[test]
fn sandwich_json_count() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    let output = cmd
        .args(["sandwich", "-n", "6", "--option", "PL", "-q", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "sandwich");
    assert_eq!(json["option"], "PL");
    assert_eq!(json["n"], 6);
    assert_eq!(json["mode"], "count");
    assert_eq!(json["count"], 70);
    assert!(json["metadata"]["laminar_version"].is_string());
}

#[test]
fn sandwich_json_stdout() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    let output = cmd
        .args(["sandwich", "-n", "2", "--option", "PL", "-e", "stdout", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "stdout");
    assert_eq!(json["count"], 2);
    assert_eq!(json["chains"].as_array().unwrap().len(), 2);
}

#[test]
fn sandwich_verbose_stderr() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    let output = cmd
        .args(["sandwich", "-n", "2", "--option", "PL", "-e", "stdout", "-v"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("symmetry=honeycomb"),
        "expected 'symmetry=honeycomb' in stderr, got: {}",
        stderr
    );
}
