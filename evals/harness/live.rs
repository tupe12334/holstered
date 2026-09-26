//! Record mode only: forward holstered's request to the real model.

use serde_json::Value;
use std::collections::BTreeMap;

/// The live model's choice and per-option probabilities for `body`. A failed
/// call's choice is `<error: …>`, so it is visible in the recorded cassette
/// instead of silently counting as none.
pub fn decide(
    upstream: &str,
    key: &Option<String>,
    body: &Value,
) -> (String, BTreeMap<String, f64>) {
    let mut req = ureq::post(upstream);
    if let Some(key) = key {
        req = req.header("authorization", &format!("Bearer {key}"));
    }
    let answer: Result<Value, String> = req
        .send_json(body)
        .map_err(|e| e.to_string())
        .and_then(|mut r| r.body_mut().read_json().map_err(|e| e.to_string()));
    let skill = match answer {
        Ok(a) => a["answers"]["skill"].clone(),
        Err(e) => return (format!("<error: {e}>"), BTreeMap::new()),
    };
    let choice = skill["choice"].as_str().unwrap_or("<error: no choice>");
    let probabilities = serde_json::from_value(skill["probabilities"].clone()).unwrap_or_default();
    (choice.to_owned(), probabilities)
}
