//! A stand-in decisions endpoint: replays a cassette, or records one by
//! forwarding each request to a live model.

use super::data::{Cassette, Decision};
use mockito::{Mock, ServerGuard};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

pub enum Mode {
    Replay(Cassette),
    Record {
        upstream: String,
        key: Option<String>,
    },
}

/// Every request's prompt and decision lands in `seen`. A replayed prompt
/// whose shortlist changed since recording gets choice `<shortlist changed>`.
pub fn mock(server: &mut ServerGuard, mode: Mode, seen: Arc<Mutex<Cassette>>) -> Mock {
    server
        .mock("POST", "/decisions")
        .with_body_from_request(move |req| {
            let body: Value = serde_json::from_slice(req.body().unwrap()).unwrap();
            let (prompt, offered) = super::request::parse(&body);
            let (choice, probabilities) = match &mode {
                Mode::Replay(tape) => match tape.get(&prompt) {
                    Some(d) if d.offered == offered => (d.choice.clone(), d.probabilities.clone()),
                    _ => ("<shortlist changed>".into(), Default::default()),
                },
                Mode::Record { upstream, key } => super::live::decide(upstream, key, &body),
            };
            let skill = json!({"type": "choice", "choice": choice, "probabilities": probabilities});
            let answer = json!({"answers": {"skill": skill}});
            let decision = Decision {
                offered,
                choice,
                probabilities,
            };
            seen.lock().unwrap().insert(prompt, decision);
            answer.to_string().into_bytes()
        })
        .create()
}
