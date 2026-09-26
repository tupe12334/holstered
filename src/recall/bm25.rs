//! Keyword recall: BM25 over skill names and descriptions.

use super::document;
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
        .map(|(i, s)| Document::new(i, document(s)));
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
