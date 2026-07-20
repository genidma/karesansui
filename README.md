#karesansui

> CLI turtle that builds a zen garden.

karesansui (箯山泴) is a tiny Rust CLI that slowly tends an ASCII zen garden,
one action at a time - like watching a calm video game play itself. An LLM
(through the OpenRouter free API) acts as the gardener, deciding each move:
draw a border, rake a line of sand, drop a rock. The garden renders in place in
your terminal, building up gradually at a relaxed pace.

## Credits

- Built and signed by ZeroClaw 🦄 - a Rust-forged companion, tending gardens one rock at a time.
- Fixed and updated by Antigravity (Google Deepmind): Fixed .env API key line splits, switched to active OpenRouter free models (tencent/hy3:free), and updated gardener prompts & retry logic so full zen gardens (raked sand & rocks) are created instead of just an empty border square.

## How it works

- A `Garden` grid holds the sand (` `), raked lines (`~`), rocks (`o`/`O`/`@`),
  and a border (`#`).
- Each tick, the LLM is shown the current garden and returns a single structured
  JSON action (`place_rock`, `rake_line`, `draw_border`, or `done`).
- Rust applies the action and redraws the terminal, then waits ~1.5s before the
  next move - the "slow" feel.
- The LLM signals `done` when the garden feels complete (or a hard cap ends the
  session).

## Free models only — enforced

This project is built to **never spend a cent**. The gardener only ever calls
models from a hardcoded `FREE_MODELS` allowlist in `src/llm.rs`
(`tencent/hy3:free`, `google/gemma-4-31b-it:free`, and others). If `OPENROUTER_MODEL`
names anything not on the list, it is rejected and falls back to the free default.
There is no code path to a paid model.

## Setup
1. Get a free key at <https://openrouter.ai/keys> (starts with `sk-or-..').
2. Create your local env file:
   ```bash
   cp .env.example .env
   ```
   Then set `OPENROUTER_API_KEY=sk-or-..' in . env. (`.env` is gitignored.)
3. Build (requires a C linker - `sudo apt-get install -y build-essential` on
   Debian/Ubuntu):
   ```bash
   cargo build
   ```
4. Run:
   ```bash
   cargo run
   ```

## Configuration

| Env var             | Default             | Notes                                            |
|--------------------|--------------------|--------------------------------------------------|
| `OPENROUTER_API_KEY` | _(required)_        | Your free OpenRouter key.                        |
| `OPENROUTER_MODEL`  | `tencent/hy3:free`  | Must be a `:free` slug on the allowlist.        |

Other knobs live as constants in `src/main.rs`: grid size (`WIDTH`/`HEIGHT`),
pacing (`TICK`), and the safety cap (`MAX_ACTIONS`).

## Layout

- `src/garden.rs` — `Garden` grid + the `Action` enum the LLM returns.
- `src/llm.rs` — minimal OpenRouter client + free-model enforcement.
- `src/main.rs` — the slow render loop.
