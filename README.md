# karesansui

> CLI turtle that builds a zen garden.

`karesansui` (枯山水) is a tiny Rust CLI that slowly tends an ASCII zen garden,
one action at a time — like watching a calm video game play itself. An LLM
(through the OpenRouter free API) acts as the gardener, deciding each move:
draw a border, rake a line of sand, scatter rocks, grow moss, lay gravel.
The garden renders in place in your terminal, building up gradually at a
relaxed pace.

**Every run is unique.** A random theme is chosen from a pool of 12 evocative
styles — *Moonlit Reef*, *Dragon Tail Ripples*, *Three Mountain Sanzen*,
*Zen Minimalist*, and more — each guiding the LLM toward a distinct
artistic composition.

## Credits

- Built and signed by **ZeroClaw** 🦀 — a Rust-forged companion, tending gardens one rock at a time.
- Fixed and updated by **Antigravity** (Google DeepMind): Fixed `.env` API key line splits, switched to active OpenRouter free models (`tencent/hy3:free`), and updated gardener prompts & retry logic so full zen gardens (raked sand & rocks) are created instead of just an empty border square.
- Enhanced by **Claude** (Anthropic, Claude Opus): Added dynamic per-session theme system with 12 unique garden styles, expanded ASCII glyph palette (moss `*`, gravel `.`), stateful prompt engineering so the LLM progresses through border → raking → rocks → accents → completion, and error resilience with retry logic.

## How it works

- A `Garden` grid holds empty sand (` `), raked lines (`~`), rocks (`o`/`O`/`@`),
  moss (`*`), gravel (`.`), and a border (`#`).
- On startup, a random **theme** is selected (e.g., *"Island Archipelago"*,
  *"Whirlpool Basin"*) and injected into the LLM system prompt.
- Each tick (~1.5s), the LLM is shown the current garden state and returns a
  single structured JSON action.
- Rust applies the action and redraws the terminal, then waits before the
  next move — the "slow video game" feel.
- The prompt adapts dynamically: once the border is drawn, `draw_border` is
  removed from the available actions so the LLM focuses on filling the garden.
- The LLM signals `done` when the garden feels complete (or a hard cap of 40
  actions ends the session).

## What will it make?

That is for the gardener to decide. Run it and see. 🍃

## Themes

Each run randomly selects one of 12 themes:

- **Moonlit Reef** — coral reef clusters with sweeping sand curves
- **Dragon Tail Ripples** — flowing diagonal rake lines in an S-curve
- **Three Mountain Sanzen** — classic triadic rock composition
- **Autumn Sand Drift** — wind-blown gravel and asymmetric rake patterns
- **Island Archipelago** — isolated rock islands with sand channels
- **Stepping Stone Path** — diagonal path of rocks with perpendicular raking
- **Crane and Turtle** — two contrasting rock groupings connected by gravel
- **Zen Minimalist** — extreme restraint, few rocks, full-row raking
- **Forest Clearing** — moss canopy edges with a central clearing
- **Whirlpool Basin** — spiral-converging rake lines around a center cluster
- **Scattered Stars** — many small rocks like a star field
- **River Delta** — fanning rake lines like a branching river

## Free models only — enforced

This project is built to **never spend a cent**. The gardener only ever calls
models from a hardcoded `FREE_MODELS` allowlist in `src/llm.rs`
(`tencent/hy3:free`, `google/gemma-4-31b-it:free`, and others). If
`OPENROUTER_MODEL` names anything not on the list, it is rejected and falls
back to the free default. There is no code path to a paid model.

## Setup

1. Get a free key at <https://openrouter.ai/keys> (starts with `sk-or-...`).
2. Create your local env file:
   ```bash
   cp .env.example .env
   ```
   Then set `OPENROUTER_API_KEY=sk-or-...` in `.env`. (`.env` is gitignored.)
3. Build (requires a C linker — `sudo apt-get install -y build-essential` on
   Debian/Ubuntu):
   ```bash
   cargo build
   ```
4. Run:
   ```bash
   cargo run
   ```

## Configuration

| Env var | Default | Notes |
|---------|---------|-------|
| `OPENROUTER_API_KEY` | _(required)_ | Your free OpenRouter key. |
| `OPENROUTER_MODEL` | `tencent/hy3:free` | Must be a `:free` slug on the allowlist. |

Other knobs live as constants in `src/main.rs`: grid size (`WIDTH`/`HEIGHT`),
pacing (`TICK`), and the safety cap (`MAX_ACTIONS`).

## Layout

- `src/garden.rs` — `Garden` grid, `Action` enum, and glyph rendering.
- `src/llm.rs` — OpenRouter client, theme pool, free-model enforcement, and dynamic prompt engineering.
- `src/main.rs` — the slow render loop with state tracking.
