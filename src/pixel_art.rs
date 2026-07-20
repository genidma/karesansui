/// Pixel Art module: Gridwright-specific LLM integration for runtime pixel art generation.
/// The LLM controls the canvas, palette, and composition through deliberate structural instructions.

use crate::canvas::{Canvas, CanvasBuilder};
use crate::color::{Color, Palette};
use crate::vec::Point;
use serde::{Deserialize, Serialize};

/// An action instruction from the LLM for pixel art composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PixelArtAction {
    /// Set the active palette by name (e.g., "zen_earth", "vibrant_neon").
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
    },

    /// Draw a vertical line from (x, y1) to (x, y2) with a glyph.
    DrawLineV {
        x: usize,
        y1: usize,
        y2: usize,
        glyph: String,
    },

    /// Draw a diagonal line from (x1, y1) to (x2, y2).
    DrawLineDiag {
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        glyph: String,
    },

    /// Draw a circle outline at (cx, cy) with radius r.
    DrawCircle {
        cx: usize,
        cy: usize,
        radius: usize,
        glyph: String,
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
    },

    /// Fill the entire canvas with a background glyph.
    ClearCanvas { glyph: String },

    /// Signal composition complete; render and return the final canvas state.
    Done,
}

/// System prompt template for Gridwright LLM pixel art generation.
pub const GRIDWRIGHT_SYSTEM_PROMPT: &str = r#"You are Gridwright: a master of deliberate grid-as-craft pixel art.
Every cell is a choice. Every glyph is intentional. You compose pixel art by commanding a canvas through structured, mathematical actions.

Your canvas is {{WIDTH}} cells wide × {{HEIGHT}} cells tall.
Each cell is one pixel. No scaling. No smoothing. Pure deliberate placement.

You have access to these palette options:
- "monochrome" — pure black/white (classic ASCII)
- "zen_earth" — sand, stone, soil tones (meditative)
- "night_sky" — deep blues and silvers (nocturnal)
- "vibrant_neon" — electric colors (energetic)
- "warm_earth" — organic, earthy tones
- "gridwright_default" — clean grays, steel blue, crimson accents (recommended)

You compose by returning JSON actions in sequence:
1. SetPalette — choose your palette
2. SetPixel, DrawLineH/V/Diag, DrawCircle, FillCircle, FillRectangle, DrawRectangle — build your subject
3. Done — complete

Your subject & composition are driven by:
- **Subject**: {{SUBJECT}} (what you will depict)
- **Palette**: {{PALETTE}} (the color intent)
- **Composition**: {{COMPOSITION}} (balance, negative space, clean edges)

Every action is deliberate. Every choice matters. No filler. No approximation.
Balance positive and negative space. Leave breathing room. Keep edges clean and defined.

Return your next action as valid JSON, nothing else."#;

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
            subject: "A zen garden with minimal stones".to_string(),
            palette: "Zen Earth".to_string(),
            composition: "Balanced, with negative space".to_string(),
            max_actions: 50,
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

            PixelArtAction::DrawLineH { y, x1, x2, glyph } => {
                canvas.draw_line(Point::new(*x1, *y), Point::new(*x2, *y), glyph);
                Ok(false)
            }

            PixelArtAction::DrawLineV { x, y1, y2, glyph } => {
                canvas.draw_line(Point::new(*x, *y1), Point::new(*x, *y2), glyph);
                Ok(false)
            }

            PixelArtAction::DrawLineDiag {
                x1,
                y1,
                x2,
                y2,
                glyph,
            } => {
                canvas.draw_line(Point::new(*x1, *y1), Point::new(*x2, *y2), glyph);
                Ok(false)
            }

            PixelArtAction::DrawCircle {
                cx,
                cy,
                radius,
                glyph,
            } => {
                canvas.draw_circle(Point::new(*cx, *cy), *radius, glyph);
                Ok(false)
            }

            PixelArtAction::FillCircle {
                cx,
                cy,
                radius,
                glyph,
                color_index,
            } => {
                if let Some(idx) = color_index {
                    if let Some(color) = palette.get(*idx) {
                        for y in 0..canvas.height {
                            for x in 0..canvas.width {
                                let point = Point::new(x, y);
                                let dist_sq =
                                    Point::new(*cx, *cy).distance_squared(point);
                                if dist_sq <= radius * radius {
                                    canvas.set_pixel_colored(point, glyph, color);
                                }
                            }
                        }
                    } else {
                        canvas.fill_circle(Point::new(*cx, *cy), *radius, glyph);
                    }
                } else {
                    canvas.fill_circle(Point::new(*cx, *cy), *radius, glyph);
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
                if let Some(idx) = color_index {
                    if let Some(color) = palette.get(*idx) {
                        canvas.fill_rectangle_colored(
                            Point::new(*x1, *y1),
                            Point::new(*x2, *y2),
                            glyph,
                            color,
                        );
                    } else {
                        canvas.fill_rectangle(Point::new(*x1, *y1), Point::new(*x2, *y2), glyph);
                    }
                } else {
                    canvas.fill_rectangle(Point::new(*x1, *y1), Point::new(*x2, *y2), glyph);
                }
                Ok(false)
            }

            PixelArtAction::DrawRectangle {
                x1,
                y1,
                x2,
                y2,
                glyph,
            } => {
                let (min_x, max_x) = if x1 <= x2 { (*x1, *x2) } else { (*x2, *x1) };
                let (min_y, max_y) = if y1 <= y2 { (*y1, *y2) } else { (*y2, *y1) };

                // Top and bottom
                canvas.draw_line(Point::new(min_x, min_y), Point::new(max_x, min_y), glyph);
                canvas.draw_line(Point::new(min_x, max_y), Point::new(max_x, max_y), glyph);

                // Left and right
                canvas.draw_line(Point::new(min_x, min_y), Point::new(min_x, max_y), glyph);
                canvas.draw_line(Point::new(max_x, min_y), Point::new(max_x, max_y), glyph);

                Ok(false)
            }

            PixelArtAction::ClearCanvas { glyph } => {
                canvas.clear();
                for y in 0..canvas.height {
                    for x in 0..canvas.width {
                        canvas.set_pixel(Point::new(x, y), glyph);
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
        };

        let _ = PixelArtExecutor::execute(&action, &mut canvas, &palette);

        // Check corners
        assert_eq!(canvas.get_pixel(Point::new(2, 2)).unwrap().glyph, "•");
        assert_eq!(canvas.get_pixel(Point::new(8, 2)).unwrap().glyph, "•");
        assert_eq!(canvas.get_pixel(Point::new(2, 8)).unwrap().glyph, "•");
        assert_eq!(canvas.get_pixel(Point::new(8, 8)).unwrap().glyph, "•");
    }
}
