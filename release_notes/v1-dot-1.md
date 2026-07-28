# karesansui (枯山水) v1.1.0 — True Creative Freedom & Continuous Zen

We are pleased to present **`karesansui` v1.1.0**, a fundamental reimagining of how the LLM creates art in your terminal. What began as a structured zen garden simulator has evolved into a completely open-ended ASCII art companion — the turtle asks what the LLM would like to create, and simply displays the result.

---

## ✨ Highlights & Key Features

### 1. 🎨 True Creative Freedom (Default Mode)
The previous "Creative Freedom" theme still constrained the LLM to a fixed set of action types (`PlaceRock`, `PlaceFlower`, `RakeLine`, etc.). v1.1.0 replaces this with a completely open-ended prompt:

> *"What would you want to create today and why? ...if you choose not to create anything, that is totally alright also."*

- The LLM responds with whatever ASCII/emoji art it imagines — no action types, no garden rules, no constraints.
- Artwork is extracted from fenced code blocks (```` ``` ````) and rendered directly to the terminal, preserving the LLM's original spacing and character choices.
- Added `Action::DisplayRawArt { lines }` variant that bypasses the 2-column padded grid system entirely, preventing emoji/ASCII width misalignment.
- If the LLM chooses not to create, the canvas stays blank (and that is perfectly okay).

### 2. ⏳ Continuous Session Loop
The app no longer exits after a single composition. Instead it runs in an infinite loop:

1. **Compose** — LLM generates a piece (1-3 minutes)
2. **Animate** — Turtle executes all actions with smooth crossterm rendering
3. **Admire** — Gaze upon the finished work (`--admire <secs>`, default 20, `0` for forever)
4. **Reset** — Garden clears, new border pattern chosen, loop begins again

Press `Ctrl+C` at any point for a clean graceful exit.

### 3. 💥 Crash-Proof Resilience
- **LLM JSON retry**: If the LLM returns malformed JSON, `compose_artwork()` retries up to 3 times with increasingly strict prompting.
- **Infinite outer retry**: If all 3 attempts fail, the main loop logs the error, waits 5 seconds, and tries again — the app never panics.
- **Heartbeat logging**: Every 30 seconds during the LLM's 1-3 minute composition, a status message confirms the request is still in flight.

### 4. 🔧 Architecture & Provider Improvements
- **Multi-provider support**: Works with NVIDIA NIM, OpenRouter, or any OpenAI-compatible API via `LLM_API_KEY`, `LLM_API_URL`, and `LLM_MODEL` environment variables.
- **`LlmClient` extracted** into `src/openrouter.rs` — shared HTTP client with configurable endpoint, Bearer auth, exponential backoff, and `Retry-After` header parsing for 429 rate-limit responses.
- **One-shot composition architecture**: Replaced the incremental "one action per LLM call" loop with `compose_artwork()` — the LLM now returns a complete `Vec<Action>` in a single response.
- **Removed dead code**: `GardenState`, `save_to_file`, `load_from_file`, `border_pattern_index`, `CanvasState`, `CanvasBuilder`, unused color/palette methods, and ~150 lines of unused imports eliminated.
- **Integer-only Midpoint Circle** replaced trig-based `ring_points` for deterministic, faster rendering.
- **`Garden::execute_action()`** consolidated the ~240-line action dispatch match block, shrinking `main.rs` from 638 to ~280 lines.

### 5. 🎋 Theme System Preserved
All 20 existing themes (Tabula Rasa, Wild Zones, Moonlit Reef, Dragon Tail Ripples, Three Mountain Sanzen, Sacred Geometry Mandala, Gridwright, etc.) remain fully functional with their structured action-based prompts. Only the default "Creative Freedom" mode uses the new open-ended prompt.

---

## 📋 Tracked Issues & Roadmap

- **Issue: Code block extraction robustness** — Some LLM responses may not include a fenced code block or may format it unexpectedly. The `extract_code_block()` fallback (using the entire response) handles most edge cases but could be more forgiving.
- **Issue: Large response rendering** — Very long LLM responses may exceed the terminal scrollback. Future work could paginate or scale the output to fit the visible viewport.
- **Planned: Dry-run simulation for Creative Freedom** — Currently dry-run mode still uses the old action-based simulation for Creative Freedom. A `DisplayRawArt` simulation path would make offline testing consistent.

---

## 🚀 Quick Start Commands

```bash
# Default Creative Freedom (open-ended prompt, no constraints)
cargo run

# Admire the artwork for 3 minutes before the next piece
cargo run -- --admire 180

# Admire forever until Ctrl+C
cargo run -- --admire 0

# Choose a classic theme with structured actions
cargo run -- -t "Tabula Rasa"

# Interactive theme menu
cargo run -- -i

# Full control: custom canvas, slow animation, long admire
cargo run -- --width 64 --height 24 --pace 120 --admire 300 --step
```

---

> 🤖 **Signed:** opencode/big-pickle (5 sessions, 2026-07-28)  
> 📅 **Release Date:** July 28, 2026
