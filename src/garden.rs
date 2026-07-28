use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// A single gardener action returned by the LLM.
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
    /// Place a multi-cell glyph pattern (for enhanced Tabula Rasa and Matrix ASCII)
    PlaceMultiCellGlyph {
        anchor_x: usize,
        anchor_y: usize,
        glyphs: Vec<(usize, usize, String)>, // (dx, dy, glyph)
    },
    /// Draw a flow line with proportional spacing (for Enhanced Tabula Rasa)
    DrawFlowLine {
        points: Vec<(usize, usize)>,
        glyph: String,
    },
    /// Apply glitch escape sequences and corruption (for Glitch ASCII)
    ApplyGlitchFilter {
        x: usize,
        y: usize,
        filter_type: GlitchFilterType,
    },
    /// Place a glyph with custom blending (for Matrix ASCII)
    PlaceBlendedGlyph {
        x: usize,
        y: usize,
        glyph: String,
        blend_mode: BlendMode,
        opacity: f32,
    },
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

/// Various ASCII art theme modes for flexible glyph rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AsciiTheme {
    Classic,          // Standard 2-column ASCII
    TabulaRasa,       // Pure ASCII, no restrictions
    WildZones,        // Unicode freedom, no borders
    EnhancedTabulaRasa, // Enhanced ASCII with proportional placement
    ChaoticASCII,     // Variable width, overlapping, absolute freedom
    MatrixASCII,      // Overlapping, layered glyphs
    GlitchASCII,      // Unicode chaos, escape sequences
    Gridwright,       // Pixel-perfect grid art (existing)
}

impl AsciiTheme {
    pub fn max_glyph_width(&self) -> usize {
        match self {
            AsciiTheme::Classic => 2,
            AsciiTheme::TabulaRasa => 2,
            AsciiTheme::WildZones => 2,
            AsciiTheme::EnhancedTabulaRasa => 8,
            AsciiTheme::ChaoticASCII => 4,
            AsciiTheme::MatrixASCII => 4,
            AsciiTheme::GlitchASCII => 8,
            AsciiTheme::Gridwright => 1,
        }
    }

}

/// Blend modes for Matrix ASCII overlapping glyphs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BlendMode {
    Replace,     // Standard overwrite
    Add,         // Additive blending
    Multiply,    // Multiply effect
    Difference,  // Difference mode
    Screen,      // Screen effect
    Custom,      // Custom function
}

/// Glitch filter types for Glitch ASCII
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GlitchFilterType {
    CharSwap,      // Swap characters randomly
    WidthShift,    // Shift characters horizontally
    HeightShift,   // Shift characters vertically
    ColorInvert,   // Invert color if supported
    GlitchBurst,   // Random corruption bursts
    EscapeSequence,// Apply escape sequences
}

/// A glyph layer for overlapping ASCII art
#[derive(Debug, Clone, PartialEq)]
pub struct LayeredGlyph {
    pub content: String,
    pub z_index: i32,
    pub blend_mode: BlendMode,
    pub opacity: f32,
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
    pub name: &'static str,
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
    // 1. Classic Bamboo & Kadomatsu Grove
    BorderPattern {
        name: "Kadomatsu Bamboo Grove",
        top_left: "🎍", top_right: "🎍", bottom_left: "🎍", bottom_right: "🎍",
        top: "🎋", top_alt: "──", bottom: "🎋", bottom_alt: "──",
        left: "│ ", left_alt: "🎋", right: "│ ", right_alt: "🎋",
    },
    // 2. Sacred Double Box
    BorderPattern {
        name: "Sacred Double Box",
        top_left: "╔═", top_right: "═╗", bottom_left: "╚═", bottom_right: "═╝",
        top: "══", top_alt: "══", bottom: "══", bottom_alt: "══",
        left: "║ ", left_alt: "║ ", right: "║ ", right_alt: "║ ",
    },
    // 3. Mandala Diamond Lattice
    BorderPattern {
        name: "Mandala Diamond Lattice",
        top_left: "◈ ", top_right: "◈ ", bottom_left: "◈ ", bottom_right: "◈ ",
        top: "◇ ", top_alt: "◈ ", bottom: "◇ ", bottom_alt: "◈ ",
        left: "◇ ", left_alt: "◈ ", right: "◇ ", right_alt: "◈ ",
    },
    // 4. Seigaiha Ocean Waves
    BorderPattern {
        name: "Seigaiha Ocean Waves",
        top_left: "🌊", top_right: "🌊", bottom_left: "🌊", bottom_right: "🌊",
        top: "〰〰", top_alt: "≈≈", bottom: "〰〰", bottom_alt: "≈≈",
        left: "≈≈", left_alt: "〰 ", right: "≈≈", right_alt: "〰 ",
    },
    // 5. Stone Pillar & Gravel Shore
    BorderPattern {
        name: "Stone Pillar & Gravel Shore",
        top_left: "⛩️ ", top_right: "⛩️ ", bottom_left: "🗿", bottom_right: "🗿",
        top: "🪨", top_alt: "··", bottom: "🪨", bottom_alt: "··",
        left: "║ ", left_alt: "🪨", right: "║ ", right_alt: "🪨",
    },
    // 6. Starfield Lattice
    BorderPattern {
        name: "Starfield Lattice",
        top_left: "🌟", top_right: "🌟", bottom_left: "🌟", bottom_right: "🌟",
        top: "✦ ", top_alt: "✧ ", bottom: "✦ ", bottom_alt: "✧ ",
        left: "✦ ", left_alt: "✧ ", right: "✦ ", right_alt: "✧ ",
    },
    // 7. Enso Yin-Yang Harmony
    BorderPattern {
        name: "Enso Yin-Yang Harmony",
        top_left: "⭕", top_right: "⭕", bottom_left: "⭕", bottom_right: "⭕",
        top: "──", top_alt: "☯ ", bottom: "──", bottom_alt: "☯ ",
        left: "│ ", left_alt: "│ ", right: "│ ", right_alt: "│ ",
    },
    // 8. Sakura Blossom Garland
    BorderPattern {
        name: "Sakura Blossom Garland",
        top_left: "🌸", top_right: "🌸", bottom_left: "🌸", bottom_right: "🌸",
        top: "──", top_alt: "🌸", bottom: "──", bottom_alt: "🌸",
        left: "│ ", left_alt: "🌸", right: "│ ", right_alt: "🌸",
    },
    // 9. Engawa Wooden Deck
    BorderPattern {
        name: "Engawa Wooden Deck",
        top_left: "+-", top_right: "-+", bottom_left: "+-", bottom_right: "-+",
        top: "--", top_alt: "==", bottom: "--", bottom_alt: "==",
        left: "| ", left_alt: "| ", right: "| ", right_alt: "| ",
    },
    // 10. Zen Gravel Ridge
    BorderPattern {
        name: "Zen Gravel Ridge",
        top_left: "░░", top_right: "░░", bottom_left: "░░", bottom_right: "░░",
        top: "▒▒", top_alt: "░░", bottom: "▒▒", bottom_alt: "░░",
        left: "▒▒", left_alt: "░░", right: "▒▒", right_alt: "░░",
    },
    // 11. Minimalist Dotted Lattice
    BorderPattern {
        name: "Minimalist Dotted Lattice",
        top_left: "+-", top_right: "-+", bottom_left: "+-", bottom_right: "-+",
        top: "· ", top_alt: "- ", bottom: "· ", bottom_alt: "- ",
        left: ": ", left_alt: "| ", right: ": ", right_alt: "| ",
    },
    // 12. Shimenawa Sacred Rope
    BorderPattern {
        name: "Shimenawa Sacred Rope",
        top_left: "❖ ", top_right: "❖ ", bottom_left: "❖ ", bottom_right: "❖ ",
        top: "≈≈", top_alt: "──", bottom: "≈≈", bottom_alt: "──",
        left: "│ ", left_alt: "≈≈", right: "│ ", right_alt: "≈≈",
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GardenState {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<String>>,
    pub turtle_pos: Option<(usize, usize)>,
    pub turtle_glyph: String,
    pub border_pattern_index: usize,
    pub prompt_count: usize,
    pub theme_name: String,
}

/// The ASCII + emoji zen garden grid.
/// Each cell is a 2-column-wide string so emojis and ASCII mix cleanly.
pub struct Garden {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<String>>,
    /// Current position of the gardener turtle (x, y).
    pub turtle_pos: Option<(usize, usize)>,
    /// Glyph for the turtle (e.g. "🐢" when walking/building, "💤" when resting).
    pub turtle_glyph: &'static str,
    /// The aesthetic border pattern framing this session's garden.
    pub border_pattern: BorderPattern,
    pub border_pattern_index: usize,
    /// Current ASCII theme affecting rendering behavior
    pub ascii_theme: AsciiTheme,
    /// Layered glyph support for Matrix ASCII
    pub glyph_layers: HashMap<(usize, usize), Vec<LayeredGlyph>>,
}

impl Garden {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![vec![EMPTY.to_string(); width]; height];
        use rand::Rng;
        let mut rng = rand::rng();
        let idx = rng.random_range(0..BORDER_PATTERNS.len());
        let border_pattern = BORDER_PATTERNS[idx].clone();
        Self {
            width,
            height,
            grid,
            turtle_pos: Some((1, 1)),
            turtle_glyph: "🐢",
            border_pattern,
            border_pattern_index: idx,
            ascii_theme: AsciiTheme::Classic,
            glyph_layers: HashMap::new(),
        }
    }

    /// Place a glyph using theme-aware rendering
    pub fn place_glyph(&mut self, x: usize, y: usize, glyph: &str) {
        if y >= self.height || x >= self.width {
            return;
        }
        match self.ascii_theme {
            AsciiTheme::Classic | AsciiTheme::TabulaRasa | AsciiTheme::WildZones => {
                self.grid[y][x] = self.format_2col_glyph(glyph);
            }
            AsciiTheme::Gridwright => {
                // Gridwright uses its own canvas, not Garden
            }
            AsciiTheme::EnhancedTabulaRasa => {
                self.place_ascii_enhanced(x, y, glyph);
            }
            AsciiTheme::ChaoticASCII => {
                self.place_chaotic_glyph(x, y, glyph);
            }
            AsciiTheme::MatrixASCII => {
                let content = self.format_2col_glyph(glyph);
                let layer = LayeredGlyph {
                    content: content.clone(),
                    z_index: 0,
                    blend_mode: BlendMode::Replace,
                    opacity: 1.0,
                };
                self.glyph_layers.entry((x, y)).or_insert_with(Vec::new).push(layer);
                self.grid[y][x] = content;
            }
            AsciiTheme::GlitchASCII => {
                let corrupted = self.apply_glitch_filter(glyph);
                self.grid[y][x] = self.format_2col_glyph(&corrupted);
            }
        }
    }

    fn place_ascii_enhanced(&mut self, x: usize, y: usize, glyph: &str) {
        let mut clean = String::new();
        for ch in glyph.chars() {
            if ch.is_ascii() && ch != '\n' && ch != '\r' && !ch.is_control() {
                clean.push(ch);
            }
        }
        let max_width = self.ascii_theme.max_glyph_width();
        let display_width = clean.len().min(max_width);
        let mut display = String::new();
        for i in 0..display_width {
            display.push(clean.chars().nth(i).unwrap_or_default());
            if i < display_width - 1 {
                display.push(' ');
            }
        }
        self.grid[y][x] = display;
    }

    fn place_chaotic_glyph(&mut self, x: usize, y: usize, glyph: &str) {
        let mut display = String::new();
        for ch in glyph.chars() {
            if ch.is_ascii() || !ch.is_control() {
                display.push(ch);
            }
            if display.len() < self.ascii_theme.max_glyph_width() {
                display.push(' ');
            }
        }
        self.grid[y][x] = display;
    }

    fn apply_glitch_filter(&self, glyph: &str) -> String {
        let mut result = String::new();
        for ch in glyph.chars() {
            if ch.is_ascii() {
                let r: f32 = rand::random_range(0.0..1.0);
                if r < 0.1 {
                    result.push('?');
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }
        result
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

    /// Render the garden to a string for terminal display, showing the turtle
    /// right at its current location.
    pub fn render(&self) -> String {
        let mut out = String::new();
        for (y, row) in self.grid.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if let Some((tx, ty)) = self.turtle_pos {
                    if x == tx && y == ty {
                        out.push_str(self.turtle_glyph);
                        continue;
                    }
                }
                out.push_str(cell);
            }
            out.push('\n');
        }
        out
    }

    pub fn render_colored(&self, no_color: bool) -> String {
        if no_color {
            return self.render();
        }
        use crossterm::style::Stylize;
        let mut out = String::new();
        for (y, row) in self.grid.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if let Some((tx, ty)) = self.turtle_pos {
                    if x == tx && y == ty {
                        out.push_str(&self.turtle_glyph.yellow().bold().to_string());
                        continue;
                    }
                }
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

    pub fn save_to_file(&self, path: &str, prompt_count: usize, theme_name: &str) -> anyhow::Result<()> {
        let state = GardenState {
            width: self.width,
            height: self.height,
            grid: self.grid.clone(),
            turtle_pos: self.turtle_pos,
            turtle_glyph: self.turtle_glyph.to_string(),
            border_pattern_index: self.border_pattern_index,
            prompt_count,
            theme_name: theme_name.to_string(),
        };
        let json = serde_json::to_string_pretty(&state)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> anyhow::Result<(Self, usize, String)> {
        let content = std::fs::read_to_string(path)?;
        let state: GardenState = serde_json::from_str(&content)?;
        let border_pattern = BORDER_PATTERNS
            .get(state.border_pattern_index)
            .cloned()
            .unwrap_or_else(|| BORDER_PATTERNS[0].clone());
        let turtle_glyph = match state.turtle_glyph.as_str() {
            "💤" => "💤",
            "[*]" => "[*]",
            "[z]" => "[z]",
            _ => "🐢",
        };
        let garden = Self {
            width: state.width,
            height: state.height,
            grid: state.grid,
            turtle_pos: state.turtle_pos,
            turtle_glyph,
            border_pattern,
            border_pattern_index: state.border_pattern_index,
            ascii_theme: AsciiTheme::Classic,
            glyph_layers: HashMap::new(),
        };
        Ok((garden, state.prompt_count, state.theme_name))
    }

    /// Render the current garden state to screen with header.
    pub fn render_screen(&self, header: &str, no_color: bool) -> Result<()> {
        use crossterm::{cursor, terminal};
        use std::io::Write;
        let mut stdout = std::io::stdout();
        crossterm::queue!(stdout, cursor::Hide, cursor::MoveTo(0, 0))?;
        let full_text = if no_color {
            format!("{header}\n\n{}", self.render())
        } else {
            format!("{header}\n\n{}", self.render_colored(false))
        };
        for line in full_text.lines() {
            crossterm::queue!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine))?;
            writeln!(stdout, "{line}")?;
        }
        crossterm::queue!(stdout, terminal::Clear(terminal::ClearType::FromCursorDown))?;
        stdout.flush()?;
        Ok(())
    }

    /// Animate the turtle walking step-by-step to (dest_x, dest_y).
    pub async fn animate_walk(&mut self, dest_x: usize, dest_y: usize, header: &str, no_color: bool) -> Result<()> {
        let (mut tx, mut ty) = self.turtle_pos.unwrap_or((1, 1));
        while tx != dest_x || ty != dest_y {
            if tx < dest_x {
                tx += 1;
            } else if tx > dest_x {
                tx -= 1;
            }
            if ty < dest_y {
                ty += 1;
            } else if ty > dest_y {
                ty -= 1;
            }
            self.turtle_pos = Some((tx, ty));
            self.render_screen(header, no_color)?;
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        Ok(())
    }

    /// Execute an action with full animation and rendering. Returns true if action was Done.
    pub async fn execute_action(&mut self, action: &Action, header: &str, no_color: bool) -> Result<bool> {
        let w = self.width;
        let h = self.height;
        match action {
            Action::DrawBorder => {
                for x in 0..w {
                    self.draw_border_at(x, 0);
                    self.turtle_pos = Some((x, 0));
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(30)).await;
                }
                for y in 0..h {
                    self.draw_border_at(w - 1, y);
                    self.turtle_pos = Some((w - 1, y));
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(30)).await;
                }
                for x in (0..w).rev() {
                    self.draw_border_at(x, h - 1);
                    self.turtle_pos = Some((x, h - 1));
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(30)).await;
                }
                for y in (0..h).rev() {
                    self.draw_border_at(0, y);
                    self.turtle_pos = Some((0, y));
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(30)).await;
                }
                self.turtle_pos = Some((1, 1));
            }
            Action::PlaceRock { x, y, size } => {
                self.animate_walk(*x, *y, header, no_color).await?;
                self.place_rock(*x, *y, *size);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Action::PlaceMoss { x, y } => {
                self.animate_walk(*x, *y, header, no_color).await?;
                self.place_moss(*x, *y);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Action::PlaceFlower { x, y } => {
                self.animate_walk(*x, *y, header, no_color).await?;
                self.place_flower(*x, *y);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Action::PlaceLantern { x, y } => {
                self.animate_walk(*x, *y, header, no_color).await?;
                self.place_lantern(*x, *y);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Action::PlaceMandala { x, y, style } => {
                self.animate_walk(*x, *y, header, no_color).await?;
                self.place_mandala(*x, *y, *style);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Action::PlaceAscii { x, y, glyph } => {
                self.animate_walk(*x, *y, header, no_color).await?;
                self.place_ascii(*x, *y, glyph);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Action::DrawAsciiLine { y, x1, x2, glyph } => {
                self.animate_walk(*x1, *y, header, no_color).await?;
                let (a, b) = if x1 <= x2 { (*x1, *x2) } else { (*x2, *x1) };
                let step_range: Vec<usize> = if x1 <= x2 {
                    (a..=b.min(w.saturating_sub(1))).collect()
                } else {
                    (a..=b.min(w.saturating_sub(1))).rev().collect()
                };
                for x in step_range {
                    self.turtle_pos = Some((x, *y));
                    self.place_ascii(x, *y, glyph);
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(120)).await;
                }
            }
            Action::PlaceGlyph { x, y, glyph } => {
                self.animate_walk(*x, *y, header, no_color).await?;
                self.place_glyph(*x, *y, glyph);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Action::DrawLine { y, x1, x2, glyph } => {
                self.animate_walk(*x1, *y, header, no_color).await?;
                let (a, b) = if x1 <= x2 { (*x1, *x2) } else { (*x2, *x1) };
                let step_range: Vec<usize> = if x1 <= x2 {
                    (a..=b.min(w.saturating_sub(1))).collect()
                } else {
                    (a..=b.min(w.saturating_sub(1))).rev().collect()
                };
                for x in step_range {
                    self.turtle_pos = Some((x, *y));
                    self.place_glyph(x, *y, glyph);
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(120)).await;
                }
            }
            Action::DrawRing { cx, cy, radius, glyph } => {
                let pts = self.ring_points(*cx, *cy, *radius);
                if let Some(&(fx, fy)) = pts.first() {
                    self.animate_walk(fx, fy, header, no_color).await?;
                }
                for (x, y) in pts {
                    self.turtle_pos = Some((x, y));
                    self.place_glyph(x, y, glyph);
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
            Action::FillBox { x1, y1, x2, y2, glyph } => {
                let (min_x, max_x) = if x1 <= x2 { (*x1, *x2) } else { (*x2, *x1) };
                let (min_y, max_y) = if y1 <= y2 { (*y1, *y2) } else { (*y2, *y1) };
                self.animate_walk(min_x, min_y, header, no_color).await?;
                for y in min_y..=max_y.min(h.saturating_sub(1)) {
                    for x in min_x..=max_x.min(w.saturating_sub(1)) {
                        self.turtle_pos = Some((x, y));
                        self.place_glyph(x, y, glyph);
                        self.render_screen(header, no_color)?;
                        tokio::time::sleep(Duration::from_millis(60)).await;
                    }
                }
            }
            Action::ClearCell { x, y } => {
                self.animate_walk(*x, *y, header, no_color).await?;
                self.clear_cell(*x, *y);
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
            Action::RakeLine { y, x1, x2 } => {
                self.animate_walk(*x1, *y, header, no_color).await?;
                let (a, b) = if x1 <= x2 { (*x1, *x2) } else { (*x2, *x1) };
                let step_range: Vec<usize> = if x1 <= x2 {
                    (a..=b.min(w.saturating_sub(1))).collect()
                } else {
                    (a..=b.min(w.saturating_sub(1))).rev().collect()
                };
                for x in step_range {
                    self.turtle_pos = Some((x, *y));
                    if self.is_empty(x, *y) {
                        self.grid[*y][x] = RAKED.to_string();
                    }
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(120)).await;
                }
            }
            Action::RakeRing { cx, cy, radius } => {
                let pts = self.ring_points(*cx, *cy, *radius);
                if let Some(first) = pts.first() {
                    self.animate_walk(first.0, first.1, header, no_color).await?;
                }
                for (x, y) in pts {
                    self.turtle_pos = Some((x, y));
                    if self.is_empty(x, y) {
                        self.grid[y][x] = RAKED.to_string();
                    }
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
            Action::PlaceGravel { y, x1, x2 } => {
                self.animate_walk(*x1, *y, header, no_color).await?;
                let (a, b) = if x1 <= x2 { (*x1, *x2) } else { (*x2, *x1) };
                let step_range: Vec<usize> = if x1 <= x2 {
                    (a..=b.min(w.saturating_sub(1))).collect()
                } else {
                    (a..=b.min(w.saturating_sub(1))).rev().collect()
                };
                for x in step_range {
                    self.turtle_pos = Some((x, *y));
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
                self.turtle_pos = Some((*anchor_x, *anchor_y));
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(60)).await;
            }
            Action::DrawFlowLine { points, glyph } => {
                for (x, y) in points {
                    self.place_glyph(*x, *y, glyph);
                    self.turtle_pos = Some((*x, *y));
                    self.render_screen(header, no_color)?;
                    tokio::time::sleep(Duration::from_millis(30)).await;
                }
            }
            Action::ApplyGlitchFilter { x, y, .. } => {
                let cell = self.grid[*y][*x].clone();
                let corrupted = format!("?{}", cell.chars().next().unwrap_or(' '));
                self.place_glyph(*x, *y, &corrupted);
                self.turtle_pos = Some((*x, *y));
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(40)).await;
            }
            Action::PlaceBlendedGlyph { x, y, glyph, .. } => {
                self.place_glyph(*x, *y, glyph);
                self.turtle_pos = Some((*x, *y));
                self.render_screen(header, no_color)?;
                tokio::time::sleep(Duration::from_millis(40)).await;
            }
        }
        Ok(false)
    }
}
