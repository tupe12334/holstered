//! `-h` / `--help` print usage with the crate description and flags.

use assert_cmd::Command;

#[test]
fn help_flags_print_usage() {
    for flag in ["-h", "--help"] {
        let out = Command::cargo_bin("holstered")
            .unwrap()
            .arg(flag)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with(env!("CARGO_PKG_DESCRIPTION")), "{text}");
        for expected in ["Usage: holstered", "-v, --version", "-h, --help", "stdin"] {
            assert!(text.contains(expected), "missing {expected:?} in {text}");
        }
    }
}
