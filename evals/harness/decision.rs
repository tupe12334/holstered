//! One recorded decision per prompt.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What the decision model was offered for one prompt, what it chose, and
/// how sure it was of each option (`none` included).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub offered: Vec<String>,
    pub choice: String,
    /// Empty when the model returned none; replayed back to holstered as-is.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub probabilities: BTreeMap<String, f64>,
}

pub type Cassette = BTreeMap<String, Decision>;

impl Decision {
    pub fn p(&self, option: &str) -> f64 {
        self.probabilities.get(option).copied().unwrap_or(0.0)
    }

    /// The most likely option other than `none`, with its probability.
    pub fn best_skill(&self) -> Option<(&str, f64)> {
        self.probabilities
            .iter()
            .filter(|(k, _)| *k != "none")
            .map(|(k, p)| (k.as_str(), *p))
            .max_by(|a, b| a.1.total_cmp(&b.1))
    }
}
