use serde::{Deserialize, Serialize};

/// A single gardener action returned by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    /// Place a rock at (x, y). `size` 1-3 controls the glyph.
    PlaceRock { x: usize, y: usize, size: u8 },
    /// Rake a horizontal line of sand between two columns on a row.
    RakeLine { y: usize, x1: usize, x2: usize },
    /// Draw a border frame around the whole garden.
    DrawBorder,
    /// Signal that the garden is complete.
    Done,
}

/// The ASCII zen garden grid.
pub struct Garden {
    pub width: usize,
    pub height: usize,
    grid: Vec<Vec<char>>,
}

impl Garden {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![vec![' '; width]; height];
        Self { width, height, grid }
    }

    pub fn place_rock(&mut self, x: usize, y: usize, size: u8) {
        if y >= self.height || x >= self.width {
            return;
        }
        let glyph = match size.clamp(1, 3) {
            1 => 'o',
            2 => 'O',
            _ => '@',
        };
        self.grid[y][x] = glyph;
    }

    pub fn rake_line(&mut self, y: usize, x1: usize, x2: usize) {
        if y >= self.height {
            return;
        }
        let (a, b) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        for x in a..=b.min(self.width.saturating_sub(1)) {
            // Don't overwrite rocks.
            if self.grid[y][x] == ' ' {
                self.grid[y][x] = '~';
            }
        }
    }

    pub fn draw_border(&mut self) {
        for x in 0..self.width {
            self.grid[0][x] = '#';
            self.grid[self.height - 1][x] = '#';
        }
        for y in 0..self.height {
            self.grid[y][0] = '#';
            self.grid[y][self.width - 1] = '#';
        }
    }

    /// Render the garden to a string for terminal display.
    pub fn render(&self) -> String {
        let mut out = String::with_capacity((self.width + 1) * self.height);
        for row in &self.grid {
            out.extend(row.iter());
            out.push('\n');
        }
        out
    }
}
