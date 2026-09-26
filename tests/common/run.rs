use super::{skills_dir, KEY};
use assert_cmd::Command;
use serde_json::Value;

/// Runs holstered on `payload`; asserts it exits 0 and never prints the key.
pub fn exec(payload: &Value, env: &[(&str, String)]) -> Value {
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
        "HOLSTERED_JEV_URL",
        "HOLSTERED_KEV_URL",
        "HOLSTERED_THRESHOLD",
    ] {
        cmd.env_remove(var);
    }
    let out = cmd
        .envs(env.iter().map(|(k, v)| (k, v)))
        .env("HOLSTERED_SKILLS_DIRS", skills.path())
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
