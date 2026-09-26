//! Sends the decision request to Jev or Kev and returns the raw answer.

use super::{request, Endpoint};
use crate::skills::Skill;
use serde_json::Value;

pub fn ask(at: &Endpoint, prompt: &str, pool: &[&Skill]) -> Result<Value, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(at.timeout))
        .build()
        .into();
    // Errors carry the status or transport failure only, never request headers,
    // so the key cannot leak through them.
    let mut req = agent.post(&at.url);
    if let Some(key) = &at.key {
        req = req.header("authorization", &format!("Bearer {key}"));
    }
    req.send_json(request::body(at.model, prompt, pool))
        .map_err(|e| format!("jev request failed: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("jev response unreadable: {e}"))
}
