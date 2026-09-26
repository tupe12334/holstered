//! Deterministic eval over a mocked skill library; see `evals/README.md`.

pub mod data;
pub mod live;
pub mod record;
pub mod request;
pub mod run;
pub mod score;
pub mod server;
pub mod tally;

use data::Cassette;

/// Prompts whose recorded decision no longer applies: the shortlist changed,
/// or a prompt now reaches (or no longer reaches) the decision model.
pub fn stale(tape: &Cassette, seen: &Cassette) -> Vec<String> {
    let changed = seen
        .iter()
        .filter(|(_, d)| d.choice == "<shortlist changed>")
        .map(|(p, _)| p.clone());
    let dropped = tape.keys().filter(|p| !seen.contains_key(*p)).cloned();
    changed.chain(dropped).collect()
}
