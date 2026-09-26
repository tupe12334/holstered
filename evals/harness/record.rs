//! Record mode: run the cases against a live model, then rewrite its
//! cassette and baseline so the diff shows what changed.

use super::data::{self, Baseline};
use super::{score, server::Mode};
use std::collections::BTreeMap;

pub fn record() {
    let model = std::env::var("HOLSTERED_EVAL_RECORD").expect("set HOLSTERED_EVAL_RECORD=jev|kev");
    let upstream = std::env::var("HOLSTERED_EVAL_UPSTREAM")
        .unwrap_or_else(|_| "https://openrouter.ai/api/alpha/decisions".into());
    let key = std::env::var("OPENROUTER_API_KEY")
        .ok()
        .filter(|_| model == "jev");
    let out = score::evaluate(&model, Mode::Record { upstream, key });
    let errors: Vec<_> = out
        .seen
        .iter()
        .filter(|(_, d)| d.choice.starts_with("<error"))
        .collect();
    assert!(errors.is_empty(), "live calls failed: {errors:?}");
    data::write(&format!("cassettes/{model}.json"), &out.seen);
    eprintln!("{model}: {:?}\n  {}", out.score, out.misses.join("\n  "));
    let mut baselines: BTreeMap<String, Baseline> = data::read("baseline.json");
    baselines.insert(model, out.score);
    data::write("baseline.json", &baselines);
}
