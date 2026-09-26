//! Record mode only: forward holstered's request to the real model.

use serde_json::Value;

/// The live model's choice for `body`, or `<error: …>` so a failed call is
/// visible in the recorded cassette instead of silently counting as none.
pub fn decide(upstream: &str, key: &Option<String>, body: &Value) -> String {
    let mut req = ureq::post(upstream);
    if let Some(key) = key {
        req = req.header("authorization", &format!("Bearer {key}"));
    }
    let answer: Result<Value, String> = req
        .send_json(body)
        .map_err(|e| e.to_string())
        .and_then(|mut r| r.body_mut().read_json().map_err(|e| e.to_string()));
    match answer {
        Ok(a) => a["answers"]["skill"]["choice"]
            .as_str()
            .unwrap_or("<error: no choice>")
            .to_owned(),
        Err(e) => format!("<error: {e}>"),
    }
}
