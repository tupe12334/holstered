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

## Releasing

Bump `version` in `Cargo.toml` through a PR, then tag the merge commit on
`main` with `v<version>` and push the tag. `release.yml` publishes it to
crates.io; its guards and checks live in that workflow.
