//! Simple geometry primitives.

use crate::sprites::Drawable;

/// A tiny position vector.
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct Point {
    pub(crate) x: usize,
    pub(crate) y: usize,
}

/// A tiny rectangle based on two absolute `Point`s.
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct Rect {
    pub(crate) p1: Point,
    pub(crate) p2: Point,
}

impl Point {
    /// Create a new point.
    pub(crate) const fn new(x: usize, y: usize) -> Point {
        Point { x, y }
    }
}

impl core::ops::Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl core::ops::Mul for Point {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y)
    }
}

impl Rect {
    /// Create a rectangle from two `Point`s.
    pub(crate) fn new(p1: &Point, p2: &Point) -> Rect {
        let p1 = *p1;
        let p2 = *p2;

        Rect { p1, p2 }
    }

    /// Create a rectangle from a `Point` and a `Drawable`.
    pub(crate) fn from_drawable<D>(pos: &Point, drawable: &D) -> Rect
    where
        D: Drawable,
    {
        let p1 = *pos;
        let p2 = p1 + Point::new(drawable.width(), drawable.height());

        Rect { p1, p2 }
    }

    /// Test for intersections between two rectangles.
    ///
    /// Rectangles intersect when the geometry of either overlaps.
    pub(crate) fn intersects(&self, other: &Rect) -> bool {
        let (top1, right1, bottom1, left1) = self.get_bounds();
        let (top2, right2, bottom2, left2) = other.get_bounds();

        bottom1 > top2 && bottom2 > top1 && right1 > left2 && right2 > left1
    }

    /// Compute the bounding box for this rectangle.
    ///
    /// # Returns
    ///
    /// Tuple of `(top, right, bottom, left)`, e.g. in CSS clockwise order.
    fn get_bounds(&self) -> (usize, usize, usize, usize) {
        (self.p1.y, self.p2.x, self.p2.y, self.p1.x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_intersect() {
        let rect_size = Point::new(10, 10);
        let r1 = Rect::new(&rect_size, &(rect_size + rect_size));

        // Test intersection between equal-sized rectangles
        for y in 0..3 {
            for x in 0..3 {
                let x = x * 5 + 5;
                let y = y * 5 + 5;

                let r2 = Rect::new(&Point::new(x, y), &(Point::new(x, y) + rect_size));

                assert!(r1.intersects(&r2), "Should intersect");
                assert!(r2.intersects(&r1), "Should intersect");
            }
        }

        // Test non-intersections
        for y in 0..3 {
            for x in 0..3 {
                if x == 1 && y == 1 {
                    continue;
                }

                let x = x * 10;
                let y = y * 10;

                let r2 = Rect::new(&Point::new(x, y), &(Point::new(x, y) + rect_size));

                assert!(!r1.intersects(&r2), "Should not intersect");
                assert!(!r2.intersects(&r1), "Should not intersect");
            }
        }

        // Test intersection between different-sized rectangles
        let r2 = Rect::new(&Point::new(0, 0), &Point::new(30, 30));

        assert!(r1.intersects(&r2), "Should intersect");
        assert!(r2.intersects(&r1), "Should intersect");
    }
}
