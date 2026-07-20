/// Pixel Art module: Gridwright-specific LLM integration for runtime pixel art generation.
/// The LLM controls the canvas, palette, and composition through deliberate structural instructions.

use crate::canvas::Canvas;
use crate::color::Palette;
use crate::vec::Point;
use serde::{Deserialize, Serialize};

/// An action instruction from the LLM for pixel art composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PixelArtAction {
    /// Set the active palette by name (e.g., "gridwright_spec", "zen_earth", "vibrant_neon").
    SetPalette { palette_name: String },

    /// Place a single pixel at (x, y) with a specific glyph.
    SetPixel {
        x: usize,
        y: usize,
        glyph: String,
        color_index: Option<usize>, // Index into the active palette
    },

    /// Draw a horizontal line from (x1, y) to (x2, y) with a glyph.
    DrawLineH {
        y: usize,
        x1: usize,
        x2: usize,
        glyph: String,
        color_index: Option<usize>,
    },

    /// Draw a vertical line from (x, y1) to (x, y2) with a glyph.
    DrawLineV {
        x: usize,
        y1: usize,
        y2: usize,
        glyph: String,
        color_index: Option<usize>,
    },

    /// Draw a diagonal line from (x1, y1) to (x2, y2).
    DrawLineDiag {
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        glyph: String,
        color_index: Option<usize>,
    },

    /// Draw a path through a sequence of points.
    DrawPath {
        points: Vec<(usize, usize)>,
        glyph: String,
        color_index: Option<usize>,
    },

    /// Draw a circle outline at (cx, cy) with radius r.
    DrawCircle {
        cx: usize,
        cy: usize,
        radius: usize,
        glyph: String,
        color_index: Option<usize>,
    },

    /// Fill a circle at (cx, cy) with radius r.
    FillCircle {
        cx: usize,
        cy: usize,
        radius: usize,
        glyph: String,
        color_index: Option<usize>,
    },

    /// Fill a rectangle from (x1, y1) to (x2, y2).
    FillRectangle {
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        glyph: String,
        color_index: Option<usize>,
    },

    /// Draw a bordered rectangle (outline only).
    DrawRectangle {
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        glyph: String,
        color_index: Option<usize>,
    },

    /// Fill the entire canvas with a background glyph.
    ClearCanvas {
        glyph: String,
        color_index: Option<usize>,
    },

    /// Signal composition complete; render and return the final canvas state.
    Done,
}

/// System prompt template for Gridwright LLM pixel art generation.
pub const GRIDWRIGHT_SYSTEM_PROMPT: &str = r#"You are Gridwright: a master of deliberate grid-as-craft pixel art.
Every cell is a choice. Every glyph is intentional. You compose pixel art by commanding a mathematical canvas engine through structured JSON actions.

### 1. THE CANVAS & PIXEL GEOMETRY (`vec` & `canvas` modules)
- Your canvas is exactly {{WIDTH}} cells wide × {{HEIGHT}} cells tall.
- Coordinate system: `x: 0..{{WIDTH_MINUS_ONE}}` (horizontal, left-to-right), `y: 0..{{HEIGHT_MINUS_ONE}}` (vertical, top-to-bottom). Top-left is `(0,0)`.
- Each cell maps to exactly one pixel. Hard edges only: no auto-scaling, no smoothing, no interpolation.
- **Terminal Proportion Rule**: To maintain chunky, visible pixels and true square proportions in the terminal (since terminal fonts are taller than they are wide), you MUST use 2-character wide block glyphs:
  - Solid blocks: `"██"` (solid fill), `"▓▓"` (dense shade / shadow), `"▒▒"` (medium shade / midtone), `"░░"` (light shade / highlight)
  - Geometric accents: `"■ "` (chunky square), `"▪ "` (small square pixel), `"  "` (empty space / void)

### 2. PALETTE & COLOR ASSIGNMENT ENGINE (`color` module)
When using the `"gridwright_spec"` palette, assign colors using `color_index` (integer `0..7`):
- `0`: `#0b0c10` (Deep space / pitch black) — Backgrounds, deep voids, outline dropshadows.
- `1`: `#1f2833` (Slate blue / dark gray) — Structural silhouettes, secondary masses, rocky shadows.
- `2`: `#45a29e` (Teal / mid cyan) — Atmospheric midtones, water depth, foliage shading.
- `3`: `#66fcf1` (Cyan glow) — High-energy accents, neon edges, magical aura, bright highlights.
- `4`: `#c5c6c7` (Light gray / silver) — Metallic sheen, stone highlights, neutral contrast.
- `5`: `#f2a65a` (Warm orange / gold) — Sunlight, fire, sunset horizons, warm focal points.
- `6`: `#ef476f` (Rose red / crimson) — Primary focal point, bold berries/blossoms, dramatic silhouettes.
- `7`: `#ffffff` (Pure white) — Specular reflections, stars, eye glints, crisp rim lighting.

Other selectable palettes via `set_palette`: `"zen_earth"`, `"night_sky"`, `"vibrant_neon"`, `"monochrome"`.

### 3. COMPLETE ACTION TOOLKIT & CAPABILITIES
You compose by returning ONE single valid JSON action per turn. Use the following geometric capabilities:

1. **Set Palette** (turn 1):
   {"action": "set_palette", "palette_name": "{{PALETTE}}"}

2. **Clear / Background Base**:
   {"action": "clear_canvas", "glyph": "  ", "color_index": 0}

3. **Solid Masses & Silhouettes (`fill_rectangle`, `fill_circle`)**:
   - Fill rectangular region (`x1, y1` to `x2, y2`):
     {"action": "fill_rectangle", "x1": 2, "y1": 2, "x2": 13, "y2": 13, "glyph": "██", "color_index": 1}
   - Solid geometric circle (Bresenham midpoint fill):
     {"action": "fill_circle", "cx": 8, "cy": 8, "radius": 4, "glyph": "██", "color_index": 5}

4. **Crisp Outlines & Borders (`draw_rectangle`, `draw_circle`)**:
   - Border outline rectangle (`rect_outline` math):
     {"action": "draw_rectangle", "x1": 2, "y1": 2, "x2": 13, "y2": 13, "glyph": "██", "color_index": 3}
   - 1-pixel circle ring outline:
     {"action": "draw_circle", "cx": 8, "cy": 8, "radius": 5, "glyph": "▓▓", "color_index": 6}

5. **Structural & Contoured Lines (`draw_line_h`, `draw_line_v`, `draw_line_diag`, `draw_path`)**:
   - Horizontal line:
     {"action": "draw_line_h", "y": 10, "x1": 4, "x2": 11, "glyph": "██", "color_index": 7}
   - Vertical line:
     {"action": "draw_line_v", "x": 8, "y1": 4, "y2": 11, "glyph": "██", "color_index": 7}
   - Diagonal segment (Bresenham exact line):
     {"action": "draw_line_diag", "x1": 3, "y1": 3, "x2": 12, "y2": 12, "glyph": "▒▒", "color_index": 4}
   - Connected multi-point path (for mountain ridges, rivers, lightning, organic contours):
     {"action": "draw_path", "points": [[4,4], [7,5], [11,8], [14,12]], "glyph": "██", "color_index": 2}

6. **Precision Pixel Placement (`set_pixel`)**:
   - Single exact pixel (for stars, eye glints, corner bevels, or stippling):
     {"action": "set_pixel", "x": 8, "y": 8, "glyph": "██", "color_index": 7}

7. **Finish & Render**:
   {"action": "done"}

### 4. MASTER PIXEL ART COMPOSITION WORKFLOW
- **Layer 1 (Void & Sky)**: Establish base canvas with `clear_canvas` or broad `fill_rectangle`.
- **Layer 2 (Dominant Silhouettes)**: Lay down major masses (`fill_rectangle`, `fill_circle`) with dark/mid colors (`color_index: 0, 1, 2`).
- **Layer 3 (Structural Contours)**: Define edges, horizons, and architectural form with `draw_line_h/v/diag`, `draw_rectangle`, `draw_circle`, and `draw_path`.
- **Layer 4 (Texturing & Shading)**: Use shade glyphs (`"▓▓"`, `"▒▒"`, `"░░"`) with midtone/accent colors (`color_index: 4, 5, 6`) to create depth and tactile gradients without blurring.
- **Layer 5 (Specular Highlights)**: Place intentional white (`7`) or cyan glow (`3`) single pixels (`set_pixel`) on top-left vertices and focal centers to make the art pop.

### YOUR SESSION BRIEFing
- **Subject**: {{SUBJECT}}
- **Palette**: {{PALETTE}}
- **Composition Intent**: {{COMPOSITION}}

Every action must be deliberate. Think in chunky blocks, sharp silhouettes, and strong structure. Leave breathing room.
Return ONLY valid JSON for your next action, nothing else."#;

/// Configuration for Gridwright pixel art composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridwrightConfig {
    pub width: usize,
    pub height: usize,
    pub subject: String,
    pub palette: String,
    pub composition: String,
    pub max_actions: usize,
}

impl GridwrightConfig {
    pub fn new(width: usize, height: usize) -> Self {
        GridwrightConfig {
            width,
            height,
            subject: "A chunky, balanced pixel composition with strong silhouette and negative space".to_string(),
            palette: "gridwright_spec".to_string(),
            composition: "Balanced, chunky blocks, hard edges, visible pixels, strong negative space".to_string(),
            max_actions: 24,
        }
    }

    pub fn with_subject(mut self, subject: &str) -> Self {
        self.subject = subject.to_string();
        self
    }

    pub fn with_palette(mut self, palette: &str) -> Self {
        self.palette = palette.to_string();
        self
    }

    pub fn with_composition(mut self, composition: &str) -> Self {
        self.composition = composition.to_string();
        self
    }

    pub fn with_max_actions(mut self, max: usize) -> Self {
        self.max_actions = max;
        self
    }

    /// Generate the system prompt for the LLM with substituted values.
    pub fn generate_system_prompt(&self) -> String {
        GRIDWRIGHT_SYSTEM_PROMPT
            .replace("{{WIDTH}}", &self.width.to_string())
            .replace("{{HEIGHT}}", &self.height.to_string())
            .replace("{{WIDTH_MINUS_ONE}}", &self.width.saturating_sub(1).to_string())
            .replace("{{HEIGHT_MINUS_ONE}}", &self.height.saturating_sub(1).to_string())
            .replace("{{SUBJECT}}", &self.subject)
            .replace("{{PALETTE}}", &self.palette)
            .replace("{{COMPOSITION}}", &self.composition)
    }
}

/// Executor for applying PixelArtActions to a canvas.
pub struct PixelArtExecutor;

impl PixelArtExecutor {
    /// Apply a single action to the canvas.
    pub fn execute(action: &PixelArtAction, canvas: &mut Canvas, palette: &Palette) -> Result<bool, String> {
        match action {
            PixelArtAction::SetPalette { palette_name } => {
                canvas.set_palette(palette_name);
                Ok(false) // Continue
            }

            PixelArtAction::SetPixel {
                x,
                y,
                glyph,
                color_index,
            } => {
                let point = Point::new(*x, *y);
                if let Some(idx) = color_index {
                    if let Some(color) = palette.get(*idx) {
                        canvas.set_pixel_colored(point, glyph, color);
                    } else {
                        canvas.set_pixel(point, glyph);
                    }
                } else {
                    canvas.set_pixel(point, glyph);
                }
                Ok(false)
            }

            PixelArtAction::DrawLineH { y, x1, x2, glyph, color_index } => {
                let from = Point::new(*x1, *y);
                let to = Point::new(*x2, *y);
                if let Some(idx) = color_index {
                    if let Some(color) = palette.get(*idx) {
                        canvas.draw_line_colored(from, to, glyph, color);
                    } else {
                        canvas.draw_line(from, to, glyph);
                    }
                } else {
                    canvas.draw_line(from, to, glyph);
                }
                Ok(false)
            }

            PixelArtAction::DrawLineV { x, y1, y2, glyph, color_index } => {
                let from = Point::new(*x, *y1);
                let to = Point::new(*x, *y2);
                if let Some(idx) = color_index {
                    if let Some(color) = palette.get(*idx) {
                        canvas.draw_line_colored(from, to, glyph, color);
                    } else {
                        canvas.draw_line(from, to, glyph);
                    }
                } else {
                    canvas.draw_line(from, to, glyph);
                }
                Ok(false)
            }

            PixelArtAction::DrawLineDiag {
                x1,
                y1,
                x2,
                y2,
                glyph,
                color_index,
            } => {
                let from = Point::new(*x1, *y1);
                let to = Point::new(*x2, *y2);
                if let Some(idx) = color_index {
                    if let Some(color) = palette.get(*idx) {
                        canvas.draw_line_colored(from, to, glyph, color);
                    } else {
                        canvas.draw_line(from, to, glyph);
                    }
                } else {
                    canvas.draw_line(from, to, glyph);
                }
                Ok(false)
            }

            PixelArtAction::DrawPath {
                points,
                glyph,
                color_index,
            } => {
                let mut previous: Option<Point> = None;
                for (x, y) in points {
                    let point = Point::new(*x, *y);
                    if let Some(prev) = previous {
                        for step in prev.line_to(point) {
                            if let Some(idx) = color_index {
                                if let Some(color) = palette.get(*idx) {
                                    canvas.set_pixel_colored(step, glyph, color);
                                } else {
                                    canvas.set_pixel(step, glyph);
                                }
                            } else {
                                canvas.set_pixel(step, glyph);
                            }
                        }
                    }
                    if let Some(idx) = color_index {
                        if let Some(color) = palette.get(*idx) {
                            canvas.set_pixel_colored(point, glyph, color);
                        } else {
                            canvas.set_pixel(point, glyph);
                        }
                    } else {
                        canvas.set_pixel(point, glyph);
                    }
                    previous = Some(point);
                }
                Ok(false)
            }

            PixelArtAction::DrawCircle {
                cx,
                cy,
                radius,
                glyph,
                color_index,
            } => {
                let center = Point::new(*cx, *cy);
                if let Some(idx) = color_index {
                    if let Some(color) = palette.get(*idx) {
                        canvas.draw_circle_colored(center, *radius, glyph, color);
                    } else {
                        canvas.draw_circle(center, *radius, glyph);
                    }
                } else {
                    canvas.draw_circle(center, *radius, glyph);
                }
                Ok(false)
            }

            PixelArtAction::FillCircle {
                cx,
                cy,
                radius,
                glyph,
                color_index,
            } => {
                let center = Point::new(*cx, *cy);
                if let Some(idx) = color_index {
                    if let Some(color) = palette.get(*idx) {
                        canvas.fill_circle_colored(center, *radius, glyph, color);
                    } else {
                        canvas.fill_circle(center, *radius, glyph);
                    }
                } else {
                    canvas.fill_circle(center, *radius, glyph);
                }
                Ok(false)
            }

            PixelArtAction::FillRectangle {
                x1,
                y1,
                x2,
                y2,
                glyph,
                color_index,
            } => {
                let from = Point::new(*x1, *y1);
                let to = Point::new(*x2, *y2);
                if let Some(idx) = color_index {
                    if let Some(color) = palette.get(*idx) {
                        canvas.fill_rectangle_colored(from, to, glyph, color);
                    } else {
                        canvas.fill_rectangle(from, to, glyph);
                    }
                } else {
                    canvas.fill_rectangle(from, to, glyph);
                }
                Ok(false)
            }

            PixelArtAction::DrawRectangle {
                x1,
                y1,
                x2,
                y2,
                glyph,
                color_index,
            } => {
                let from = Point::new(*x1, *y1);
                let to = Point::new(*x2, *y2);
                if let Some(idx) = color_index {
                    if let Some(color) = palette.get(*idx) {
                        canvas.draw_rectangle_colored(from, to, glyph, color);
                    } else {
                        canvas.draw_rectangle(from, to, glyph);
                    }
                } else {
                    canvas.draw_rectangle(from, to, glyph);
                }
                Ok(false)
            }

            PixelArtAction::ClearCanvas { glyph, color_index } => {
                canvas.clear();
                let color = color_index.and_then(|idx| palette.get(idx));
                for y in 0..canvas.height {
                    for x in 0..canvas.width {
                        let point = Point::new(x, y);
                        if let Some(c) = color {
                            canvas.set_pixel_colored(point, glyph, c);
                        } else {
                            canvas.set_pixel(point, glyph);
                        }
                    }
                }
                Ok(false)
            }

            PixelArtAction::Done => Ok(true), // Stop iteration
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::palettes;

    #[test]
    fn test_draw_path_action() {
        let mut canvas = Canvas::new(20, 20);
        let palette = palettes::gridwright_default();

        let action = PixelArtAction::DrawPath {
            points: vec![(1, 1), (3, 2), (5, 1)],
            glyph: "/".to_string(),
            color_index: None,
        };

        let done = PixelArtExecutor::execute(&action, &mut canvas, &palette).unwrap();
        assert!(!done);
        assert_eq!(canvas.get_pixel(Point::new(1, 1)).unwrap().glyph, "/");
        assert_eq!(canvas.get_pixel(Point::new(5, 1)).unwrap().glyph, "/");
    }

    #[test]
    fn test_gridwright_config() {
        let config = GridwrightConfig::new(32, 16)
            .with_subject("A mountain")
            .with_palette("zen_earth");

        assert_eq!(config.width, 32);
        assert_eq!(config.subject, "A mountain");
    }

    #[test]
    fn test_system_prompt_generation() {
        let config = GridwrightConfig::new(32, 16);
        let prompt = config.generate_system_prompt();
        assert!(prompt.contains("32"));
        assert!(prompt.contains("16"));
    }

    #[test]
    fn test_pixel_art_executor() {
        let mut canvas = Canvas::new(20, 20);
        let palette = palettes::gridwright_default();

        let action = PixelArtAction::SetPixel {
            x: 10,
            y: 10,
            glyph: "■".to_string(),
            color_index: None,
        };

        let done = PixelArtExecutor::execute(&action, &mut canvas, &palette).unwrap();
        assert!(!done);
        assert_eq!(canvas.get_pixel(Point::new(10, 10)).unwrap().glyph, "■");
    }

    #[test]
    fn test_draw_rectangle_action() {
        let mut canvas = Canvas::new(20, 20);
        let palette = palettes::gridwright_default();

        let action = PixelArtAction::DrawRectangle {
            x1: 2,
            y1: 2,
            x2: 8,
            y2: 8,
            glyph: "•".to_string(),
            color_index: None,
        };

        let _ = PixelArtExecutor::execute(&action, &mut canvas, &palette);

        // Check corners
        assert_eq!(canvas.get_pixel(Point::new(2, 2)).unwrap().glyph, "•");
        assert_eq!(canvas.get_pixel(Point::new(8, 2)).unwrap().glyph, "•");
        assert_eq!(canvas.get_pixel(Point::new(2, 8)).unwrap().glyph, "•");
        assert_eq!(canvas.get_pixel(Point::new(8, 8)).unwrap().glyph, "•");
    }
}
