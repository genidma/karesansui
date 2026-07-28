/// Color module: palette construction and per-cell color assignment for pixel art.
/// Manages RGB colors, palettes, and deliberate color mapping to grid cells.

use serde::{Deserialize, Serialize};

/// An RGB color triplet (0-255 per channel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    /// Parse color from hex string (e.g., "#FF00FF" or "FF00FF").
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color::new(r, g, b))
    }

}

/// A color palette: a set of deliberate, named colors for a theme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Palette {
    pub name: String,
    pub colors: Vec<Color>,
}

impl Palette {
    pub fn with_colors(name: impl Into<String>, colors: Vec<Color>) -> Self {
        Palette {
            name: name.into(),
            colors,
        }
    }

    pub fn get(&self, index: usize) -> Option<Color> {
        self.colors.get(index).copied()
    }
}

/// Pre-defined palettes for pixel art themes.
pub mod palettes {
    use super::*;

    /// Classic monochromatic (black and white) for ASCII-style art.
    pub fn monochrome() -> Palette {
        Palette::with_colors(
            "Monochrome",
            vec![
                Color::new(0, 0, 0),       // Black
                Color::new(255, 255, 255), // White
            ],
        )
    }

    /// Zen garden earth tones.
    pub fn zen_earth() -> Palette {
        Palette::with_colors(
            "Zen Earth",
            vec![
                Color::new(229, 218, 199), // Light sand
                Color::new(188, 172, 147), // Medium sand
                Color::new(140, 120, 93),  // Dark sand
                Color::new(101, 84, 63),   // Stone gray
                Color::new(60, 40, 20),    // Dark soil
            ],
        )
    }

    /// Cool, minimalist night palette.
    pub fn night_sky() -> Palette {
        Palette::with_colors(
            "Night Sky",
            vec![
                Color::new(15, 23, 42),   // Deep blue-black
                Color::new(30, 41, 59),   // Dark blue-gray
                Color::new(71, 85, 105),  // Slate
                Color::new(148, 163, 184), // Light silver
                Color::new(226, 232, 240), // Near-white
            ],
        )
    }

    /// Vibrant, energetic palette for expressive pixel art.
    pub fn vibrant_neon() -> Palette {
        Palette::with_colors(
            "Vibrant Neon",
            vec![
                Color::new(0, 0, 0),       // Black
                Color::new(255, 0, 127),  // Hot magenta
                Color::new(0, 255, 255),  // Cyan
                Color::new(255, 255, 0),  // Yellow
                Color::new(0, 255, 0),    // Lime
            ],
        )
    }

    /// Warm, earthy tones for organic shapes.
    pub fn warm_earth() -> Palette {
        Palette::with_colors(
            "Warm Earth",
            vec![
                Color::new(240, 217, 181), // Warm beige
                Color::new(184, 134, 11),  // Dark goldenrod
                Color::new(210, 105, 30),  // Chocolate
                Color::new(139, 69, 19),   // Saddle brown
                Color::new(101, 50, 25),   // Dark brown
            ],
        )
    }

    /// Gridwright's explicit 16x16 chunky-pixel palette.
    pub fn gridwright_spec() -> Palette {
        Palette::with_colors(
            "Gridwright",
            vec![
                Color::from_hex("#0b0c10").unwrap(), // Deep space
                Color::from_hex("#1f2833").unwrap(), // Slate blue
                Color::from_hex("#45a29e").unwrap(), // Teal
                Color::from_hex("#66fcf1").unwrap(), // Cyan glow
                Color::from_hex("#c5c6c7").unwrap(), // Light gray
                Color::from_hex("#f2a65a").unwrap(), // Warm orange
                Color::from_hex("#ef476f").unwrap(), // Rose red
                Color::from_hex("#ffffff").unwrap(), // White
            ],
        )
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        let c = Color::from_hex("#FF00FF").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 255);
    }

}
