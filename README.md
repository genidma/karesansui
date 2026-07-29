# karesansui

> CLI that generates creative ASCII art in your terminal.

`karesansui` (枯山水) is a tiny Rust CLI that generates ASCII art on a terminal canvas.
An LLM (via any OpenAI-compatible API — NVIDIA NIM, OpenRouter, etc.) composes a complete
artwork in a single shot: 20-40 actions that create a full composition, then animates each
action step-by-step, bringing the artwork to life right in your terminal.

**Every run starts in Creative Freedom mode** — the LLM is asked a completely open-ended
question: "What would you want to create today and why?" It can respond with any ASCII art it
imagines, using any emoji or character, with zero constraints. If it chooses not to create anything,
we just sit and stare at a blank terminal.

## How it works

- The LLM is asked a completely open-ended question: "What would you like to create today?"
- It responds with raw ASCII/emoji art in a fenced code block, plus a narrative explanation.
- The artwork is rendered directly on the canvas — no action types, no constraints.
- If the LLM chooses not to create, the canvas stays blank (and that's okay).
- After completion, the piece is displayed for admiration, then the canvas clears and a new piece begins.
- Use `-d` for dry-run simulation, `--step` to approve actions one-by-one.
- If network rate-limits (429) occur, the engine reads the `Retry-After` header and backs off.
- Press `Ctrl+C` at any time for graceful shutdown.

## Commands & Usage

Run with zero arguments for the default Creative Freedom experience, or customize using
CLI flags and the interactive startup menu:

### Interactive Menu Mode (`-i` / `--interactive`)

```bash
cargo run -- -i
```

### Command-Line Flags

```bash
# Run an offline simulation (dry run) with single-step verification:
cargo run -- --dry-run --step --snapshot garden_dump.txt

# Customize canvas size, animation speed, and admiration time:
cargo run -- --width 54 --height 22 --pace 40 --admire 180 --no-color

# Admire forever until Ctrl+C:
cargo run -- --admire 0

# View all available CLI flags:
cargo run -- --help
```

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--width <WIDTH>` | `-w` | `48` | Grid width in terminal columns |
| `--height <HEIGHT>` | | `20` | Grid height in terminal rows |
| `--pace <MS>` | `-p` | `80` | Milliseconds between animation steps |
| `--interactive` | `-i` | `false` | Launch interactive menu selection |
| `--dry-run` | `-d` | `false` | Offline simulation without LLM API calls |
| `--step` | `-s` | `false` | Single-step mode: press Enter between each action |
| `--snapshot <PATH>` | | — | Save final canvas to file |
| `--no-color` | | `false` | Disable crossterm coloring, plain text output |
| `--admire <SECS>` | | `20` | Seconds to admire artwork before next piece (`0` = forever until Ctrl+C) |

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

## Docker

karesansui is also available as a Docker image — no Rust toolchain required.

### Build

```bash
docker build -t karesansui .
```

> **Note:** The first build downloads `rust:alpine` (~700MB) for the compiler toolchain, then compiles all Rust dependencies — expect several minutes. Docker caches everything after that, so subsequent builds are near-instant. The final tagged runtime image is only ~15-20MB.

### Run

The container requires an interactive terminal (`-it`) and your LLM API credentials. Use `--rm` to clean up automatically on exit.

**With a `.env` file** (recommended):

```bash
# Create your .env from the example first
cp .env.example .env
# edit .env with your API key and provider
docker run -it --rm --env-file .env karesansui
```

**With inline environment variables:**

```bash
docker run -it --rm -e LLM_API_KEY=sk-or-... karesansui
```

### Passing CLI flags

Append flags after `--` so they are passed to the binary rather than to Docker:

```bash
# Interactive menu mode
docker run -it --rm --env-file .env karesansui -- -i

# Dry-run with single-step verification, saving output
docker run -it --rm --env-file .env karesansui -- --dry-run --step --snapshot /tmp/garden.txt
```

> Implemented via [#3](https://github.com/genidma/karesansui/issues/3).

## Configuration

| Env var | Default | Notes |
|---------|---------|-------|
| `LLM_API_KEY` | _(required)_ | API key (`nvapi-...`, `sk-or-...`, etc.) |
| `LLM_API_URL` | `https://openrouter.ai/api/v1/chat/completions` | Any OpenAI-compatible endpoint |
| `LLM_MODEL` | `tencent/hy3:free` | Model identifier |

## Layout

- `src/openrouter.rs` — `LlmClient` shared HTTP client: configurable endpoint, Bearer auth, exponential backoff, 429 Retry-After support, OpenRouter-specific headers.
- `src/garden.rs` — `Garden` grid, `crossterm` rendering, `BorderPattern` (12 dynamic border styles), `Action` enum with `execute_action()` dispatch.
- `src/llm.rs` — LLM prompt engine: composable prompts, FREE_MODELS allowlist, one-shot composition, offline simulation.
- `src/main.rs` — CLI parser (`clap`), interactive menu (`-i`), `crossterm` screen management, `Ctrl+C` shutdown handler, single-step debugging (`--step`).
- `src/vec.rs` — `Point` geometry, Bresenham lines, Midpoint Circle, distance calculations, filled rectangles.
- `src/color.rs` — RGB color management, 5 pre-defined palettes, palette quantization.
- `src/canvas.rs` — 2D pixel grid with drawing primitives and ANSI 24-bit RGB output.
- `src/pixel_art.rs` — `PixelArtAction` enum for LLM-driven pixel art, `GridwrightConfig`, action executor.
- `src/gridwright_runner.rs` — End-to-end LLM orchestration for Gridwright pixel art sessions.

## Credits

See [CREDITS.md](CREDITS.md) for the full history of contributions to this project.
