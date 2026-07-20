/// Canvas module: explicit grid rendering where each cell maps to exactly one pixel.
/// No auto-scaling, no smoothing, no interpolation—pure deliberate placement.

use crate::color::Color;
use crate::vec::Point;
use serde::{Deserialize, Serialize};

/// A single pixel in the canvas: a point with a character/glyph and optional color.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pixel {
    pub x: usize,
    pub y: usize,
    pub glyph: String, // 1-2 chars for terminal width
    pub color: Option<Color>,
}

impl Pixel {
    pub fn new(x: usize, y: usize, glyph: impl Into<String>) -> Self {
        Pixel {
            x,
            y,
            glyph: glyph.into(),
            color: None,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// A canvas: a 2D grid of pixels for rendering pixel art.
/// Each cell is explicitly placed; no automatic scaling or interpolation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub palette: Option<String>, // Name of the active palette
    pub pixels: Vec<Vec<Pixel>>,
}

impl Canvas {
    /// Create a new empty canvas.
    pub fn new(width: usize, height: usize) -> Self {
        let empty_pixel = Pixel::new(0, 0, "  ");
        let pixels = vec![vec![empty_pixel; width]; height];
        Canvas {
            width,
            height,
            palette: None,
            pixels,
        }
    }

    /// Set the active palette by name.
    pub fn set_palette(&mut self, palette_name: &str) {
        self.palette = Some(palette_name.to_string());
    }

    /// Place a pixel at (x, y).
    pub fn set_pixel(&mut self, point: Point, glyph: impl Into<String>) {
        if point.x < self.width && point.y < self.height {
            self.pixels[point.y][point.x] = Pixel::new(point.x, point.y, glyph);
        }
    }

    /// Place a pixel with a specific color.
    pub fn set_pixel_colored(&mut self, point: Point, glyph: impl Into<String>, color: Color) {
        if point.x < self.width && point.y < self.height {
            self.pixels[point.y][point.x] = Pixel::new(point.x, point.y, glyph).with_color(color);
        }
    }

    /// Get the pixel at a point.
    pub fn get_pixel(&self, point: Point) -> Option<&Pixel> {
        if point.x < self.width && point.y < self.height {
            Some(&self.pixels[point.y][point.x])
        } else {
            None
        }
    }

    /// Draw a line of pixels from one point to another.
    pub fn draw_line(&mut self, from: Point, to: Point, glyph: impl Into<String>) {
        let glyph_str = glyph.into();
        for point in from.line_to(to) {
            if point.x < self.width && point.y < self.height {
                self.set_pixel(point, glyph_str.clone());
            }
        }
    }

    /// Draw a line of pixels with a specific color from one point to another.
    pub fn draw_line_colored(&mut self, from: Point, to: Point, glyph: impl Into<String>, color: Color) {
        let glyph_str = glyph.into();
        for point in from.line_to(to) {
            if point.x < self.width && point.y < self.height {
                self.set_pixel_colored(point, glyph_str.clone(), color);
            }
        }
    }

    /// Draw a circle outline at a given radius.
    pub fn draw_circle(&mut self, center: Point, radius: usize, glyph: impl Into<String>) {
        let glyph_str = glyph.into();
        for point in center.circle_points(radius) {
            if point.x < self.width && point.y < self.height {
                self.set_pixel(point, glyph_str.clone());
            }
        }
    }

    /// Draw a circle outline with a specific color at a given radius.
    pub fn draw_circle_colored(&mut self, center: Point, radius: usize, glyph: impl Into<String>, color: Color) {
        let glyph_str = glyph.into();
        for point in center.circle_points(radius) {
            if point.x < self.width && point.y < self.height {
                self.set_pixel_colored(point, glyph_str.clone(), color);
            }
        }
    }

    /// Fill a rectangle with a glyph.
    pub fn fill_rectangle(&mut self, from: Point, to: Point, glyph: impl Into<String>) {
        let glyph_str = glyph.into();
        for point in from.rect_filled(to) {
            if point.x < self.width && point.y < self.height {
                self.set_pixel(point, glyph_str.clone());
            }
        }
    }

    /// Fill a rectangle with a specific color.
    pub fn fill_rectangle_colored(
        &mut self,
        from: Point,
        to: Point,
        glyph: impl Into<String>,
        color: Color,
    ) {
        let glyph_str = glyph.into();
        for point in from.rect_filled(to) {
            if point.x < self.width && point.y < self.height {
                self.set_pixel_colored(point, glyph_str.clone(), color);
            }
        }
    }

    /// Draw a rectangle outline from one point to another.
    pub fn draw_rectangle(&mut self, from: Point, to: Point, glyph: impl Into<String>) {
        let glyph_str = glyph.into();
        for point in from.rect_outline(to) {
            if point.x < self.width && point.y < self.height {
                self.set_pixel(point, glyph_str.clone());
            }
        }
    }

    /// Draw a rectangle outline with a specific color from one point to another.
    pub fn draw_rectangle_colored(
        &mut self,
        from: Point,
        to: Point,
        glyph: impl Into<String>,
        color: Color,
    ) {
        let glyph_str = glyph.into();
        for point in from.rect_outline(to) {
            if point.x < self.width && point.y < self.height {
                self.set_pixel_colored(point, glyph_str.clone(), color);
            }
        }
    }

    /// Draw a filled circle.
    pub fn fill_circle(&mut self, center: Point, radius: usize, glyph: impl Into<String>) {
        let glyph_str = glyph.into();
        for y in 0..self.height {
            for x in 0..self.width {
                let point = Point::new(x, y);
                let dist_sq = center.distance_squared(point);
                if dist_sq <= radius * radius {
                    self.set_pixel(point, glyph_str.clone());
                }
            }
        }
    }

    /// Draw a filled circle with a specific color.
    pub fn fill_circle_colored(&mut self, center: Point, radius: usize, glyph: impl Into<String>, color: Color) {
        let glyph_str = glyph.into();
        for y in 0..self.height {
            for x in 0..self.width {
                let point = Point::new(x, y);
                let dist_sq = center.distance_squared(point);
                if dist_sq <= radius * radius {
                    self.set_pixel_colored(point, glyph_str.clone(), color);
                }
            }
        }
    }

    /// Clear the entire canvas (fill with spaces).
    pub fn clear(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.pixels[y][x] = Pixel::new(x, y, "  ");
            }
        }
    }

    /// Render the canvas to a string for terminal display.
    pub fn render(&self) -> String {
        let mut output = String::new();
        for row in &self.pixels {
            for pixel in row {
                output.push_str(&pixel.glyph);
            }
            output.push('\n');
        }
        output
    }

    /// Render with color escape codes (ANSI 24-bit RGB).
    pub fn render_with_colors(&self) -> String {
        let mut output = String::new();
        for row in &self.pixels {
            for pixel in row {
                if let Some(color) = pixel.color {
                    output.push_str(&format!(
                        "\x1b[38;2;{};{};{}m{}\x1b[0m",
                        color.r, color.g, color.b, pixel.glyph
                    ));
                } else {
                    output.push_str(&pixel.glyph);
                }
            }
            output.push('\n');
        }
        output
    }

    /// Get a serializable representation of the canvas state.
    pub fn to_state(&self) -> CanvasState {
        CanvasState {
            width: self.width,
            height: self.height,
            palette: self.palette.clone(),
            pixels: self.pixels.clone(),
        }
    }
}

/// Serializable state of a canvas for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasState {
    pub width: usize,
    pub height: usize,
    pub palette: Option<String>,
    pub pixels: Vec<Vec<Pixel>>,
}

impl CanvasState {
    pub fn to_canvas(&self) -> Canvas {
        Canvas {
            width: self.width,
            height: self.height,
            palette: self.palette.clone(),
            pixels: self.pixels.clone(),
        }
    }
}

/// Builder for composing canvas layouts with deliberate structure.
pub struct CanvasBuilder {
    canvas: Canvas,
}

impl CanvasBuilder {
    pub fn new(width: usize, height: usize) -> Self {
        CanvasBuilder {
            canvas: Canvas::new(width, height),
        }
    }

    pub fn with_palette(mut self, palette_name: &str) -> Self {
        self.canvas.set_palette(palette_name);
        self
    }

    pub fn draw_line(mut self, from: Point, to: Point, glyph: impl Into<String>) -> Self {
        self.canvas.draw_line(from, to, glyph);
        self
    }

    pub fn draw_line_colored(mut self, from: Point, to: Point, glyph: impl Into<String>, color: Color) -> Self {
        self.canvas.draw_line_colored(from, to, glyph, color);
        self
    }

    pub fn draw_circle(mut self, center: Point, radius: usize, glyph: impl Into<String>) -> Self {
        self.canvas.draw_circle(center, radius, glyph);
        self
    }

    pub fn draw_circle_colored(mut self, center: Point, radius: usize, glyph: impl Into<String>, color: Color) -> Self {
        self.canvas.draw_circle_colored(center, radius, glyph, color);
        self
    }

    pub fn fill_rectangle(
        mut self,
        from: Point,
        to: Point,
        glyph: impl Into<String>,
    ) -> Self {
        self.canvas.fill_rectangle(from, to, glyph);
        self
    }

    pub fn fill_rectangle_colored(
        mut self,
        from: Point,
        to: Point,
        glyph: impl Into<String>,
        color: Color,
    ) -> Self {
        self.canvas.fill_rectangle_colored(from, to, glyph, color);
        self
    }

    pub fn draw_rectangle(mut self, from: Point, to: Point, glyph: impl Into<String>) -> Self {
        self.canvas.draw_rectangle(from, to, glyph);
        self
    }

    pub fn draw_rectangle_colored(mut self, from: Point, to: Point, glyph: impl Into<String>, color: Color) -> Self {
        self.canvas.draw_rectangle_colored(from, to, glyph, color);
        self
    }

    pub fn fill_circle(mut self, center: Point, radius: usize, glyph: impl Into<String>) -> Self {
        self.canvas.fill_circle(center, radius, glyph);
        self
    }

    pub fn fill_circle_colored(mut self, center: Point, radius: usize, glyph: impl Into<String>, color: Color) -> Self {
        self.canvas.fill_circle_colored(center, radius, glyph, color);
        self
    }

    pub fn set_pixel(mut self, point: Point, glyph: impl Into<String>) -> Self {
        self.canvas.set_pixel(point, glyph);
        self
    }

    pub fn set_pixel_colored(mut self, point: Point, glyph: impl Into<String>, color: Color) -> Self {
        self.canvas.set_pixel_colored(point, glyph, color);
        self
    }

    pub fn build(self) -> Canvas {
        self.canvas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_creation() {
        let canvas = Canvas::new(10, 10);
        assert_eq!(canvas.width, 10);
        assert_eq!(canvas.height, 10);
        assert_eq!(canvas.pixels.len(), 10);
    }

    #[test]
    fn test_set_pixel() {
        let mut canvas = Canvas::new(10, 10);
        let point = Point::new(5, 5);
        canvas.set_pixel(point, "#");
        assert_eq!(canvas.get_pixel(point).unwrap().glyph, "#");
    }

    #[test]
    fn test_draw_line() {
        let mut canvas = Canvas::new(20, 20);
        canvas.draw_line(Point::new(0, 0), Point::new(10, 0), "─");
        for x in 0..=10 {
            let pixel = canvas.get_pixel(Point::new(x, 0)).unwrap();
            assert_eq!(pixel.glyph, "─");
        }
    }

    #[test]
    fn test_canvas_builder() {
        let canvas = CanvasBuilder::new(10, 10)
            .draw_line(Point::new(0, 0), Point::new(5, 5), "•")
            .set_pixel(Point::new(5, 5), "●")
            .build();

        assert_eq!(canvas.get_pixel(Point::new(5, 5)).unwrap().glyph, "●");
    }

    #[test]
    fn test_render() {
        let mut canvas = Canvas::new(3, 3);
        canvas.set_pixel(Point::new(1, 1), "X");
        let rendered = canvas.render();
        assert!(rendered.contains("X"));
    }
}
