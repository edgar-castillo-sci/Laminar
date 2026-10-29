//! Integration tests for config loading.

use assert_cmd::Command;

#[test]
fn config_defaults_from_local_file() {
    // Write a local .laminar.conf, then run `sandwich -q` without -n/-option.
    use std::io::Write;
    let tmp = tempfile::tempdir().unwrap();
    let conf_path = tmp.path().join(".laminar.conf");
    let mut f = std::fs::File::create(&conf_path).unwrap();
    writeln!(f, "[defaults]").unwrap();
    writeln!(f, "option = \"PL\"").unwrap();
    writeln!(f, "layers = 6").unwrap();
    drop(f);

    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.current_dir(tmp.path())
        .args(["sandwich", "-q"])
        .assert()
        .success()
        .stdout("70\n");
}

#[test]
fn no_config_flag_ignores_local_file() {
    use std::io::Write;
    let tmp = tempfile::tempdir().unwrap();
    let conf_path = tmp.path().join(".laminar.conf");
    let mut f = std::fs::File::create(&conf_path).unwrap();
    writeln!(f, "[defaults]").unwrap();
    writeln!(f, "option = \"PL\"").unwrap();
    writeln!(f, "layers = 6").unwrap();
    drop(f);

    let mut cmd = Command::cargo_bin("laminar").unwrap();
    cmd.current_dir(tmp.path())
        .args(["sandwich", "--no-config", "-q"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("--option is required"));
}
