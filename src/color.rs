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

    /// Convert to hex string (e.g., "#FF00FF").
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Perceptual distance between two colors in RGB space (Euclidean).
    pub fn distance(&self, other: Color) -> f64 {
        let dr = (self.r as f64 - other.r as f64).powi(2);
        let dg = (self.g as f64 - other.g as f64).powi(2);
        let db = (self.b as f64 - other.b as f64).powi(2);
        (dr + dg + db).sqrt()
    }

    /// Blend two colors with a factor (0.0 = self, 1.0 = other).
    pub fn blend(&self, other: Color, factor: f64) -> Color {
        let factor = factor.clamp(0.0, 1.0);
        let r = (self.r as f64 * (1.0 - factor) + other.r as f64 * factor) as u8;
        let g = (self.g as f64 * (1.0 - factor) + other.g as f64 * factor) as u8;
        let b = (self.b as f64 * (1.0 - factor) + other.b as f64 * factor) as u8;
        Color::new(r, g, b)
    }

    /// Find the closest color in a palette.
    pub fn nearest_in_palette(&self, palette: &[Color]) -> Color {
        palette
            .iter()
            .min_by(|a, b| {
                self.distance(**a)
                    .partial_cmp(&self.distance(**b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
            .unwrap_or(Color::new(0, 0, 0))
    }
}

/// A color palette: a set of deliberate, named colors for a theme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Palette {
    pub name: String,
    pub colors: Vec<Color>,
}

impl Palette {
    pub fn new(name: impl Into<String>) -> Self {
        Palette {
            name: name.into(),
            colors: vec![],
        }
    }

    pub fn with_colors(name: impl Into<String>, colors: Vec<Color>) -> Self {
        Palette {
            name: name.into(),
            colors,
        }
    }

    pub fn add_color(&mut self, color: Color) {
        self.colors.push(color);
    }

    pub fn get(&self, index: usize) -> Option<Color> {
        self.colors.get(index).copied()
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
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

    /// Gridwright's default palette: clean, mathematical, precise.
    pub fn gridwright_default() -> Palette {
        Palette::with_colors(
            "Gridwright",
            vec![
                Color::new(255, 255, 255), // Pure white (canvas)
                Color::new(200, 200, 200), // Light gray
                Color::new(128, 128, 128), // Medium gray
                Color::new(64, 64, 64),    // Dark gray
                Color::new(0, 0, 0),       // Black (lines, edges)
                Color::new(70, 130, 180),  // Steel blue (accent)
                Color::new(220, 20, 60),   // Crimson red (secondary accent)
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

    #[test]
    fn test_color_to_hex() {
        let c = Color::new(255, 0, 255);
        assert_eq!(c.to_hex(), "#FF00FF");
    }

    #[test]
    fn test_color_distance() {
        let c1 = Color::new(0, 0, 0);
        let c2 = Color::new(255, 0, 0);
        let d = c1.distance(c2);
        assert!((d - 255.0).abs() < 0.1);
    }

    #[test]
    fn test_nearest_in_palette() {
        let palette = vec![
            Color::new(0, 0, 0),
            Color::new(255, 0, 0),
            Color::new(0, 255, 0),
        ];
        let target = Color::new(250, 5, 0);
        let nearest = target.nearest_in_palette(&palette);
        assert_eq!(nearest, Color::new(255, 0, 0));
    }

    #[test]
    fn test_palette_operations() {
        let mut p = Palette::new("Test");
        assert_eq!(p.len(), 0);
        p.add_color(Color::new(255, 0, 0));
        assert_eq!(p.len(), 1);
        assert_eq!(p.get(0), Some(Color::new(255, 0, 0)));
    }
}
