//! Skill-selection eval: labelled prompts over `evals/skills`, scored against
//! recorded model decisions so every run is deterministic and offline.

mod evals;

use evals::data::{self, Baseline, Cassette};
use evals::{score, server::Mode, stale};
use std::collections::BTreeMap;

#[test]
fn recorded_decisions_meet_the_baseline() {
    let baselines: BTreeMap<String, Baseline> = data::read("baseline.json");
    for (model, want) in baselines {
        let tape: Cassette = data::read(&format!("cassettes/{model}.json"));
        let out = score::evaluate(&model, Mode::Replay(tape.clone()));
        eprintln!("{model}: {:?}\n  {}", out.score, out.misses.join("\n  "));
        let stale = stale(&tape, &out.seen);
        assert!(
            stale.is_empty(),
            "{model}: re-record, shortlist changed for {stale:?}"
        );
        let s = &out.score;
        assert!(
            s.correct >= want.correct
                && s.abstained >= want.abstained
                && s.recalled >= want.recalled,
            "{model}: {s:?} fell below the baseline {want:?}"
        );
    }
}

/// `HOLSTERED_EVAL_RECORD=jev|kev cargo test --test eval -- --ignored record`
#[test]
#[ignore = "calls a live decision model; see evals/README.md"]
fn record() {
    evals::record::record();
}
