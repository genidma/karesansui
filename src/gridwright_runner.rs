/// Gridwright Runner: Full integration for pixel-art LLM generation with canvas system.
/// This runner orchestrates the Gridwright theme end-to-end using OpenRouter LLM calls
/// and the canvas/color/vec modules for precise, mathematical pixel composition.

use crate::canvas::Canvas;
use crate::color::{palettes, Palette};
use crate::pixel_art::{GridwrightConfig, PixelArtAction, PixelArtExecutor};
use anyhow::Result;
use rand::seq::IndexedRandom;
use reqwest;
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ORResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Choice {
    message: ChatMessageOut,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ChatMessageOut {
    content: String,
}

/// GridwrightRunner orchestrates pixel art generation via LLM.
pub struct GridwrightRunner {
    client: reqwest::Client,
    api_key: String,
    model: String,
    config: GridwrightConfig,
    dry_run: bool,
    pace: Duration,
    step: bool,
    no_color: bool,
    snapshot_path: Option<String>,
}

impl GridwrightRunner {
    pub fn new(
        api_key: String,
        model: String,
        config: GridwrightConfig,
        dry_run: bool,
    ) -> Self {
        GridwrightRunner {
            client: reqwest::Client::new(),
            api_key,
            model,
            config,
            dry_run,
            pace: Duration::from_millis(1500),
            step: false,
            no_color: false,
            snapshot_path: None,
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
    pub async fn run(&self) -> Result<Canvas> {
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

        // Initial screen render
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
                log::info!("Step completed (action #{action_count}). Exiting step mode.");
                break;
            }

            if self.pace > Duration::ZERO {
                tokio::time::sleep(self.pace).await;
            }
        }

        Ok(canvas)
    }

    /// Get the next action from the LLM.
    async fn get_next_action(&self, canvas: &Canvas, action_num: usize) -> Result<PixelArtAction> {
        let canvas_preview = self.render_canvas_preview(canvas);
        let system_prompt = self.config.generate_system_prompt();
        let user_prompt = format!(
            "Canvas state (action #{} of {} max):\n{}\n\nProvide your next action as ONE single valid JSON object from your capability toolkit (`clear_canvas`, `fill_rectangle`, `draw_rectangle`, `fill_circle`, `draw_circle`, `draw_line_h`, `draw_line_v`, `draw_line_diag`, `draw_path`, `set_pixel`, or `done`). Use exact coordinate math (`x: 0..{}`, `y: 0..{}`), `color_index` (`0..7`), and 2-character wide block glyphs (`██`, `▓▓`, `▒▒`, `░░`, `■ `, `  `).",
            action_num, self.config.max_actions, canvas_preview, self.config.width.saturating_sub(1), self.config.height.saturating_sub(1)
        );

        if self.dry_run {
            return self.simulate_action();
        }

        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt }
            ],
            "temperature": 0.7,
        });

        let mut backoff = Duration::from_millis(1000);
        let max_attempts = 4;

        for attempt in 1..=max_attempts {
            let resp_result = self
                .client
                .post(OPENROUTER_URL)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .header("HTTP-Referer", "https://github.com/karesansui")
                .header("X-Title", "karesansui-gridwright")
                .json(&body)
                .send()
                .await;

            let resp = match resp_result {
                Ok(r) => r,
                Err(e) => {
                    if attempt < max_attempts {
                        log::warn!(
                            "Network error (attempt {}/{}): {}. Retrying in {:?}...",
                            attempt, max_attempts, e, backoff
                        );
                        tokio::time::sleep(backoff).await;
                        backoff *= 2;
                        continue;
                    } else {
                        return Err(anyhow::anyhow!(
                            "Network error after {} attempts: {}",
                            max_attempts,
                            e
                        ));
                    }
                }
            };

            let status = resp.status();
            if !status.is_success() {
                let err_body = resp.text().await.unwrap_or_default();
                if (status == reqwest::StatusCode::TOO_MANY_REQUESTS
                    || status.is_server_error()
                    || status == reqwest::StatusCode::BAD_REQUEST)
                    && attempt < max_attempts
                {
                    log::warn!(
                        "API error {status} (attempt {}/{max_attempts}): {err_body}. Retrying...",
                        attempt, max_attempts = max_attempts
                    );
                    tokio::time::sleep(backoff).await;
                    backoff *= 2;
                    continue;
                } else {
                    return Err(anyhow::anyhow!("API error {status}: {err_body}"));
                }
            }

            let or_resp: ORResponse = resp.json().await?;
            let content = or_resp
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .ok_or_else(|| anyhow::anyhow!("No choices in response"))?;

            let clean = content
                .trim()
                .strip_prefix("```json")
                .unwrap_or(content.trim())
                .strip_prefix("```")
                .unwrap_or(content.trim())
                .strip_suffix("```")
                .unwrap_or(content.trim())
                .trim();

            let action: PixelArtAction = serde_json::from_str(clean)?;
            return Ok(action);
        }

        Err(anyhow::anyhow!("Exceeded max retry attempts"))
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

        // Show a downsampled version if canvas is large
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
            true, // dry_run
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
