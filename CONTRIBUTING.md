# Contributing

## Prerequisites

- Rust (stable) with `cargo`, `clippy` and `rustfmt`
- [linecheck](https://github.com/tupe12334/linecheck): `cargo install linecheck`

## Checks

Run these before opening a PR; CI runs the same checks (`ci.yml`, `linecheck.yml`).

```bash
cargo test                                   # unit + end-to-end against a mock Jev, per agent format
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
linecheck .                                  # .rs files max 50 lines (linecheck.yml)
```

The end-to-end tests run against a mock Jev server, so no
`OPENROUTER_API_KEY` is needed.

`cargo test` also runs the skill-selection eval, which replays recorded model
decisions over a mocked skill library. After changing retrieval, the request,
or the model, re-record it as described in [evals/README.md](evals/README.md).

## Releasing

Bump `version` in `Cargo.toml` and `gemini-extension.json` through a PR, then tag the merge commit on
`main` with `v<version>` and push the tag. `release.yml` publishes it to
crates.io, then points the formula in
[tupe12334/homebrew-tap](https://github.com/tupe12334/homebrew-tap) at the new
tag, pushing with the `HOMEBREW_TAP_DEPLOY_KEY` deploy key. Its guards and
checks live in that workflow.
