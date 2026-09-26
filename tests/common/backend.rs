//! Point holstered at Jev on OpenRouter or at a local Kev server.

use super::run::exec;
use mockito::ServerGuard;
use serde_json::Value;

/// Runs holstered against Jev at `server`, with `key` if given.
pub fn run(payload: &Value, server: &ServerGuard, key: Option<&str>) -> Value {
    let mut env = vec![("HOLSTERED_JEV_URL", format!("{}/decisions", server.url()))];
    env.extend(key.map(|k| ("OPENROUTER_API_KEY", k.to_owned())));
    exec(payload, &env)
}

/// Env pointing holstered at `server` as Jev (with a key), then as Kev.
pub fn jev_and_kev(server: &ServerGuard) -> [Vec<(&'static str, String)>; 2] {
    let url = format!("{}/decisions", server.url());
    [
        vec![
            ("HOLSTERED_JEV_URL", url.clone()),
            ("OPENROUTER_API_KEY", super::KEY.to_owned()),
        ],
        vec![("HOLSTERED_KEV_URL", url)],
    ]
}

/// Runs holstered against a local Kev at `server`, with no key set.
pub fn run_kev(payload: &Value, server: &ServerGuard) -> Value {
    exec(
        payload,
        &[("HOLSTERED_KEV_URL", format!("{}/decisions", server.url()))],
    )
}
