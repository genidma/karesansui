use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};

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
    /// Scatter gravel across a horizontal span on a row.
    PlaceGravel { y: usize, x1: usize, x2: usize },
    /// Place a cherry blossom accent at (x, y).
    PlaceFlower { x: usize, y: usize },
    /// Place a stone lantern at (x, y).
    PlaceLantern { x: usize, y: usize },
    /// Place a geometric minimalist mandala or fractal accent at (x, y). `style` 1-6 controls the glyph.
    PlaceMandala { x: usize, y: usize, style: u8 },
    /// Place arbitrary ASCII glyph(s) at (x, y) for Tabula Rasa freeform art.
    PlaceAscii { x: usize, y: usize, glyph: String },
    /// Draw a horizontal ASCII stroke or line from x1 to x2 on row y using arbitrary ASCII characters.
    DrawAsciiLine { y: usize, x1: usize, x2: usize, glyph: String },
    /// Place ANY universal character(s) — single emoji, symbol, or 1-2 ASCII characters at (x, y) for Wild Zones liberation.
    PlaceGlyph { x: usize, y: usize, glyph: String },
    /// Draw a horizontal stroke of any glyph from x1 to x2 on row y.
    DrawLine { y: usize, x1: usize, x2: usize, glyph: String },
    /// Draw a circular ring of any glyph centered at (cx, cy) with given radius.
    DrawRing { cx: usize, cy: usize, radius: usize, glyph: String },
    /// Fill a rectangular box from (x1, y1) to (x2, y2) with any glyph.
    FillBox { x1: usize, y1: usize, x2: usize, y2: usize, glyph: String },
    /// Clear or reset a specific cell back to empty space (`  `).
    ClearCell { x: usize, y: usize },
    /// Draw a border frame around the whole garden.
    DrawBorder,
    /// Signal that the garden is complete.
    Done,
}

/// Glyphs — each is exactly 2 terminal columns wide for alignment.
pub const EMPTY: &str = "  ";
#[allow(dead_code)]
pub const BORDER: &str = "🎋";
pub const RAKED: &str = "~~";
pub const ROCK_S: &str = "🪨";
pub const ROCK_M: &str = "🗿";
pub const ROCK_L: &str = "🗿";
pub const MOSS: &str = "🌿";
pub const GRAVEL: &str = "··";
pub const FLOWER: &str = "🌸";
pub const LANTERN: &str = "🏮";

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
}

impl Garden {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![vec![EMPTY.to_string(); width]; height];
        let mut rng = rand::rng();
        let border_pattern = BORDER_PATTERNS.choose(&mut rng).unwrap().clone();
        Self {
            width,
            height,
            grid,
            turtle_pos: Some((1, 1)),
            turtle_glyph: "🐢",
            border_pattern,
        }
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

    /// Helper to compute grid coordinates around a circle circumference.
    pub fn ring_points(&self, cx: usize, cy: usize, radius: usize) -> Vec<(usize, usize)> {
        let mut points = Vec::new();
        let r = radius as f64;
        // Step around 360 degrees
        for deg in (0..360).step_by(10) {
            let rad = (deg as f64).to_radians();
            let dx = (r * rad.cos()).round() as isize;
            let dy = (r * rad.sin()).round() as isize;
            let px = cx as isize + dx;
            let py = cy as isize + dy;
            if px >= 1 && px < (self.width.saturating_sub(1)) as isize && py >= 1 && py < (self.height.saturating_sub(1)) as isize {
                let pt = (px as usize, py as usize);
                if !points.contains(&pt) {
                    points.push(pt);
                }
            }
        }
        points
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
            // Non-ASCII emoji or wide symbol naturally occupies 2 terminal columns.
            first.to_string()
        }
    }

    pub fn place_glyph(&mut self, x: usize, y: usize, glyph: &str) {
        if y >= self.height || x >= self.width {
            return;
        }
        self.grid[y][x] = self.format_2col_glyph(glyph);
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
}
