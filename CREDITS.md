# Credits

## ZeroClaw — Original Author

| | |
|---|---|
| **Source** | ZeroClaw |
| **Date** | 2026-07-19 |
| **Changes** | Initial project creation. Built and signed the original Rust-forged companion, tending gardens one rock at a time. |

---

## Claude — Anthropic Claude Opus

| | |
|---|---|
| **Source** | Anthropic, Claude Opus |
| **Date** | 2026-07-20 |
| **Changes** | Added dynamic per-session theme system, expanded ASCII glyph palette, stateful prompt engineering (border → raking → rocks → accents → completion), and error resilience with retry logic. |

---

## Antigravity — Google DeepMind

| | |
|---|---|
| **Source** | Google DeepMind |
| **Date** | 2026-07-19 – 2026-07-20 |
| **Issues** | [#9](https://github.com/genidma/karesansui/issues/9), [#10](https://github.com/genidma/karesansui/issues/10), [#19](https://github.com/genidma/karesansui/issues/19) |
| **Changes** | |
| | - Fixed `.env` API key configuration and switched to active OpenRouter free models |
| | - Added animated turtle (`🐢`) pathfinding with rest (`💤`) between turns |
| | - Implemented crossterm terminal renderer & rake animation (no flicker, left-to-right raking) |
| | - State persistence & resume (`--resume`) with JSON serialization |
| | - Single-step debugging (`--step`), offline simulation (`--dry-run`), snapshots (`--snapshot`) |
| | - Rate-limiting pacing & session cycles (6s pace, 30s rest/10, 30-min auto-reset) |
| | - Minimalist mandala & fractal actions (`place_mandala`, `rake_ring`) and 5 geometric themes |
| | - 12 dynamic patterned borders (Sacred Double Box, Seigaiha Waves, Sakura Garland, etc.) |
| | - Interactive startup menu (`-i`) and full CLI integration (`clap`) |
| | - Tabula Rasa (Pure ASCII Muse) and Liberated Wild Zones themes |

---

## Gridwright — GitHub Copilot

| | |
|---|---|
| **Source** | GitHub Copilot |
| **Date** | 2026-07-19 – 2026-07-20 |
| **Changes** | Architected modular, LLM-driven pixel art framework as theme #20: |
| | - `vec.rs` — coordinate geometry with Bresenham lines, Midpoint Circle, distance metrics |
| | - `color.rs` — RGB palette management, 6 pre-defined palettes, blending and quantization |
| | - `canvas.rs` — 2D pixel grid with drawing primitives (lines, circles, rectangles), ANSI 24-bit RGB |
| | - `pixel_art.rs` — LLM action types, GridwrightConfig, system prompt generation, executor |
| | - `gridwright_runner.rs` — end-to-end session orchestration via OpenRouter, retry logic, dry-run |
| | Philosophy: pixel-perfect grid art, every cell deliberate, no auto-scaling or smoothing |

---

## opencode/big-pickle — First Session

| | |
|---|---|
| **Source** | opencode/big-pickle (via opencode on claude.ai) |
| **Date** | 2026-07-28 |
| **Changes** | Resolved compilation errors from `git stash pop` merge conflict: |
| | - Removed stray `}` delimiter |
| | - Deduplicated two `impl Garden` blocks into one |
| | - Fixed move-after-use on `LayeredGlyph` |
| | - Added exhaustive match arms for 4 new `Action` variants |

---

## opencode/big-pickle — Second Session

| | |
|---|---|
| **Source** | opencode/big-pickle (via opencode on claude.ai) |
| **Date** | 2026-07-28 |
| **Changes** | Major refactoring and multi-provider support: |
| | - Extracted shared `LlmClient` from `llm.rs` and `gridwright_runner.rs` into `src/openrouter.rs` |
| | - Replaced ~240-line action dispatch match block with `Garden::execute_action()` (main.rs: 638→388 lines) |
| | - Replaced trig-based `ring_points` with integer-only Midpoint Circle algorithm |
| | - Added graceful Ctrl+C shutdown handler |
| | - Removed ~150 lines of dead code across `canvas.rs`, `color.rs`, `garden.rs`, `vec.rs` |
| | - Made LLM provider configurable (NVIDIA NIM, OpenRouter, or any OpenAI-compatible API) |
| | - Added Retry-After header parsing for 429 rate-limit responses |
| | - Changed default mode to Creative Freedom (unlimited ASCII art, no constraints) |
| | - Changed default pacing to 180s for NVIDIA NIM rate-limit compatibility |
