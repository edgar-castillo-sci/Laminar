//! Integration tests for the `compare` command.

use assert_cmd::Command;

#[test]
fn compare_equivalent_two_chains() {
    // Two OneH chains that are equivalent (same canonical).
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args([
        "compare",
        "+0*j(+k-k)/*j+0(+k-k)",
        "*j+0(+k-k)/+0*j(+k-k)",
        "--option", "OneH",
    ])
    .assert()
    .success()
    .stdout(predicates::str::contains("equivalent: true"));
}

#[test]
fn compare_non_equivalent_chains() {
    // Two PL chains with different words: [0,1] vs [0,0].
    // No σ ∈ S3 maps [0,1] to [0,0] (σ is injective), so they are not
    // equivalent.
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args([
        "compare",
        "+0+j+j/+j+0+j",
        "+0+j+j/+0+j+j",
        "--option", "PL",
    ])
    .assert()
    .success()
    .stdout(predicates::str::contains("equivalent: false"));
}

#[test]
fn compare_from_files() {
    // Two files, one chain each. Both PL chains are in the same orbit.
    use std::io::Write;
    let mut f1 = tempfile::NamedTempFile::new().unwrap();
    writeln!(f1, "+0+j+j").unwrap();
    let mut f2 = tempfile::NamedTempFile::new().unwrap();
    writeln!(f2, "+j+0+j").unwrap();

    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args([
        "compare",
        "--file", f1.path().to_str().unwrap(),
        "--file", f2.path().to_str().unwrap(),
        "--option", "PL",
    ])
    .assert()
    .success()
    .stdout(predicates::str::contains("equivalent: true"));
}

#[test]
fn compare_infers_option() {
    // No --option: infer PL from the chains.
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args([
        "compare",
        "+0+j+j",
        "+j+0+j",
    ])
    .assert()
    .success()
    .stdout(predicates::str::contains("option=PL"))
    .stdout(predicates::str::contains("equivalent: true"));
}

#[test]
fn compare_mixed_options_fails() {
    // A PL chain cannot be parsed as OneH.
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args([
        "compare",
        "+0+j+j",
        "+0*j(+k-k)",
        "--option", "OneH",
    ])
    .assert()
    .failure()
    .stderr(predicates::str::contains("chain"));
}

#[test]
fn compare_invalid_chain_fails() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args([
        "compare",
        "invalid_chain",
        "+0+j+j",
        "--option", "PL",
    ])
    .assert()
    .failure()
    .stderr(predicates::str::contains("chain"));
}

#[test]
fn compare_empty_fails() {
    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.args(["compare"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("no chains to compare"));
}
