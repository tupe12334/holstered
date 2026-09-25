//! BM25 recall over skill names and descriptions.
//!
//! Recall only: this picks the shortlist Jev chooses from, so it is tuned to
//! keep the right skill in the top N, not to put it first.

use crate::skills::Skill;
use std::collections::HashMap;

const K1: f64 = 1.2;
const B: f64 = 0.75;

const STOPWORDS: &[&str] = &[
    "the", "and", "for", "with", "this", "that", "you", "your", "are", "can", "was", "were", "has",
    "have", "had", "did", "does", "not", "but", "from", "into", "out", "all", "any", "how", "why",
    "what", "when", "who", "use", "using", "used", "make", "made", "get", "got", "now", "then",
    "also", "should", "would", "could", "will", "its", "please", "need", "want", "let", "lets",
    "like", "just", "one", "two", "new", "old", "see", "look", "check", "here", "there", "some",
    "more", "most", "than", "only",
];

/// Crude suffix stripper so `merged`, `merging` and `merge` agree. Prompt and
/// documents go through the same function, so only agreement matters.
fn stem(word: &str) -> String {
    let mut w = word.to_owned();
    for suffix in ["ing", "ed", "es", "s"] {
        if w.len() >= suffix.len() + 3 && w.ends_with(suffix) {
            w.truncate(w.len() - suffix.len());
            break;
        }
    }
    if w.ends_with('i') {
        w.pop();
        w.push('y');
    }
    if w.len() > 3 && w.ends_with('e') {
        w.pop();
    }
    w
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| w.len() >= 3 && !STOPWORDS.contains(w))
        .map(stem)
        .collect()
}

/// The `limit` best-scoring skills for `query`, best first. Skills that share
/// no term with the query are never returned.
pub fn rank<'a>(query: &str, skills: &'a [Skill], limit: usize) -> Vec<&'a Skill> {
    let docs: Vec<Vec<String>> = skills
        .iter()
        .map(|s| tokenize(&format!("{} {}", s.name.replace('-', " "), s.description)))
        .collect();
    if docs.is_empty() {
        return Vec::new();
    }
    let n = docs.len() as f64;
    let avgdl = docs.iter().map(Vec::len).sum::<usize>() as f64 / n;
    let mut df: HashMap<&str, f64> = HashMap::new();
    for doc in &docs {
        let mut seen: Vec<&str> = doc.iter().map(String::as_str).collect();
        seen.sort_unstable();
        seen.dedup();
        for term in seen {
            *df.entry(term).or_default() += 1.0;
        }
    }
    let mut terms = tokenize(query);
    terms.sort();
    terms.dedup();

    let mut scored: Vec<(f64, usize)> = docs
        .iter()
        .enumerate()
        .filter_map(|(i, doc)| {
            let len = doc.len() as f64;
            let score: f64 = terms
                .iter()
                .filter_map(|t| {
                    let f = doc.iter().filter(|w| *w == t).count() as f64;
                    let d = *df.get(t.as_str())?;
                    let idf = (1.0 + (n - d + 0.5) / (d + 0.5)).ln();
                    Some(idf * f * (K1 + 1.0) / (f + K1 * (1.0 - B + B * len / avgdl)))
                })
                .sum();
            (score > 0.0).then_some((score, i))
        })
        .collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, i)| &skills[i])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(name: &str, description: &str) -> Skill {
        Skill {
            name: name.into(),
            description: description.into(),
            path: Default::default(),
        }
    }

    #[test]
    fn stems_agree_across_inflections() {
        assert_eq!(stem("merged"), stem("merging"));
        assert_eq!(stem("merges"), stem("merge"));
        assert_eq!(stem("queries"), "query");
    }

    #[test]
    fn ranks_matching_skill_first_and_drops_non_matches() {
        let skills = [
            skill(
                "productivity-powerpoint",
                "Create and edit slide decks and presentations.",
            ),
            skill("github-pr-merge", "Merge pull requests safely."),
            skill("cooking", "Bake bread."),
        ];
        let got: Vec<_> = rank("merge the pull request", &skills, 20)
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert_eq!(got, ["github-pr-merge"]);
        let got = rank("make a slide deck presentation", &skills, 20);
        assert_eq!(got[0].name, "productivity-powerpoint");
    }

    #[test]
    fn respects_limit_and_handles_empty() {
        let skills = [
            skill("a-merge", "merge"),
            skill("b-merge", "merge"),
            skill("c-merge", "merge"),
        ];
        assert_eq!(rank("merge", &skills, 2).len(), 2);
        assert!(rank("merge", &[], 5).is_empty());
        assert!(rank("the and", &skills, 5).is_empty());
    }
}
