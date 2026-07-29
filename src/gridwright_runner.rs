use crate::canvas::Canvas;
use crate::color::{palettes, Palette};
use crate::openrouter::LlmClient;
use crate::pixel_art::{GridwrightConfig, PixelArtAction, PixelArtExecutor};
use anyhow::{Context, Result};
use std::time::Duration;

/// GridwrightRunner orchestrates pixel art generation via LLM.
pub struct GridwrightRunner {
    client: Option<LlmClient>,
    config: GridwrightConfig,
    dry_run: bool,
    pace: Duration,
    step: bool,
    no_color: bool,
    snapshot_path: Option<String>,
    steering: Option<String>,
}

impl GridwrightRunner {
    pub fn new(
        api_key: String,
        model: String,
        config: GridwrightConfig,
        dry_run: bool,
    ) -> Self {
        let client = if dry_run {
            None
        } else {
            Some(LlmClient::new(api_key, model))
        };
        GridwrightRunner {
            client,
            config,
            dry_run,
            pace: Duration::from_millis(1500),
            step: false,
            no_color: false,
            snapshot_path: None,
            steering: None,
        }
    }

    pub fn with_pace(mut self, pace: Duration) -> Self {
        self.pace = pace;
        self
    }

    pub fn with_step(mut self, step: bool) -> Self {
        self.step = step;
        self
    }

    pub fn with_no_color(mut self, no_color: bool) -> Self {
        self.no_color = no_color;
        self
    }

    pub fn with_snapshot_path(mut self, path: Option<String>) -> Self {
        self.snapshot_path = path;
        self
    }

    /// Helper: render live canvas screen without flicker using crossterm.
    fn render_live_screen(&self, header: &str, canvas: &Canvas) -> Result<()> {
        use crossterm::{cursor, terminal};
        use std::io::Write;
        let mut stdout = std::io::stdout();
        let _ = crossterm::queue!(stdout, cursor::Hide, cursor::MoveTo(0, 0));
        let rendered = if self.no_color {
            canvas.render()
        } else {
            canvas.render_with_colors()
        };
        let full_text = format!("{header}\n\n{rendered}");
        for line in full_text.lines() {
            let _ = crossterm::queue!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine));
            let _ = writeln!(stdout, "{line}");
        }
        let _ = crossterm::queue!(stdout, terminal::Clear(terminal::ClearType::FromCursorDown));
        let _ = stdout.flush();
        Ok(())
    }

    /// Run a complete Gridwright session: initialize canvas, get LLM actions, and execute them with live rendering.
    pub async fn run(&mut self) -> Result<Canvas> {
        let mut canvas = Canvas::new(self.config.width, self.config.height);
        let palette = self.select_palette(&self.config.palette);

        log::info!(
            "🎨 Gridwright Session: {} × {} grid | Subject: {} | Palette: {}",
            self.config.width,
            self.config.height,
            self.config.subject,
            self.config.palette
        );

        canvas.set_palette(&self.config.palette);

        let header = format!(
            "🎨 Gridwright — Subject: \"{}\" | Palette: \"{}\"  [initializing...]",
            self.config.subject,
            canvas.palette.as_deref().unwrap_or(&self.config.palette)
        );
        let _ = self.render_live_screen(&header, &canvas);
        tokio::time::sleep(Duration::from_millis(500)).await;

        let mut action_count = 0;
        loop {
            action_count += 1;
            log::debug!("Action #{}", action_count);

            if action_count > self.config.max_actions {
                log::info!(
                    "✅ Reached max actions ({}/{}), wrapping up.",
                    action_count - 1,
                    self.config.max_actions
                );
                break;
            }

            let header = format!(
                "🎨 Gridwright — Subject: \"{}\" | Palette: \"{}\"  [action #{}/{} — thinking...]",
                self.config.subject,
                canvas.palette.as_deref().unwrap_or(&self.config.palette),
                action_count,
                self.config.max_actions
            );
            let _ = self.render_live_screen(&header, &canvas);

            let action = match self.get_next_action(&canvas, action_count).await {
                Ok(a) => a,
                Err(e) => {
                    log::warn!("Failed to get action #{}: {}", action_count, e);
                    tokio::time::sleep(Duration::from_millis(1500)).await;
                    continue;
                }
            };

            let is_done = match PixelArtExecutor::execute(&action, &mut canvas, &palette) {
                Ok(done) => done,
                Err(e) => {
                    log::warn!("Executor error on action #{}: {}", action_count, e);
                    false
                }
            };

            let header = format!(
                "🎨 Gridwright — Subject: \"{}\" | Palette: \"{}\"  [action #{}/{}]",
                self.config.subject,
                canvas.palette.as_deref().unwrap_or(&self.config.palette),
                action_count,
                self.config.max_actions
            );
            let _ = self.render_live_screen(&header, &canvas);

            if let Some(ref snapshot_path) = self.snapshot_path {
                let rendered = if self.no_color {
                    canvas.render()
                } else {
                    canvas.render_with_colors()
                };
                let _ = std::fs::write(snapshot_path, rendered);
            }

            if is_done {
                let header = format!(
                    "🎨 Gridwright — Subject: \"{}\" | Palette: \"{}\"  [✨ Masterpiece Complete!]",
                    self.config.subject,
                    canvas.palette.as_deref().unwrap_or(&self.config.palette)
                );
                let _ = self.render_live_screen(&header, &canvas);
                log::info!("🎨 Composition complete after {} actions", action_count);
                break;
            }

            if self.step {
                log::info!("Step completed (action #{action_count}).");
                print!("\n  Press Enter for next action, or type instructions to steer the LLM: ");
                use std::io::Write;
                let _ = std::io::stdout().flush();
                let mut input = String::new();
                let _ = std::io::stdin().read_line(&mut input);
                let trimmed = input.trim().to_string();
                if trimmed.is_empty() {
                    self.steering = None;
                } else {
                    self.steering = Some(trimmed);
                }
            } else if self.pace > Duration::ZERO {
                tokio::time::sleep(self.pace).await;
            }
        }

        Ok(canvas)
    }

    /// Get the next action from the LLM.
    async fn get_next_action(&self, canvas: &Canvas, action_num: usize) -> Result<PixelArtAction> {
        let canvas_preview = self.render_canvas_preview(canvas);
        let system_prompt = self.config.generate_system_prompt();
        let steering = self
            .steering
            .as_ref()
            .map(|s| format!("\nUser steering: {s}\n"))
            .unwrap_or_default();
        let user_prompt = format!(
            "Canvas state (action #{} of {} max):\n{canvas_preview}{steering}\n\nProvide your next action as ONE single valid JSON object from your capability toolkit (`clear_canvas`, `fill_rectangle`, `draw_rectangle`, `fill_circle`, `draw_circle`, `draw_line_h`, `draw_line_v`, `draw_line_diag`, `draw_path`, `set_pixel`, or `done`). Use exact coordinate math (`x: 0..{}`, `y: 0..{}`), `color_index` (`0..7`), and 2-character wide block glyphs (`██`, `▓▓`, `▒▒`, `░░`, `■ `, `  `).",
            action_num, self.config.max_actions, self.config.width.saturating_sub(1), self.config.height.saturating_sub(1)
        );

        if self.dry_run {
            return self.simulate_action();
        }

        let client = self.client.as_ref().unwrap();
        let content = client.call_raw(&system_prompt, &user_prompt, 0.7, "karesansui-gridwright").await?;

        let action: PixelArtAction = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse Gridwright action: '{content}'"))?;
        Ok(action)
    }

    /// Render a compact preview of the canvas for the LLM.
    fn render_canvas_preview(&self, canvas: &Canvas) -> String {
        let mut preview = String::new();
        preview.push_str(&format!(
            "[Canvas {}×{} | Pixels used: {} | Palette: {}]\n",
            canvas.width,
            canvas.height,
            self.count_filled_pixels(canvas),
            canvas.palette.as_deref().unwrap_or("none")
        ));

        if canvas.width > 32 || canvas.height > 16 {
            preview.push_str("(Downsampled view)\n");
            for y in (0..canvas.height).step_by(2) {
                for x in (0..canvas.width).step_by(2) {
                    if let Some(pixel) = canvas.get_pixel(crate::vec::Point::new(x, y)) {
                        if pixel.glyph != "  " && pixel.glyph != " " {
                            preview.push_str(&pixel.glyph);
                        } else {
                            preview.push('·');
                        }
                    }
                }
                preview.push('\n');
            }
        } else {
            preview.push_str(&canvas.render());
        }

        preview
    }

    /// Count filled pixels in the canvas.
    fn count_filled_pixels(&self, canvas: &Canvas) -> usize {
        canvas
            .pixels
            .iter()
            .flat_map(|row| row.iter())
            .filter(|p| p.glyph != "  " && p.glyph != " ")
            .count()
    }

    /// Select a palette by name.
    fn select_palette(&self, name: &str) -> Palette {
        match name.to_lowercase().as_str() {
            "monochrome" => palettes::monochrome(),
            "zen_earth" => palettes::zen_earth(),
            "night_sky" => palettes::night_sky(),
            "vibrant_neon" => palettes::vibrant_neon(),
            "warm_earth" => palettes::warm_earth(),
            "gridwright_spec" | "gridwright" | "gridwright_default" | "default" => palettes::gridwright_spec(),
            _ => palettes::gridwright_spec(),
        }
    }

    /// Simulate an action for dry-run mode.
    fn simulate_action(&self) -> Result<PixelArtAction> {
        use rand::Rng;
        use rand::seq::IndexedRandom;
        let mut rng = rand::rng();
        let max_x = self.config.width.saturating_sub(1);
        let max_y = self.config.height.saturating_sub(1);

        let choice = rng.random_range(0..8);
        Ok(match choice {
            0 => PixelArtAction::SetPixel {
                x: rng.random_range(0..=max_x),
                y: rng.random_range(0..=max_y),
                glyph: ["██", "▓▓", "▒▒", "░░", "■ ", "▪ "].choose(&mut rng).unwrap().to_string(),
                color_index: Some(rng.random_range(0..8)),
            },
            1 => PixelArtAction::DrawLineH {
                y: rng.random_range(0..=max_y),
                x1: 0,
                x2: max_x,
                glyph: "██".to_string(),
                color_index: Some(rng.random_range(0..8)),
            },
            2 => PixelArtAction::DrawLineV {
                x: rng.random_range(0..=max_x),
                y1: 0,
                y2: max_y,
                glyph: "██".to_string(),
                color_index: Some(rng.random_range(0..8)),
            },
            3 => PixelArtAction::DrawCircle {
                cx: max_x / 2,
                cy: max_y / 2,
                radius: rng.random_range(2..8),
                glyph: "▓▓".to_string(),
                color_index: Some(rng.random_range(0..8)),
            },
            4 => PixelArtAction::FillRectangle {
                x1: rng.random_range(0..max_x / 2),
                y1: rng.random_range(0..max_y / 2),
                x2: rng.random_range(max_x / 2..=max_x),
                y2: rng.random_range(max_y / 2..=max_y),
                glyph: "██".to_string(),
                color_index: Some(rng.random_range(0..8)),
            },
            5 => PixelArtAction::SetPalette {
                palette_name: ["monochrome", "zen_earth", "night_sky", "gridwright_spec"]
                    .choose(&mut rng)
                    .unwrap()
                    .to_string(),
            },
            6 => PixelArtAction::DrawPath {
                points: vec![(2, 2), (6, 5), (10, 3), (13, 8)],
                glyph: "██".to_string(),
                color_index: Some(rng.random_range(0..8)),
            },
            7 => PixelArtAction::DrawRectangle {
                x1: 3,
                y1: 3,
                x2: 11,
                y2: 11,
                glyph: "▒▒".to_string(),
                color_index: Some(rng.random_range(0..8)),
            },
            _ => PixelArtAction::Done,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gridwright_runner_creation() {
        let config = GridwrightConfig::new(32, 16)
            .with_subject("A mountain landscape")
            .with_palette("zen_earth");

        let runner = GridwrightRunner::new(
            "test_key".to_string(),
            "test_model".to_string(),
            config,
            true,
        );

        assert_eq!(runner.config.width, 32);
        assert_eq!(runner.config.height, 16);
    }

    #[test]
    fn test_select_palette() {
        let config = GridwrightConfig::new(32, 16);
        let runner = GridwrightRunner::new(
            "test".to_string(),
            "test".to_string(),
            config,
            true,
        );

        let p1 = runner.select_palette("zen_earth");
        assert_eq!(p1.name, "Zen Earth");

        let p2 = runner.select_palette("unknown");
        assert_eq!(p2.name, "Gridwright");
    }
}
