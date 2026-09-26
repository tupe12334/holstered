//! Run every labelled case through holstered and score the picks.

use super::data::{cases, Baseline, Cassette};
use super::{run, server, tally::tally};
use std::sync::{Arc, Mutex};

pub struct Outcome {
    pub score: Baseline,
    pub misses: Vec<String>,
    pub seen: Cassette,
}

pub fn evaluate(model: &str, mode: server::Mode) -> Outcome {
    let mut srv = mockito::Server::new();
    let seen = Arc::new(Mutex::new(Cassette::new()));
    let _mock = server::mock(&mut srv, mode, seen.clone());
    let url = format!("{}/decisions", srv.url());
    let mut score = Baseline {
        correct: 0,
        abstained: 0,
        recalled: 0,
    };
    let mut misses = Vec::new();
    for case in cases() {
        let pick = run::pick(&case.prompt, model, &url);
        let offered = seen
            .lock()
            .unwrap()
            .get(&case.prompt)
            .map(|d| d.offered.clone());
        tally(
            &case,
            pick.as_deref(),
            offered.as_deref(),
            &mut score,
            &mut misses,
        );
    }
    let seen = seen.lock().unwrap().clone();
    Outcome {
        score,
        misses,
        seen,
    }
}
