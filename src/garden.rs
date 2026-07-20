use serde::{Deserialize, Serialize};

/// A single gardener action returned by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    /// Place a rock at (x, y). `size` 1-3 controls the glyph.
    PlaceRock { x: usize, y: usize, size: u8 },
    /// Rake a horizontal line of sand between two columns on a row.
    RakeLine { y: usize, x1: usize, x2: usize },
    /// Place a patch of moss at (x, y).
    PlaceMoss { x: usize, y: usize },
    /// Scatter gravel across a horizontal span on a row.
    PlaceGravel { y: usize, x1: usize, x2: usize },
    /// Place a cherry blossom accent at (x, y).
    PlaceFlower { x: usize, y: usize },
    /// Place a stone lantern at (x, y).
    PlaceLantern { x: usize, y: usize },
    /// Draw a border frame around the whole garden.
    DrawBorder,
    /// Signal that the garden is complete.
    Done,
}

/// Glyphs — each is exactly 2 terminal columns wide for alignment.
pub const EMPTY: &str = "  ";
pub const BORDER: &str = "🎋";
pub const RAKED: &str = "~~";
pub const ROCK_S: &str = "🪨";
pub const ROCK_M: &str = "🗿";
pub const ROCK_L: &str = "🗿";
pub const MOSS: &str = "🌿";
pub const GRAVEL: &str = "··";
pub const FLOWER: &str = "🌸";
pub const LANTERN: &str = "🏮";

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
}

impl Garden {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![vec![EMPTY.to_string(); width]; height];
        Self {
            width,
            height,
            grid,
            turtle_pos: Some((1, 1)),
            turtle_glyph: "🐢",
        }
    }

    pub fn is_empty(&self, x: usize, y: usize) -> bool {
        self.grid[y][x] == EMPTY
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

    #[allow(dead_code)]
    pub fn draw_border(&mut self) {
        for x in 0..self.width {
            self.grid[0][x] = BORDER.to_string();
            self.grid[self.height - 1][x] = BORDER.to_string();
        }
        for y in 0..self.height {
            self.grid[y][0] = BORDER.to_string();
            self.grid[y][self.width - 1] = BORDER.to_string();
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
