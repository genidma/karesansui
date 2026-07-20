/// Vec module: point and coordinate math for grid positioning.
/// Each point is a deliberate position in the grid; no implicit scaling or interpolation.

use std::ops::{Add, Sub};

/// A 2D point in grid space (x, y).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Point { x, y }
    }

    /// Manhattan distance between two points.
    pub fn manhattan_distance(&self, other: Point) -> usize {
        ((self.x as isize - other.x as isize).abs() + (self.y as isize - other.y as isize).abs())
            as usize
    }

    /// Euclidean distance (squared, to avoid floating point).
    pub fn distance_squared(&self, other: Point) -> usize {
        let dx = ((self.x as isize - other.x as isize).abs()) as usize;
        let dy = ((self.y as isize - other.y as isize).abs()) as usize;
        dx * dx + dy * dy
    }

    /// Generate a line of points between this and another using Bresenham's algorithm.
    pub fn line_to(&self, other: Point) -> Vec<Point> {
        let mut points = Vec::new();
        let (x0, y0) = (self.x as isize, self.y as isize);
        let (x1, y1) = (other.x as isize, other.y as isize);

        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = (dx as isize - dy as isize) / 2;

        let mut x = x0;
        let mut y = y0;

        loop {
            if x >= 0 && x < 10000 && y >= 0 && y < 10000 {
                points.push(Point::new(x as usize, y as usize));
            }
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 > -dy as isize {
                err -= dy as isize;
                x += sx;
            }
            if e2 < dx as isize {
                err += dx as isize;
                y += sy;
            }
        }
        points
    }

    /// Generate points around a circle at a given radius using Midpoint Circle Algorithm.
    pub fn circle_points(&self, radius: usize) -> Vec<Point> {
        let mut points = Vec::new();
        let center_x = self.x as isize;
        let center_y = self.y as isize;
        let r = radius as isize;

        let mut x = 0isize;
        let mut y = r;
        let mut d = 3 - 2 * r;

        while x <= y {
            // 8-way symmetry
            for (px, py) in [
                (center_x + x, center_y + y),
                (center_x - x, center_y + y),
                (center_x + x, center_y - y),
                (center_x - x, center_y - y),
                (center_x + y, center_y + x),
                (center_x - y, center_y + x),
                (center_x + y, center_y - x),
                (center_x - y, center_y - x),
            ] {
                if px >= 0 && py >= 0 {
                    points.push(Point::new(px as usize, py as usize));
                }
            }

            if d < 0 {
                d += 4 * x + 6;
            } else {
                d += 4 * (x - y) + 10;
                y -= 1;
            }
            x += 1;
        }

        // Deduplicate
        points.sort();
        points.dedup();
        points
    }

    /// Generate all points in a filled rectangle from this to another point.
    pub fn rect_filled(&self, other: Point) -> Vec<Point> {
        let mut points = Vec::new();
        let (min_x, max_x) = if self.x <= other.x { (self.x, other.x) } else { (other.x, self.x) };
        let (min_y, max_y) = if self.y <= other.y { (self.y, other.y) } else { (other.y, self.y) };

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                points.push(Point::new(x, y));
            }
        }
        points
    }

    /// Generate points forming the outline of a rectangle from this to another point.
    pub fn rect_outline(&self, other: Point) -> Vec<Point> {
        let mut points = Vec::new();
        let (min_x, max_x) = if self.x <= other.x { (self.x, other.x) } else { (other.x, self.x) };
        let (min_y, max_y) = if self.y <= other.y { (self.y, other.y) } else { (other.y, self.y) };

        for x in min_x..=max_x {
            points.push(Point::new(x, min_y));
            points.push(Point::new(x, max_y));
        }
        for y in min_y..=max_y {
            points.push(Point::new(min_x, y));
            points.push(Point::new(max_x, y));
        }
        points.sort();
        points.dedup();
        points
    }

    /// Clamp this point to fit within grid bounds.
    pub fn clamp(&self, width: usize, height: usize) -> Point {
        Point::new(self.x.min(width.saturating_sub(1)), self.y.min(height.saturating_sub(1)))
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Point {
        Point::new(
            (self.x as isize - other.x as isize).max(0) as usize,
            (self.y as isize - other.y as isize).max(0) as usize,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manhattan_distance() {
        let p1 = Point::new(0, 0);
        let p2 = Point::new(3, 4);
        assert_eq!(p1.manhattan_distance(p2), 7);
    }

    #[test]
    fn test_circle_points() {
        let center = Point::new(10, 10);
        let points = center.circle_points(3);
        assert!(!points.is_empty());
        // Check that points are roughly at the expected distance
        for &p in &points {
            let dist_sq = center.distance_squared(p);
            assert!(dist_sq >= 7 && dist_sq <= 11, "Point {:?} not on circle", p);
        }
    }
}
