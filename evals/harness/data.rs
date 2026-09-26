//! The eval's inputs: labelled cases, recorded decisions, and the baseline.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Case {
    pub prompt: String,
    /// Skills that count as correct; empty means the right answer is none.
    pub expect: Vec<String>,
}

pub use super::decision::{Cassette, Decision};

#[derive(Debug, Serialize, Deserialize)]
pub struct Baseline {
    pub correct: usize,
    pub abstained: usize,
    pub recalled: usize,
}

pub fn path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("evals")
        .join(rel)
}

pub fn cases() -> Vec<Case> {
    serde_json::from_slice(&std::fs::read(path("cases.json")).unwrap()).unwrap()
}

pub fn read<T: for<'de> Deserialize<'de>>(rel: &str) -> T {
    let bytes = std::fs::read(path(rel)).unwrap_or_else(|e| panic!("evals/{rel}: {e}"));
    serde_json::from_slice(&bytes).unwrap()
}

pub fn write<T: Serialize>(rel: &str, value: &T) {
    let json = serde_json::to_string_pretty(value).unwrap() + "\n";
    std::fs::write(path(rel), json).unwrap();
}
