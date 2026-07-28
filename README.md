# karesansui

> CLI turtle that generates creative ASCII art in your terminal.

`karesansui` (枯山水) is a tiny Rust CLI that progressively builds ASCII art
on a terminal canvas, one action at a time — like watching a calm video game play itself.
An LLM (via any OpenAI-compatible API — NVIDIA NIM, OpenRouter, etc.) creatively directs each move,
with **complete freedom** to place any emoji or ASCII glyph anywhere using `place_glyph`,
`draw_line`, `draw_ring`, `fill_box`, or `clear_cell`.

A turtle (`🐢`) physically walks across the canvas to carry out each instruction.
The art renders right in your terminal, building up gradually at a relaxed pace.
Every **30 minutes**, the session finishes its cycle and begins anew.

**Every run starts in Creative Freedom mode** — the LLM has unlimited freedom over the full canvas.
You can also use `-t` to explore **20 themed styles** — from zen garden classics like
*Three Mountain Sanzen* and *Moonlit Reef* to *Sacred Geometry Mandala*,
*Enso Fractal Solitude*, **Tabula Rasa (Pure ASCII Muse)**, **Wild Zones (Unbound Serenity)**,
and **Gridwright (The Deliberate Grid as Craft)**.

## How it works

- A `Garden` grid holds 2-column wide strings so emoji and ASCII align cleanly: empty sand (`  `), raked lines (`~~`), circular concentric ripples (Midpoint Circle algorithm — `◎ `), rocks (`🪨`, `🗿`, `🗻`), moss (`🌿`), gravel (`··`), flowers (`🌸`), lanterns (`🏮`), dynamic borders (`🎋`, `═╗`, `🌊`, `🌸`, `❖ `), mandala symbols (`⭕`, `◈ `, `✦ `, `☯ `, `❖ `), and pure ASCII glyphs (`# `, `/**`, `/\\`, `||`).
- On startup, **Creative Freedom** is used by default (full canvas, no borders). Use `-t random` or `-t classic` for randomized classic themes, or pick a specific theme by name or index.
- Every turn, the LLM inspects the exact visual state of the canvas and returns a structured JSON action.
- The turtle (`🐢`) animates step-by-step to the destination coordinates and applies the change with smooth `crossterm` rendering.
- To respect free-tier API rate limits (e.g. NVIDIA NIM: ~1 req/3 min), moves pace out every **180 seconds** by default (configurable via `--pace` or `KARESANSUI_TICK_MS`).
- If network rate-limits (429) occur, the engine reads the `Retry-After` header and backs off accordingly with capped exponential backoff.
- Press `Ctrl+C` at any time for a graceful shutdown — the terminal is restored (cursor shown, screen cleared) before exit.
- After **30 minutes of continuous contemplation**, the canvas resets into a fresh session (or saves state to file if `--resume` is enabled).

## What will it make?

That is for the gardener to decide. Run it and see. 🍃

## Commands & Usage

`karesansui` can be run with zero arguments for the default Creative Freedom experience, or customized using CLI flags and an interactive startup menu:

### Interactive Menu Mode (`-i` / `--interactive`)
Launch a clean terminal menu on startup to pick your exact theme and pacing settings before the turtle wakes up:
```bash
cargo run -- -i
# Or directly with the compiled binary:
./target/debug/karesansui --interactive
```

### Command-Line Flags
You can pass your preferred theme, dimensions, state persistence, and debugging flags directly via CLI arguments:
```bash
# Choose a specific theme by name substring or index (1-21), or "random" / "classic":
cargo run -- -t "Tabula Rasa"
cargo run -- --theme 21
cargo run -- -t random

# Resume a previously saved garden session across restarts:
cargo run -- --resume --state-file my_garden.json

# Run an offline simulation (dry run) with single-step verification and export snapshot:
cargo run -- --dry-run --step --snapshot garden_dump.txt

# Customize canvas size, pacing speed, and disable color formatting:
cargo run -- --width 54 --height 22 --pace 300 --no-color

# View all available CLI flags and options:
cargo run -- --help
```

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--theme <THEME>` | `-t` | `random` | Theme name substring or index (`1-20`), or `random` |
| `--width <WIDTH>` | `-w` | `48` | Grid width in terminal columns |
| `--height <HEIGHT>` | | `20` | Grid height in terminal rows |
| `--pace <SECONDS>` | `-p` | `6` | Seconds between normal LLM prompts |
| `--rest <SECONDS>` | | `30` | Seconds to rest after every 10 prompts (rate-limit pause) |
| `--interactive` | `-i` | `false` | Launch interactive theme/setting selection menu |
| `--resume` | `-r` | `false` | Resume garden from saved state file across sessions |
| `--state-file <PATH>`| | `karesansui_state.json` | Path to JSON state file for saving or resuming state |
| `--dry-run` | `-d` | `false` | Offline simulation without making LLM API calls |
| `--step` | `-s` | `false` | Single-step debug mode: run one action and exit cleanly |
| `--snapshot <PATH>` | | `None` | Dump garden state/text to file on completion or step |
| `--no-color` | | `false` | Disable faint crossterm coloring and use plain text output |

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
- **Wild Zones (Unbound Serenity)** *(✨ NEW)* — true liberation: all zen garden rules, raked sand, mandalas, and rigid borders are completely removed without any forced visual persona. Guided strictly by calm, peace, and serenity (zero profanity/threats/abuse), the LLM has absolute freedom across the open canvas (`place_glyph`, `draw_line`, `draw_ring`, `fill_box`, `clear_cell`) using any emoji or ASCII characters without influence from other themes
- **Gridwright (The Deliberate Grid as Craft)** *(✨ NEW)* — pixel-perfect grid art where every cell is a deliberate choice. Uses precise geometric shapes (lines, circles, rectangles), runtime-selected palettes, and mathematical composition with no scaling, no smoothing, no interpolation—just pure, intentional placement on a clean grid. The LLM orchestrates pixel-by-pixel construction using explicit drawing actions (`SetPixel`, `DrawLine`, `DrawCircle`, `FillRectangle`) and color palette indices for controlled artistic expression.

## Providers & Models

Two providers are supported out of the box:

| Provider | Endpoint | Key prefix | Best for |
|----------|----------|------------|----------|
| **NVIDIA NIM** ⭐ | `https://integrate.api.nvidia.com/v1/chat/completions` | `nvapi-...` | Higher rate limits, faster inference (recommended) |
| **OpenRouter** | `https://openrouter.ai/api/v1/chat/completions` | `sk-or-...` | Free tier models, broad selection |

Any OpenAI-compatible API can be used by setting `LLM_API_URL` to the desired endpoint.

**OpenRouter free model allowlist:** When using the default OpenRouter endpoint, the gardener restricts itself to a hardcoded `FREE_MODELS` list in `src/llm.rs` (`tencent/hy3:free`, `google/gemma-4-31b-it:free`, and others). Non-allowlisted models fall back to the default. This check is **bypassed** when using a custom `LLM_API_URL` (e.g. NVIDIA), giving you full model freedom.

## Setup

1. **Get an API key:**
   - [OpenRouter](https://openrouter.ai/keys) (free tier, `sk-or-...`)
   - [NVIDIA NIM](https://build.nvidia.com/) (free tier, `nvapi-...`)
2. Create your local env file:
   ```bash
   cp .env.example .env
   ```
3. Configure `.env` with your provider:
   ```bash
   # NVIDIA NIM (recommended — higher rate limits, faster inference):
   LLM_API_KEY=nvapi-...
   LLM_API_URL=https://integrate.api.nvidia.com/v1/chat/completions
   LLM_MODEL=nvidia/nemotron-3-nano-30b-a3b

   # OpenRouter (fallback — free tier, more limited rate limits):
   LLM_API_KEY=sk-or-...
   # LLM_API_URL defaults to OpenRouter
   LLM_MODEL=tencent/hy3:free
   ```
   (`.env` is gitignored.)
4. Build:
   ```bash
   cargo build
   ```
5. Run:
   ```bash
   cargo run
   ```

## Configuration

| Env var | Default | Notes |
|---------|---------|-------|
| Env var | Default | Notes |
|---------|---------|-------|
| `LLM_API_KEY` | _(required)_ | API key (`nvapi-...`, `sk-or-...`, etc.). Also accepts legacy `OPENROUTER_API_KEY`. |
| `LLM_API_URL` | `https://openrouter.ai/api/v1/chat/completions` | Any OpenAI-compatible chat completions endpoint. Also accepts legacy `OPENROUTER_URL`. |
| `LLM_MODEL` | `tencent/hy3:free` | Model identifier. Also accepts legacy `OPENROUTER_MODEL`. |
| `KARESANSUI_TICK_MS` | `6000` (`--pace` * 1000) | Milliseconds between normal moves (overrides `--pace`). |
| `KARESANSUI_REST_SECS` | `30` (`--rest`) | Seconds to rest after every 10 moves (overrides `--rest`). |

In addition to `.env`, grid size (`--width`/`--height`), pacing (`--pace`), rate-limit pauses (`--rest`), state persistence (`--resume`), and debugging modes (`--step`/`--snapshot`) can be passed dynamically via command-line options or configured interactively (`-i`).

## Layout

- `src/openrouter.rs` — `LlmClient` shared HTTP client for OpenAI-compatible APIs. Configurable endpoint (`LLM_API_URL`), Bearer auth, exponential backoff with retries (4 attempts), and OpenRouter-specific header injection (`HTTP-Referer`, `X-Title`) only when the endpoint contains `openrouter.ai`.
- `src/garden.rs` — `Garden` grid, `GardenState` persistence (`serde`), `crossterm` colored rendering, `BorderPattern` (12 dynamic border styles), `Action` enum (`RakeRing`, `PlaceMandala`, etc.), `execute_action()` dispatch for all action types with turtle animation, and 2-column glyph definitions.
- `src/llm.rs` — LLM prompt engineer: 20-theme pool, `FREE_MODELS` allowlist (bypassed with custom `LLM_API_URL`), offline simulation (`--dry-run`), and dynamic prompt construction.
- `src/main.rs` — CLI parser (`clap`), interactive terminal menu (`-i`), `crossterm` screen management (`CleanExit` guard), `Ctrl+C` graceful shutdown handler (`tokio::signal`), single-step debugging (`--step`), state persistence loop, and 30-minute session lifecycle.
- `src/vec.rs` — `Point` geometry, line drawing (Bresenham), circle generation (Midpoint Circle algorithm), distance calculations, and filled rectangle primitives.
- `src/color.rs` — RGB color management, 5 pre-defined palettes (monochrome, zen_earth, night_sky, vibrant_neon, warm_earth), palette quantization.
- `src/canvas.rs` — 2D pixel grid with drawing primitives (lines, circles, rectangles) and ANSI 24-bit RGB color output.
- `src/pixel_art.rs` — `PixelArtAction` enum for LLM-driven pixel art, `GridwrightConfig`, and action executor.
- `src/gridwright_runner.rs` — End-to-end LLM orchestration for Gridwright pixel art sessions.

## Credits

See [CREDITS.md](CREDITS.md) for the full history of contributions to this project.
