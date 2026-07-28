# karesansui

> CLI turtle that generates creative ASCII art in your terminal.

`karesansui` (枯山水) is a tiny Rust CLI that generates ASCII art on a terminal canvas.
An LLM (via any OpenAI-compatible API — NVIDIA NIM, OpenRouter, etc.) composes a complete
artwork in a single shot: 20-40 actions that create a full composition. The turtle (`🐢`)
then animates each action step-by-step, bringing the artwork to life right in your terminal.

**Every run starts in Creative Freedom mode** — the LLM has unlimited freedom over the
full canvas using any emoji or ASCII glyph. You can also use `-t` to explore **20 themed
styles** — from zen garden classics like *Three Mountain Sanzen* and *Moonlit Reef* to
*Sacred Geometry Mandala*, **Tabula Rasa (Pure ASCII Muse)**, **Wild Zones (Unbound Serenity)**,
and **Gridwright (The Deliberate Grid as Craft)**.

## How it works

- On startup, the LLM is asked to compose a complete piece of ASCII art in one response.
- The LLM (taking 1-3 minutes) returns a JSON array of 20-40 actions.
- The turtle (`🐢`) animates each action sequentially with smooth `crossterm` rendering.
- The final piece is displayed — admire for 15 seconds, then the app exits cleanly.
- Use `-t` to pick a theme, `-d` for dry-run simulation, `--step` to approve actions one-by-one.
- If network rate-limits (429) occur, the engine reads the `Retry-After` header and backs off.
- Press `Ctrl+C` at any time for graceful shutdown.

## What will it make?

That is for the gardener to decide. Run it and see. 🍃

## Commands & Usage

Run with zero arguments for the default Creative Freedom experience, or customize using
CLI flags and the interactive startup menu:

### Interactive Menu Mode (`-i` / `--interactive`)

```bash
cargo run -- -i
```

### Command-Line Flags

```bash
# Choose a specific theme:
cargo run -- -t "Tabula Rasa"
cargo run -- -t random
cargo run -- -t classic

# Run an offline simulation (dry run) with single-step verification:
cargo run -- --dry-run --step --snapshot garden_dump.txt

# Customize canvas size and animation speed:
cargo run -- --width 54 --height 22 --pace 40 --no-color

# View all available CLI flags:
cargo run -- --help
```

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--theme <THEME>` | `-t` | `creative` | Theme name, index (`1-21`), `random`, or `classic` |
| `--width <WIDTH>` | `-w` | `48` | Grid width in terminal columns |
| `--height <HEIGHT>` | | `20` | Grid height in terminal rows |
| `--pace <MS>` | `-p` | `80` | Milliseconds between animation steps |
| `--interactive` | `-i` | `false` | Launch interactive theme/menu selection |
| `--dry-run` | `-d` | `false` | Offline simulation without LLM API calls |
| `--step` | `-s` | `false` | Single-step mode: press Enter between each action |
| `--snapshot <PATH>` | | — | Save final canvas to file |
| `--no-color` | | `false` | Disable crossterm coloring, plain text output |

## Themes

Each run selects or assigns one of **21 themes**:

- **Creative Freedom** ⭐ *(default)* — unlimited ASCII art across the full canvas with any emoji or ASCII characters, no borders, no rules
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
- **Sacred Geometry Mandala** — radial symmetry with concentric rings and diamond/star cores
- **Enso Fractal Solitude** — minimalist void anchored by a single Enso circle
- **Concentric Rings of Sanzen** — nested circular ripples around triadic rock placements
- **Fractal Starfield Void** — self-similar lattice of stars and geometric crests
- **Yin-Yang Balance** — dual equilibrium of circular sand rings and textured gravel
- **Tabula Rasa (Pure ASCII Muse)** — pure ASCII sketching, no emoji
- **Wild Zones (Unbound Serenity)** — complete freedom across all emoji and symbols
- **Gridwright (The Deliberate Grid as Craft)** — pixel-perfect grid art with chunky blocks and color palettes

## Providers & Models

| Provider | Endpoint | Key prefix | Best for |
|----------|----------|------------|----------|
| **NVIDIA NIM** ⭐ | `https://integrate.api.nvidia.com/v1/chat/completions` | `nvapi-...` | Higher rate limits, faster inference (recommended) |
| **OpenRouter** | `https://openrouter.ai/api/v1/chat/completions` | `sk-or-...` | Free tier models, broad selection |

Any OpenAI-compatible API can be used by setting `LLM_API_URL` to the desired endpoint.

## Setup

1. **Get an API key:**
   - [NVIDIA NIM](https://build.nvidia.com/) (free tier, `nvapi-...`)
   - [OpenRouter](https://openrouter.ai/keys) (free tier, `sk-or-...`)
2. Create your local env file:
   ```bash
   cp .env.example .env
   ```
3. Configure `.env` with your provider:
   ```bash
   # NVIDIA NIM (recommended):
   LLM_API_KEY=nvapi-...
   LLM_API_URL=https://integrate.api.nvidia.com/v1/chat/completions
   LLM_MODEL=nvidia/nemotron-3-nano-30b-a3b

   # OpenRouter (fallback):
   # LLM_API_KEY=sk-or-...
   # LLM_API_URL defaults to OpenRouter
   # LLM_MODEL=tencent/hy3:free
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
| `LLM_API_KEY` | _(required)_ | API key (`nvapi-...`, `sk-or-...`, etc.) |
| `LLM_API_URL` | `https://openrouter.ai/api/v1/chat/completions` | Any OpenAI-compatible endpoint |
| `LLM_MODEL` | `tencent/hy3:free` | Model identifier |

## Layout

- `src/openrouter.rs` — `LlmClient` shared HTTP client: configurable endpoint, Bearer auth, exponential backoff, 429 Retry-After support, OpenRouter-specific headers.
- `src/garden.rs` — `Garden` grid, `crossterm` rendering, `BorderPattern` (12 dynamic border styles), `Action` enum with `execute_action()` dispatch and turtle animation.
- `src/llm.rs` — LLM prompt engineer: 21-theme pool, `FREE_MODELS` allowlist (bypassed with custom `LLM_API_URL`), one-shot composition prompt, offline simulation.
- `src/main.rs` — CLI parser (`clap`), interactive menu (`-i`), `crossterm` screen management, `Ctrl+C` shutdown handler, single-step debugging (`--step`).
- `src/vec.rs` — `Point` geometry, Bresenham lines, Midpoint Circle, distance calculations, filled rectangles.
- `src/color.rs` — RGB color management, 5 pre-defined palettes, palette quantization.
- `src/canvas.rs` — 2D pixel grid with drawing primitives and ANSI 24-bit RGB output.
- `src/pixel_art.rs` — `PixelArtAction` enum for LLM-driven pixel art, `GridwrightConfig`, action executor.
- `src/gridwright_runner.rs` — End-to-end LLM orchestration for Gridwright pixel art sessions.

## Credits

See [CREDITS.md](CREDITS.md) for the full history of contributions to this project.
