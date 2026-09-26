//! BM25 recall over skill names and descriptions.
//!
//! Recall only: this picks the shortlist Jev chooses from, so it only has to
//! keep the right skill in the top N, not put it first.

use crate::skills::Skill;
use bm25::{Document, Language, SearchEngineBuilder};

/// The `limit` best-scoring skills for `query`, best first. Skills that share
/// no term with the query are never returned.
pub fn rank<'a>(query: &str, skills: &'a [Skill], limit: usize) -> Vec<&'a Skill> {
    if skills.is_empty() {
        return Vec::new();
    }
    let documents = skills
        .iter()
        .enumerate()
        .map(|(i, s)| Document::new(i, format!("{} {}", s.name.replace('-', " "), s.description)));
    SearchEngineBuilder::<usize>::with_documents(Language::English, documents)
        .build()
        .search(query, limit)
        .into_iter()
        .map(|result| &skills[result.document.id])
        .collect()
}

#[cfg(test)]
#[path = "bm25_tests.rs"]
mod tests;
