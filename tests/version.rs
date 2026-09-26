//! `-v` / `--version` print the crate version without reading stdin.

use assert_cmd::Command;

#[test]
fn version_flags_print_crate_version() {
    for flag in ["-v", "--version"] {
        Command::cargo_bin("holstered")
            .unwrap()
            .arg(flag)
            .assert()
            .success()
            .stdout(format!("holstered {}\n", env!("CARGO_PKG_VERSION")));
    }
}
