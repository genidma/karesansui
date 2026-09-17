use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A single drawing action returned by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    /// Place a rock at (x, y). `size` 1-3 controls the glyph.
    PlaceRock { x: usize, y: usize, size: u8 },
    /// Rake a horizontal line of sand between two columns on a row.
    RakeLine { y: usize, x1: usize, x2: usize },
    /// Rake a concentric circular ring of sand centered at (cx, cy) with given radius.
    RakeRing { cx: usize, cy: usize, radius: usize },
    /// Place a patch of moss at (x, y).
    PlaceMoss { x: usize, y: usize },
    /// Place a flower patch at (x, y).
    PlaceFlower { x: usize, y: usize },
    /// Place a stone lantern at (x, y).
    PlaceLantern { x: usize, y: usize },
    /// Place a mandala or fractal pattern center at (x, y) with given style (1-6).
    PlaceMandala { x: usize, y: usize, style: u8 },
    /// Place an ASCII minimalist character at (x, y).
    PlaceAscii { x: usize, y: usize, glyph: String },
    /// Draw an ASCII minimalist horizontal line from x1 to x2 at row y using glyph.
    DrawAsciiLine { y: usize, x1: usize, x2: usize, glyph: String },
    /// Place a custom glyph or emoji at (x, y).
    PlaceGlyph { x: usize, y: usize, glyph: String },
    /// Draw a horizontal line of custom glyphs from x1 to x2 at row y.
    DrawLine { y: usize, x1: usize, x2: usize, glyph: String },
    /// Draw a circular ring of custom glyphs centered at (cx, cy) with given radius.
    DrawRing { cx: usize, cy: usize, radius: usize, glyph: String },
    /// Fill a rectangular box from (x1, y1) to (x2, y2) with custom glyphs.
    FillBox { x1: usize, y1: usize, x2: usize, y2: usize, glyph: String },
    /// Clear a cell at (x, y) back to empty.
    ClearCell { x: usize, y: usize },
    /// Place gravel in a horizontal line from x1 to x2 at row y.
    PlaceGravel { y: usize, x1: usize, x2: usize },
    /// Draw a border frame around the whole garden.
    DrawBorder,
    /// Signal that the garden composition is complete.
    Done,
    /// Place a multi-cell glyph pattern.
    PlaceMultiCellGlyph {
        anchor_x: usize,
        anchor_y: usize,
        glyphs: Vec<(usize, usize, String)>, // (dx, dy, glyph)
    },
    /// Draw a flow line with proportional spacing.
    DrawFlowLine {
        points: Vec<(usize, usize)>,
        glyph: String,
    },
    /// Apply glitch escape sequences and corruption.
    ApplyGlitchFilter {
        x: usize,
        y: usize,
        filter_type: GlitchFilterType,
    },
    /// Place a glyph with custom blending.
    PlaceBlendedGlyph {
        x: usize,
        y: usize,
        glyph: String,
        blend_mode: BlendMode,
        opacity: f32,
    },
    /// Display raw ASCII art lines from the LLM's own creation.
    #[serde(skip)]
    DisplayRawArt { lines: Vec<String> },
}

pub const EMPTY: &str = "  ";
#[allow(dead_code)]
pub const BORDER: &str = "🎋";
pub const RAKED: &str = "~~";
pub const ROCK_S: &str = "🪨";
pub const ROCK_M: &str = "🗿";
pub const ROCK_L: &str = "🗻";
pub const MOSS: &str = "🌿";
pub const GRAVEL: &str = "··";
pub const FLOWER: &str = "🌸";
pub const LANTERN: &str = "🏮";

/// Blend modes for overlapping glyphs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BlendMode {
    Replace,     // Standard overwrite
    Add,         // Additive blending
    Multiply,    // Multiply effect
    Difference,  // Difference mode
    Screen,      // Screen effect
    Custom,      // Custom function
}

/// Glitch filter types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GlitchFilterType {
    CharSwap,      // Swap characters randomly
    WidthShift,    // Shift characters horizontally
    HeightShift,   // Shift characters vertically
    ColorInvert,   // Invert color if supported
    GlitchBurst,   // Random corruption bursts
    EscapeSequence,// Apply escape sequences
}

// Mandala & Fractal Minimalist Glyphs (2 columns wide)
pub const ENSO: &str = "⭕";
pub const MANDALA_RING: &str = "◎ ";
pub const MANDALA_CORE: &str = "◈ ";
pub const FRACTAL_STAR: &str = "✦ ";
pub const YIN_YANG: &str = "☯ ";
pub const CREST: &str = "❖ ";

/// A geometric, patterned, and aesthetically pleasing border style.
#[derive(Debug, Clone)]
pub struct BorderPattern {
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_right: &'static str,
    pub top: &'static str,
    pub top_alt: &'static str,
    pub bottom: &'static str,
    pub bottom_alt: &'static str,
    pub left: &'static str,
    pub left_alt: &'static str,
    pub right: &'static str,
    pub right_alt: &'static str,
}

pub const BORDER_PATTERNS: &[BorderPattern] = &[
    BorderPattern {
        top_left: "🎍", top_right: "🎍", bottom_left: "🎍", bottom_right: "🎍",
        top: "🎋", top_alt: "──", bottom: "🎋", bottom_alt: "──",
        left: "│ ", left_alt: "🎋", right: "│ ", right_alt: "🎋",
    },
    BorderPattern {
        top_left: "╔═", top_right: "═╗", bottom_left: "╚═", bottom_right: "═╝",
        top: "══", top_alt: "══", bottom: "══", bottom_alt: "══",
        left: "║ ", left_alt: "║ ", right: "║ ", right_alt: "║ ",
    },
    // 3. Mandala Diamond Lattice
    BorderPattern {
        top_left: "◈ ", top_right: "◈ ", bottom_left: "◈ ", bottom_right: "◈ ",
        top: "◇ ", top_alt: "◈ ", bottom: "◇ ", bottom_alt: "◈ ",
        left: "◇ ", left_alt: "◈ ", right: "◇ ", right_alt: "◈ ",
    },
    // 4. Seigaiha Ocean Waves
    BorderPattern {
        top_left: "🌊", top_right: "🌊", bottom_left: "🌊", bottom_right: "🌊",
        top: "〰〰", top_alt: "≈≈", bottom: "〰〰", bottom_alt: "≈≈",
        left: "≈≈", left_alt: "〰 ", right: "≈≈", right_alt: "〰 ",
    },
    // 5. Stone Pillar & Gravel Shore
    BorderPattern {
        top_left: "⛩️ ", top_right: "⛩️ ", bottom_left: "🗿", bottom_right: "🗿",
        top: "🪨", top_alt: "··", bottom: "🪨", bottom_alt: "··",
        left: "║ ", left_alt: "🪨", right: "║ ", right_alt: "🪨",
    },
    // 6. Starfield Lattice
    BorderPattern {
        top_left: "🌟", top_right: "🌟", bottom_left: "🌟", bottom_right: "🌟",
        top: "✦ ", top_alt: "✧ ", bottom: "✦ ", bottom_alt: "✧ ",
        left: "✦ ", left_alt: "✧ ", right: "✦ ", right_alt: "✧ ",
    },
    // 7. Enso Yin-Yang Harmony
    BorderPattern {
        top_left: "⭕", top_right: "⭕", bottom_left: "⭕", bottom_right: "⭕",
        top: "──", top_alt: "☯ ", bottom: "──", bottom_alt: "☯ ",
        left: "│ ", left_alt: "│ ", right: "│ ", right_alt: "│ ",
    },
    // 8. Sakura Blossom Garland
    BorderPattern {
        top_left: "🌸", top_right: "🌸", bottom_left: "🌸", bottom_right: "🌸",
        top: "──", top_alt: "🌸", bottom: "──", bottom_alt: "🌸",
        left: "│ ", left_alt: "🌸", right: "│ ", right_alt: "🌸",
    },
    // 9. Engawa Wooden Deck
    BorderPattern {
        top_left: "+-", top_right: "-+", bottom_left: "+-", bottom_right: "-+",
        top: "--", top_alt: "==", bottom: "--", bottom_alt: "==",
        left: "| ", left_alt: "| ", right: "| ", right_alt: "| ",
    },
    // 10. Zen Gravel Ridge
    BorderPattern {
        top_left: "░░", top_right: "░░", bottom_left: "░░", bottom_right: "░░",
        top: "▒▒", top_alt: "░░", bottom: "▒▒", bottom_alt: "░░",
        left: "▒▒", left_alt: "░░", right: "▒▒", right_alt: "░░",
    },
    // 11. Minimalist Dotted Lattice
    BorderPattern {
        top_left: "+-", top_right: "-+", bottom_left: "+-", bottom_right: "-+",
        top: "· ", top_alt: "- ", bottom: "· ", bottom_alt: "- ",
        left: ": ", left_alt: "| ", right: ": ", right_alt: "| ",
    },
    // 12. Shimenawa Sacred Rope
    BorderPattern {
        top_left: "❖ ", top_right: "❖ ", bottom_left: "❖ ", bottom_right: "❖ ",
        top: "≈≈", top_alt: "──", bottom: "≈≈", bottom_alt: "──",
        left: "│ ", left_alt: "≈≈", right: "│ ", right_alt: "≈≈",
    },
];

/// The ASCII + emoji zen garden grid.
/// Each cell is a 2-column-wide string so emojis and ASCII mix cleanly.
pub struct Garden {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<String>>,
    /// The aesthetic border pattern framing this session's garden.
    pub border_pattern: BorderPattern,
    /// Raw art lines for direct display (bypasses grid)
    pub raw_art_lines: Option<Vec<String>>,
}

impl Garden {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![vec![EMPTY.to_string(); width]; height];
        use rand::Rng;
        let mut rng = rand::rng();
        let border_pattern = BORDER_PATTERNS[rng.random_range(0..BORDER_PATTERNS.len())].clone();
        Self {
            width,
            height,
            grid,
            border_pattern,
            raw_art_lines: None,
        }
    }

    /// Place a glyph into a single 2-column cell.
    pub fn place_glyph(&mut self, x: usize, y: usize, glyph: &str) {
        if y >= self.height || x >= self.width {
            return;
        }
        self.grid[y][x] = self.format_2col_glyph(glyph);
    }

    pub fn is_empty(&self, x: usize, y: usize) -> bool {
        self.grid[y][x] == EMPTY
    }

    pub fn border_glyph_for(&self, x: usize, y: usize) -> &str {
        let w = self.width.saturating_sub(1);
        let h = self.height.saturating_sub(1);
        let p = &self.border_pattern;
        if x == 0 && y == 0 {
            p.top_left
        } else if x == w && y == 0 {
            p.top_right
        } else if x == 0 && y == h {
            p.bottom_left
        } else if x == w && y == h {
            p.bottom_right
        } else if y == 0 {
            if x % 2 == 0 { p.top } else { p.top_alt }
        } else if y == h {
            if x % 2 == 0 { p.bottom } else { p.bottom_alt }
        } else if x == 0 {
            if y % 2 == 0 { p.left } else { p.left_alt }
        } else if x == w {
            if y % 2 == 0 { p.right } else { p.right_alt }
        } else {
            EMPTY
        }
    }

    pub fn draw_border_at(&mut self, x: usize, y: usize) {
        if y >= self.height || x >= self.width {
            return;
        }
        let glyph = self.border_glyph_for(x, y);
        if glyph != EMPTY {
            self.grid[y][x] = glyph.to_string();
        }
    }

    pub fn place_rock(&mut self, x: usize, y: usize, size: u8) {
        if y >= self.height || x >= self.width {
            return;
        }
        let glyph = match size.clamp(1, 3) {
            1 => ROCK_S,
            2 => ROCK_M,
            _ => ROCK_L,
        };
        self.grid[y][x] = glyph.to_string();
    }

    #[allow(dead_code)]
    pub fn rake_line(&mut self, y: usize, x1: usize, x2: usize) {
        if y >= self.height {
            return;
        }
        let (a, b) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        for x in a..=b.min(self.width.saturating_sub(1)) {
            if self.is_empty(x, y) {
                self.grid[y][x] = RAKED.to_string();
            }
        }
    }

    pub fn ring_points(&self, cx: usize, cy: usize, radius: usize) -> Vec<(usize, usize)> {
        let center = crate::vec::Point::new(cx, cy);
        center
            .circle_points(radius)
            .into_iter()
            .filter(|p| {
                p.x >= 1
                    && p.x < self.width.saturating_sub(1)
                    && p.y >= 1
                    && p.y < self.height.saturating_sub(1)
            })
            .map(|p| (p.x, p.y))
            .collect()
    }

    #[allow(dead_code)]
    pub fn rake_ring(&mut self, cx: usize, cy: usize, radius: usize) {
        let pts = self.ring_points(cx, cy, radius);
        for (x, y) in pts {
            if self.is_empty(x, y) {
                self.grid[y][x] = RAKED.to_string();
            }
        }
    }

    pub fn place_moss(&mut self, x: usize, y: usize) {
        if y >= self.height || x >= self.width {
            return;
        }
        if self.is_empty(x, y) {
            self.grid[y][x] = MOSS.to_string();
        }
    }

    #[allow(dead_code)]
    pub fn place_gravel(&mut self, y: usize, x1: usize, x2: usize) {
        if y >= self.height {
            return;
        }
        let (a, b) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        for x in a..=b.min(self.width.saturating_sub(1)) {
            if self.is_empty(x, y) {
                self.grid[y][x] = GRAVEL.to_string();
            }
        }
    }

    pub fn place_flower(&mut self, x: usize, y: usize) {
        if y >= self.height || x >= self.width {
            return;
        }
        if self.is_empty(x, y) {
            self.grid[y][x] = FLOWER.to_string();
        }
    }

    pub fn place_lantern(&mut self, x: usize, y: usize) {
        if y >= self.height || x >= self.width {
            return;
        }
        if self.is_empty(x, y) {
            self.grid[y][x] = LANTERN.to_string();
        }
    }

    pub fn place_mandala(&mut self, x: usize, y: usize, style: u8) {
        if y >= self.height || x >= self.width {
            return;
        }
        let glyph = match style.clamp(1, 6) {
            1 => ENSO,
            2 => MANDALA_RING,
            3 => MANDALA_CORE,
            4 => FRACTAL_STAR,
            5 => YIN_YANG,
            _ => CREST,
        };
        if self.is_empty(x, y) {
            self.grid[y][x] = glyph.to_string();
        }
    }

    pub fn place_ascii(&mut self, x: usize, y: usize, glyph: &str) {
        if y >= self.height || x >= self.width {
            return;
        }
        let mut clean = String::new();
        for ch in glyph.chars() {
            if ch.is_ascii() && ch != '\n' && ch != '\r' && !ch.is_control() {
                clean.push(ch);
            }
        }
        let display = if clean.is_empty() {
            "  ".to_string()
        } else if clean.len() == 1 {
             format!("{} ", clean)
        } else {
             clean.chars().take(2).collect::<String>()
        };
        self.grid[y][x] = display;
    }

    #[allow(dead_code)]
    pub fn draw_ascii_line(&mut self, y: usize, x1: usize, x2: usize, glyph: &str) {
        if y >= self.height {
            return;
        }
        let (a, b) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        for x in a..=b.min(self.width.saturating_sub(1)) {
            self.place_ascii(x, y, glyph);
        }
    }

    pub fn format_2col_glyph(&self, glyph: &str) -> String {
        let clean: Vec<char> = glyph
            .chars()
            .filter(|c| *c != '\n' && *c != '\r' && !c.is_control())
            .collect();
        if clean.is_empty() {
            return "  ".to_string();
        }
        let first = clean[0];
        if first.is_ascii() {
            if clean.len() >= 2 && clean[1].is_ascii() {
                let mut s = String::new();
                s.push(first);
                s.push(clean[1]);
                s
            } else {
                format!("{} ", first)
            }
        } else {
            first.to_string()
        }
    }

    #[allow(dead_code)]
    pub fn draw_line(&mut self, y: usize, x1: usize, x2: usize, glyph: &str) {
        if y >= self.height {
            return;
        }
        let (a, b) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        for x in a..=b.min(self.width.saturating_sub(1)) {
            self.place_glyph(x, y, glyph);
        }
    }

    #[allow(dead_code)]
    pub fn draw_ring(&mut self, cx: usize, cy: usize, radius: usize, glyph: &str) {
        let pts = self.ring_points(cx, cy, radius);
        for (x, y) in pts {
            self.place_glyph(x, y, glyph);
        }
    }

    #[allow(dead_code)]
    pub fn fill_box(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, glyph: &str) {
        let (min_x, max_x) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        let (min_y, max_y) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        for y in min_y..=max_y.min(self.height.saturating_sub(1)) {
            for x in min_x..=max_x.min(self.width.saturating_sub(1)) {
                self.place_glyph(x, y, glyph);
            }
        }
    }

    #[allow(dead_code)]
    pub fn clear_cell(&mut self, x: usize, y: usize) {
        if y >= self.height || x >= self.width {
            return;
        }
        self.grid[y][x] = EMPTY.to_string();
    }

    #[allow(dead_code)]
    pub fn draw_border(&mut self) {
        for x in 0..self.width {
            self.draw_border_at(x, 0);
            self.draw_border_at(x, self.height.saturating_sub(1));
        }
        for y in 0..self.height {
            self.draw_border_at(0, y);
            self.draw_border_at(self.width.saturating_sub(1), y);
        }
    }

    /// Render the garden to a string for terminal display.
    pub fn render(&self) -> String {
        if let Some(ref lines) = self.raw_art_lines {
            return lines.join("\n");
        }
        let mut out = String::new();
        for row in self.grid.iter() {
            for cell in row.iter() {
                out.push_str(cell);
            }
            out.push('\n');
        }
        out
    }

    pub fn render_colored(&self, no_color: bool) -> String {
        if self.raw_art_lines.is_some() {
            return self.render();
        }
        if no_color {
            return self.render();
        }
        use crossterm::style::Stylize;
        let mut out = String::new();
        for (y, row) in self.grid.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let styled = match cell.as_str() {
                    ROCK_S | ROCK_M | ROCK_L => cell.as_str().dark_grey().bold().to_string(),
                    MOSS => cell.as_str().green().to_string(),
                    FLOWER => cell.as_str().magenta().to_string(),
                    RAKED | "≈≈" | GRAVEL => cell.as_str().dark_cyan().to_string(),
                    EMPTY => cell.clone(),
                    _ => {
                        if x == 0 || y == 0 || x == self.width.saturating_sub(1) || y == self.height.saturating_sub(1) {
                            cell.as_str().yellow().to_string()
                        } else {
                            cell.clone()
                        }
                    }
                };
                out.push_str(&styled);
            }
            out.push('\n');
        }
        out
    }

    /// Render the current garden state to screen with header.
    /// Uses raw_art_lines when available, otherwise renders the garden grid.
    pub fn render_screen(&self, header: &str, no_color: bool) -> Result<()> {
        use crossterm::{cursor, terminal};
        use std::io::Write;
        let mut stdout = std::io::stdout();
        crossterm::queue!(stdout, cursor::Hide, cursor::MoveTo(0, 0), terminal::Clear(terminal::ClearType::All))?;
        let rendered = if let Some(ref lines) = self.raw_art_lines {
            lines.join("\n")
        } else if no_color {
            self.render()
        } else {
            self.render_colored(false)
        };
        writeln!(stdout, "{header}\n")?;
        writeln!(stdout, "{rendered}")?;
        stdout.flush()?;
        Ok(())
    }

    /// Reset the garden for a new piece: clear grid, pick new border pattern.
    pub fn reset(&mut self) {
        self.grid = vec![vec![EMPTY.to_string(); self.width]; self.height];
        use rand::Rng;
        self.border_pattern = BORDER_PATTERNS[rand::rng().random_range(0..BORDER_PATTERNS.len())].clone();
        self.raw_art_lines = None;
    }

    /// Execute an action with full animation and rendering. Returns true if action was Done.
    pub async fn execute_action(&mut self, action: &Action, header: &str, no_color: bool) -> Result<bool> {
        let w = self.width;
        let h = self.height;
        match action {
            Action::DrawBorder => {
                for x in 0..w {
                    self.draw_border_at(x, 0);
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(30)).await;
                }
                for y in 0..h {
                    self.draw_border_at(w - 1, y);
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(30)).await;
                }
                for x in (0..w).rev() {
                    self.draw_border_at(x, h - 1);
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(30)).await;
                }
                for y in (0..h).rev() {
                    self.draw_border_at(0, y);
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(30)).await;
                }
            }
            Action::PlaceRock { x, y, size } => {
                self.place_rock(*x, *y, *size);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Action::PlaceMoss { x, y } => {
                self.place_moss(*x, *y);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Action::PlaceFlower { x, y } => {
                self.place_flower(*x, *y);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Action::PlaceLantern { x, y } => {
                self.place_lantern(*x, *y);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Action::PlaceMandala { x, y, style } => {
                self.place_mandala(*x, *y, *style);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Action::PlaceAscii { x, y, glyph } => {
                self.place_ascii(*x, *y, glyph);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Action::DrawAsciiLine { y, x1, x2, glyph } => {
                let (a, b) = if x1 <= x2 { (*x1, *x2) } else { (*x2, *x1) };
                let step_range: Vec<usize> = if x1 <= x2 {
                    (a..=b.min(w.saturating_sub(1))).collect()
                } else {
                    (a..=b.min(w.saturating_sub(1))).rev().collect()
                };
                for x in step_range {
                    self.place_ascii(x, *y, glyph);
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(120)).await;
                }
            }
            Action::PlaceGlyph { x, y, glyph } => {
                self.place_glyph(*x, *y, glyph);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Action::DrawLine { y, x1, x2, glyph } => {
                let (a, b) = if x1 <= x2 { (*x1, *x2) } else { (*x2, *x1) };
                let step_range: Vec<usize> = if x1 <= x2 {
                    (a..=b.min(w.saturating_sub(1))).collect()
                } else {
                    (a..=b.min(w.saturating_sub(1))).rev().collect()
                };
                for x in step_range {
                    self.place_glyph(x, *y, glyph);
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(120)).await;
                }
            }
            Action::DrawRing { cx, cy, radius, glyph } => {
                let pts = self.ring_points(*cx, *cy, *radius);
                for (x, y) in pts {
                    self.place_glyph(x, y, glyph);
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
            Action::FillBox { x1, y1, x2, y2, glyph } => {
                let (min_x, max_x) = if x1 <= x2 { (*x1, *x2) } else { (*x2, *x1) };
                let (min_y, max_y) = if y1 <= y2 { (*y1, *y2) } else { (*y2, *y1) };
                for y in min_y..=max_y.min(h.saturating_sub(1)) {
                    for x in min_x..=max_x.min(w.saturating_sub(1)) {
                        self.place_glyph(x, y, glyph);
                        self.render_screen(header, no_color)?;
                        tokio::time::sleep(Duration::from_millis(60)).await;
                    }
                }
            }
            Action::ClearCell { x, y } => {
                self.clear_cell(*x, *y);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
            Action::RakeLine { y, x1, x2 } => {
                let (a, b) = if x1 <= x2 { (*x1, *x2) } else { (*x2, *x1) };
                let step_range: Vec<usize> = if x1 <= x2 {
                    (a..=b.min(w.saturating_sub(1))).collect()
                } else {
                    (a..=b.min(w.saturating_sub(1))).rev().collect()
                };
                for x in step_range {
                    if self.is_empty(x, *y) {
                        self.grid[*y][x] = RAKED.to_string();
                    }
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(120)).await;
                }
            }
            Action::RakeRing { cx, cy, radius } => {
                let pts = self.ring_points(*cx, *cy, *radius);
                for (x, y) in pts {
                    if self.is_empty(x, y) {
                        self.grid[y][x] = RAKED.to_string();
                    }
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
            Action::PlaceGravel { y, x1, x2 } => {
                let (a, b) = if x1 <= x2 { (*x1, *x2) } else { (*x2, *x1) };
                let step_range: Vec<usize> = if x1 <= x2 {
                    (a..=b.min(w.saturating_sub(1))).collect()
                } else {
                    (a..=b.min(w.saturating_sub(1))).rev().collect()
                };
                for x in step_range {
                    if self.is_empty(x, *y) {
                        self.grid[*y][x] = GRAVEL.to_string();
                    }
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(120)).await;
                }
            }
            Action::Done => {
                return Ok(true);
            }
            Action::PlaceMultiCellGlyph { anchor_x, anchor_y, glyphs } => {
                for (dx, dy, glyph) in glyphs {
                    let x = anchor_x.saturating_add(*dx);
                    let y = anchor_y.saturating_add(*dy);
                    self.place_glyph(x, y, glyph);
                }
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(60)).await;
            }
            Action::DrawFlowLine { points, glyph } => {
                for (x, y) in points {
                    self.place_glyph(*x, *y, glyph);
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(30)).await;
                }
            }
            Action::ApplyGlitchFilter { x, y, .. } => {
                let cell = self.grid[*y][*x].clone();
                let corrupted = format!("?{}", cell.chars().next().unwrap_or(' '));
                self.place_glyph(*x, *y, &corrupted);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(40)).await;
            }
            Action::PlaceBlendedGlyph { x, y, glyph, .. } => {
                self.place_glyph(*x, *y, glyph);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(40)).await;
            }
            Action::DisplayRawArt { lines } => {
                self.raw_art_lines = Some(lines.clone());
                self.render_screen(header, no_color)?;
            }
        }
        Ok(false)
    }
}