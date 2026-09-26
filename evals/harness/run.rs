//! Run the real binary on one prompt against the mocked skill library.

use super::data::path;
use assert_cmd::Command;
use serde_json::{json, Value};

/// The skill holstered injected for `prompt`, or `None` when it stayed silent.
/// `model` is `jev` or `kev` and decides which endpoint variable points at `url`.
pub fn pick(prompt: &str, model: &str, url: &str) -> Option<String> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_holstered"));
    for var in [
        "CLAUDE_CODE_VERSION",
        "POLYHOOK_CALLER",
        "HOLSTERED_KEV_URL",
    ] {
        cmd.env_remove(var);
    }
    if model == "kev" {
        cmd.env("HOLSTERED_KEV_URL", url);
    } else {
        cmd.env("HOLSTERED_JEV_URL", url)
            .env("OPENROUTER_API_KEY", "eval-placeholder");
    }
    let payload =
        json!({"hook_event_name": "UserPromptSubmit", "prompt": prompt, "session_id": "eval"});
    let out = cmd
        .env("HOLSTERED_SKILLS_DIRS", path("skills"))
        .env("HOLSTERED_TIMEOUT_MS", "120000")
        .write_stdin(payload.to_string())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let out: Value = serde_json::from_slice(&out).ok()?;
    let context = out["hookSpecificOutput"]["additionalContext"].as_str()?;
    let name = context.strip_prefix("SKILL SELECTED by holstered: ")?;
    Some(name.split_whitespace().next()?.to_owned())
}
