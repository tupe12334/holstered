use super::{skills_dir, KEY};
use assert_cmd::Command;
use mockito::ServerGuard;
use serde_json::Value;

/// Runs holstered on `payload`; asserts it exits 0 and never prints the key.
pub fn run(payload: &Value, server: &ServerGuard, key: Option<&str>) -> Value {
    let skills = skills_dir();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_holstered"));
    for var in [
        "POLYHOOK_CALLER",
        "CLAUDE_CODE_VERSION",
        "CURSOR_SESSION_ID",
        "WINDSURF_SESSION_ID",
        "CLINE_SESSION_ID",
        "AMP_SESSION_ID",
        "GEMINI_PROJECT_DIR",
        "OPENROUTER_API_KEY",
    ] {
        cmd.env_remove(var);
    }
    if let Some(key) = key {
        cmd.env("OPENROUTER_API_KEY", key);
    }
    let out = cmd
        .env("HOLSTERED_SKILLS_DIRS", skills.path())
        .env("HOLSTERED_JEV_URL", format!("{}/decisions", server.url()))
        .write_stdin(payload.to_string())
        .assert()
        .success()
        .get_output()
        .clone();
    let (stdout, stderr) = (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    );
    assert!(
        !stdout.contains(KEY) && !stderr.contains(KEY),
        "the key must never be printed"
    );
    serde_json::from_str(&stdout).unwrap_or(Value::Null)
}
