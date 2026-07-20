# karesansui (枯山水) v1.0.0 — The Deliberate Grid & Minimalist Zen Garden

We are thrilled to present **`karesansui` v1.0.0**, a major evolution of our minimalist Zen garden, mandala, and ASCII/pixel art terminal generator.

This milestone release introduces **Gridwright**, our flagship runtime LLM-driven pixel art generator powered by OpenRouter, built on top of a robust geometric engine.

---

## ✨ Highlights & Key Features

### 1. 🎨 Gridwright — Runtime LLM Pixel Craft
Gridwright transforms the CLI from static or pre-rendered patterns into a live, multi-turn AI art studio where an LLM constructs deliberate pixel masterpieces turn-by-turn right in your terminal.
- **Strict Geometric & Rendering Foundations (`vec`, `color`, `canvas`)**:
  - **`vec`**: Precise grid positioning and coordinate math (`Point`, Manhattan distance, line tracing, circle/rectangle outlines and fills).
  - **`color`**: Curated 8-color `gridwright_spec` palette (Deep space, Slate blue, Teal, Cyan glow, Silver, Warm gold, Crimson, and Pure white) with automatic RGB distance blending and nearest-in-palette assignment.
  - **`canvas`**: Exact 2-column square blocks (`"  "` / colored background escape codes) to guarantee crisp 1:1 pixel proportions regardless of terminal line height.
- **Advanced LLM Instruction Toolkit (`GRIDWRIGHT_SYSTEM_PROMPT`)**:
  - The LLM is provided with a comprehensive geometric and coloring manual teaching it how to leverage all canvas primitives: `clear_canvas`, `fill_rectangle`, `draw_rectangle`, `fill_circle`, `draw_circle`, `draw_line_h`, `draw_line_v`, `draw_line_diag`, `draw_path`, and `set_pixel`.
  - Enforces bold compositions, strong negative space, chunky blocks over cluttered detail, and up to 20 progressive execution turns.

### 2. ⚡ Live Terminal Animation & CLI Control
- **Flicker-Free Live Rendering**: Utilizes clean ANSI terminal restoration and buffer rendering (`render_live_screen`) so you can watch the artwork materialize frame by frame without screen tearing.
- **Configurable Pacing (`--pace`)**: Control the speed of generation (`--pace <seconds>` or environment variable `KARESANSUI_TICK_MS`)—from relaxing 6-second Zen contemplation to instant `< 50ms` rapid previews.
- **Step-Through Mode (`--step`)**: Inspect individual actions step-by-step for fine-grained debugging.
- **Snapshot Export (`--snapshot`)**: Save the finished, full-color (or monochrome `--no-color`) canvas directly to disk as a text artifact (`--snapshot output.txt`).
- **Dynamic Sizing (`--width` / `--height`)**: Override default grid dimensions at launch to fit custom terminal windows.

### 3. 🎋 Classical & Tabula Rasa Themes
- Includes our rich library of 14+ themes: from traditional raked gravel patterns and meditative mandalas to wild zone generative freedom (`Tabula Rasa` / `Wild Zones`).
- Built-in `crossterm` UI management with guaranteed clean terminal exit (`CleanExit` guard restoring hidden cursors on drop).

---

## 📋 Tracked Issues & Up Next (v1.0.1+ Roadmap)

To maintain absolute transparency, our open refinement items are tracked directly on GitHub:
- **[Issue #1](https://github.com/genidma/karesansui/issues/1) — Interactive & Step Mode Controls**: Refining `--step` behavior to prompt for user steering (`[Press Enter for next turn]`) instead of single-turn exit, and exposing runtime steering options inside the `-i` (`--interactive`) menu.
- **[Issue #2](https://github.com/genidma/karesansui/issues/2) — Domain API Cleanup**: Adding explicit `#[allow(dead_code)]` annotations or trimming unused library helpers across `canvas`, `color`, and `vec` so compiler check output is zero-warning clean.

---

## 🚀 Quick Start Commands

```bash
# Launch Gridwright with live OpenRouter generation (3 seconds between turns)
cargo run -- -t Gridwright --pace 3

# Save a finished snapshot to a file upon completion
cargo run -- -t Gridwright --pace 1 --snapshot my_portrait.txt

# Launch the interactive theme menu
cargo run -- -i
```

---

> 🤖 **Signed:** Antigravity, powered by **Gemini 2.5 Pro**  
> 📅 **Release Prep Date:** July 20, 2026
