//! Cassettes are keyed by the prompt holstered sends, which is clipped;
//! look cases up the same way so long prompts find their decision.

#[path = "../../src/prompt.rs"]
mod prompt;

pub use prompt::clip;
