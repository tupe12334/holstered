# holstered

**Hands your coding agent the right skill for each prompt.**

Agents with hundreds of skills see only their names, so the one that fits the
task often goes unused. holstered is a prompt hook: on every user prompt it
shortlists skills with BM25, asks a decision model to pick one (or none), and
injects that skill's `SKILL.md` into the model's context for the turn.

The decision model is your choice: **Jev**, hosted on OpenRouter, or **Kev**,
an open model you run locally so nothing leaves your machine. See
[Choose a decision model](#choose-a-decision-model).

One Rust binary serves every agent through [polyhook](https://github.com/polyhook/polyhook),
which translates each agent's hook payload and response format.

## How it works

```
user prompt ─▶ agent prompt hook ─▶ holstered
                                      │ polyhook: detect agent, read prompt
                                      │ BM25 over name + description → top 20
                                      │ Jev (hosted) or Kev (local): pick one or "none"
                                      │ polyhook: inject SKILL.md in the agent's format
                                      ▼
                            model sees the skill this turn
```

Nothing is injected, and the prompt goes through untouched, when the decision
model answers `none`, names a skill it was not offered, fails or times out
(8s by default), or none is configured. holstered never blocks a prompt.

### Why BM25 + a decision model

On a 38-prompt labeled set over a 581-skill library (32 prompts with a correct
skill, 6 with none):

| Pipeline | Correct pick | Stays silent on no-skill prompts |
|---|---|---|
| Keyword match | 14/32 | 5/6 |
| BM25 top-1 | 17/32 | 2/6 |
| BM25 top-20 → Cohere rerank-v3.5 | 25/32 | 2/6 |
| BM25 top-20 → Jev (Python prototype) | 28/32 | 6/6 |
| **holstered binary (`bm25` crate → Jev)** | **29/32** | **6/6** |
| holstered binary (`bm25` crate → local Kev-4B) | 26/32 | 6/6 |

The holstered rows are the release binary run end to end against live Jev
(median 915 ms per prompt) and a local Kev-4B on an Apple Silicon Mac (median
1.7s). One of its three misses picked a `pptx` skill for a
slide-deck prompt the labels credited only to another skill. Small set, single labeler: read it as a direction, not a
guarantee.

## Supported agents

Agent support comes from polyhook; see its [Supported Tools](https://github.com/polyhook/polyhook#supported-tools)
for the full list and which agents can take context injection.

| Agent | Hook | Injection |
|---|---|---|
| Claude Code | `UserPromptSubmit` | ✅ |
| Codex | `UserPromptSubmit` | ✅ |
| Gemini CLI | `BeforeAgent` | ✅ |
| Hermes Agent | `pre_llm_call` | ✅ |
| Cline | `UserPromptSubmit` | ✅ |
| Cursor, Windsurf, Amp | — | ❌ their prompt hooks can only allow or block, so there is nothing to register |

## Install

Pick one:

```bash
# crates.io
cargo install holstered

# Homebrew (builds from source)
brew install tupe12334/tap/holstered

# latest main from GitHub
cargo install --git https://github.com/tupe12334/holstered
```

Then [choose a decision model](#choose-a-decision-model) and register
holstered as the prompt hook of each agent you use.

**Claude Code** — `~/.claude/settings.json`
```json
{ "hooks": { "UserPromptSubmit": [ { "hooks": [ { "type": "command", "command": "holstered" } ] } ] } }
```

**Codex** — `~/.codex/hooks.json`
```json
{ "hooks": { "UserPromptSubmit": [ { "hooks": [ { "type": "command", "command": "holstered" } ] } ] } }
```

**Gemini CLI** — `~/.gemini/settings.json`
```json
{ "hooks": { "BeforeAgent": [ { "hooks": [ { "type": "command", "command": "holstered" } ] } ] } }
```

**Hermes Agent** — `~/.hermes/config.yaml` (approve it on first run, or set `hooks_auto_accept`)
```yaml
hooks:
  pre_llm_call:
    - command: holstered
```

**Cline** — hooks are executables named after the event
```bash
mkdir -p ~/Documents/Cline/Hooks
ln -s "$(command -v holstered)" ~/Documents/Cline/Hooks/UserPromptSubmit
```

## Choose a decision model

Set one of these in the environment your agent starts from. If both are set,
Kev wins.

| | Jev (hosted) | Kev (local) |
|---|---|---|
| Set | `OPENROUTER_API_KEY` | `HOLSTERED_KEV_URL` |
| Runs | OpenRouter Decisions API | [Kev](https://github.com/jaredpalmer/kev) server on your machine |
| Privacy | prompt (first 2,000 chars) and shortlisted skill descriptions go to OpenRouter | nothing leaves the machine |
| Latency per prompt | ~1s | ~1.7–3.5s on an Apple Silicon Mac |
| Accuracy on the set above | 29/32 | 26/32 |

Kev setup and first-request timeout: [CONFIGURATION.md](CONFIGURATION.md#local-model-kev).

## Configuration

Environment variables, the data sent to OpenRouter, and running fully offline
with a local Kev model: see [CONFIGURATION.md](CONFIGURATION.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
