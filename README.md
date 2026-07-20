# karesansui

> CLI turtle that builds a zen garden, mandala, and fractal in your terminal.

`karesansui` (枯山水) is a tiny Rust CLI that slowly tends an ASCII & emoji zen garden,
one action at a time — like watching a calm video game play itself. An LLM
(through the OpenRouter free API) acts as the master gardener, deciding each move:
draw a bamboo border (`🎋`), rake horizontal (`~~`) or circular (`◎ `) sand ripples,
place stones (`🪨`, `🗿`), grow moss (`🌿`), lay gravel (`··`), add cherry blossom (`🌸`)
and lantern (`🏮`) accents, or compose geometric mandalas (`⭕`, `◈ `, `✦ `, `☯ `, `❖ `).

A turtle (`🐢`) physically walks across the canvas to carry out each instruction.
The garden renders right in your terminal, building up gradually at a relaxed pace.
Every **30 minutes**, the garden finishes its cycle and begins anew.

**Every run is unique.** A theme is chosen from a pool of **19 evocative styles** —
from classic *Three Mountain Sanzen* and *Moonlit Reef* to *Sacred Geometry Mandala*,
*Enso Fractal Solitude*, the spontaneous **Tabula Rasa (Pure ASCII Muse)**, and the unbound **Wild Zones (Unbound Serenity)** — each guiding the LLM toward a distinct composition.

## Credits

- Built and signed by **ZeroClaw** 🦀 — a Rust-forged companion, tending gardens one rock at a time.
- Enhanced by **Claude** (Anthropic, Claude Opus): Added dynamic per-session theme system, expanded ASCII glyph palette, stateful prompt engineering so the LLM progresses through border → raking → rocks → accents → completion, and error resilience with retry logic.
- Updated & Expanded by **Antigravity** (Google DeepMind):
  - Fixed `.env` API key configuration and switched to active OpenRouter free models (`tencent/hy3:free`).
  - Added the **animated turtle (`🐢`)** that pathfinds across the grid to place items and rests (`💤`) between turns.
  - Implemented **rate-limiting pacing & session cycles** (6s pace between prompts, 30s rest every 10 prompts, 30-minute auto-reset loop).
  - Added **minimalist ASCII/emoji mandala & fractal actions** (`place_mandala`, `rake_ring`) and 5 new geometric themes.
  - Implemented **Dynamic Patterned Borders**: Each session is framed by one of **12 unique, aesthetically pleasing border patterns** (*Sacred Double Box*, *Seigaiha Ocean Waves*, *Sakura Blossom Garland*, *Starfield Lattice*, *Shimenawa Sacred Rope*, etc.) laid down by the turtle during the opening perimeter walk.
  - Built **interactive startup menu (`-i`)** and full **CLI command option weaving (`clap`)**.
  - Created **Tabula Rasa (Pure ASCII Muse)** mode: A complete rethink where the LLM discards all zen garden instructions and emoji, taking control of the blank canvas (`[*]`) to spontaneously compose evocative pure ASCII landscapes, architectural monuments, and abstract art using `place_ascii` and `draw_ascii_line`.

## How it works

- A `Garden` grid holds 2-column wide strings so emoji and ASCII align cleanly: empty sand (`  `), raked lines (`~~`), circular concentric ripples (`◎ `), rocks (`🪨`, `🗿`), moss (`🌿`), gravel (`··`), flowers (`🌸`), lanterns (`🏮`), dynamic borders (`🎋`, `═╗`, `🌊`, `🌸`, `❖ `), mandala symbols (`⭕`, `◈ `, `✦ `, `☯ `, `❖ `), and pure ASCII glyphs (`# `, `/**`, `/\\`, `||`).
- On startup, a **theme** is selected (or chosen by you via CLI/interactive menu) and injected into the LLM system prompt.
- Every turn, the LLM inspects the exact visual state of the garden and returns a structured JSON action.
- The turtle (`🐢` or `[*]`) animates step-by-step to the destination coordinates (`animate_walk`) and applies the change.
- To maintain serenity and respect free-tier API rate limits, normal moves space out every **6 seconds**, with a **30-second resting pause** after every 10 moves.
- After **30 minutes of continuous contemplation**, the garden resets into a fresh session.

## What will it make?

That is for the gardener to decide. Run it and see. 🍃

## Commands & Usage

`karesansui` can be run with zero arguments for a randomized experience, or customized using CLI flags and an interactive startup menu:

### Interactive Menu Mode (`-i` / `--interactive`)
Launch a clean terminal menu on startup to pick your exact theme and pacing settings before the turtle wakes up:
```bash
cargo run -- -i
# Or directly with the compiled binary:
./target/debug/karesansui --interactive
```

### Command-Line Flags
You can also pass your preferred theme, dimensions, and timing settings directly via CLI arguments:
```bash
# Choose a specific theme by name substring or index (1-19):
cargo run -- -t "Tabula Rasa"
cargo run -- --theme 19

# Customize canvas size and pacing speeds:
cargo run -- --width 54 --height 22 --pace 4 --rest 15

# View all available CLI flags and options:
cargo run -- --help
```

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--theme <THEME>` | `-t` | `random` | Theme name substring or index (`1-19`), or `random` |
| `--width <WIDTH>` | `-w` | `48` | Grid width in terminal columns |
| `--height <HEIGHT>` | | `20` | Grid height in terminal rows |
| `--pace <SECONDS>` | `-p` | `6` | Seconds between normal LLM prompts |
| `--rest <SECONDS>` | | `30` | Seconds to rest after every 10 prompts (rate-limit pause) |
| `--interactive` | `-i` | `false` | Launch interactive theme/setting selection menu |

## Themes

Each run selects or assigns one of **19 themes**:

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
- **Sacred Geometry Mandala** *(✨ NEW)* — radial symmetry with concentric rings (`rake_ring`) and diamond (`◈ `) or star (`✦ `) cores
- **Enso Fractal Solitude** *(✨ NEW)* — minimalist void anchored by a single Enso circle (`⭕`) with radiating circular ripples
- **Concentric Rings of Sanzen** *(✨ NEW)* — nested circular ripples around triadic rock placements
- **Fractal Starfield Void** *(✨ NEW)* — self-similar lattice of stars (`✦ `) and geometric crests (`❖ `)
- **Yin-Yang Balance** *(✨ NEW)* — dual equilibrium dividing circular sand rings from textured gravel (`··`) and moss (`🌿`)
- **Tabula Rasa (Pure ASCII Muse)** *(✨ NEW)* — complete rethink: ignores all zen garden instructions and emoji, giving the LLM pure ASCII sketching freedom (`place_ascii`, `draw_ascii_line`) across the blank canvas (`[*]`) based on what inspires it right now
- **Wild Zones (Unbound Serenity)** *(✨ NEW)* — true liberation (`🕊️`): all zen garden rules, raked sand, mandalas, and rigid borders are completely removed. Guided strictly by peace, calm, and serenity (zero profanity/threats/abuse), the LLM has absolute freedom across the open canvas (`place_glyph`, `draw_line`, `draw_ring`, `fill_box`, `clear_cell`) using any emoji or ASCII characters without influence from other themes

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
3. Build locally or on your remote machine:
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

In addition to `.env`, grid size (`--width`/`--height`), pacing (`--pace`), and rate-limit pauses (`--rest`) can be passed dynamically via command-line options or configured interactively (`-i`).

## Layout

- `src/garden.rs` — `Garden` grid, `BorderPattern` (12 dynamic geometric/aesthetic border styles), `Action` enum (including `RakeRing` & `PlaceMandala`), turtle pathfinding, and 2-column glyph definitions.
- `src/llm.rs` — OpenRouter client, 19-theme pool, free-model enforcement, and dynamic prompt engineering.
- `src/main.rs` — CLI parser (`clap`), interactive terminal menu (`-i`), slow turtle animation loop (`animate_walk`), and 30-minute session lifecycle.
