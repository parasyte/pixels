//! Simple geometry primitives.

use crate::sprites::Drawable;
use arrayvec::ArrayVec;

/// A tiny absolute position vector.
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct Point {
    pub(crate) x: usize,
    pub(crate) y: usize,
}

/// A tiny absolute rectangle based on two `Point`s.
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct Rect {
    pub(crate) p1: Point,
    pub(crate) p2: Point,
}

/// A tiny 2D vector with floating point coordinates.
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct Vec2D {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

/// A tiny 2D line segment based on `Vec2D`s.
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct LineSegment {
    pub(crate) p: Vec2D,
    pub(crate) q: Vec2D,
}

impl Point {
    /// Create a new point.
    pub(crate) const fn new(x: usize, y: usize) -> Point {
        Point { x, y }
    }
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::Mul for Point {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y)
    }
}

/// Saturates to 0.0
impl From<Vec2D> for Point {
    fn from(v: Vec2D) -> Point {
        Point::new(v.x.round().max(0.0) as usize, v.y.round().max(0.0) as usize)
    }
}

impl Rect {
    /// Create a rectangle from two `Point`s.
    pub(crate) fn new(p1: Point, p2: Point) -> Rect {
        Rect { p1, p2 }
    }

    /// Create a rectangle from a `Point` and a `Drawable`.
    pub(crate) fn from_drawable<D>(p1: Point, drawable: &D) -> Rect
    where
        D: Drawable,
    {
        let p2 = p1 + Point::new(drawable.width(), drawable.height());

        Rect { p1, p2 }
    }

    /// Test for intersections between two rectangles.
    ///
    /// Rectangles intersect when the geometry of either overlaps.
    pub(crate) fn intersects(&self, other: Rect) -> bool {
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

impl Vec2D {
    /// Create a 2D vector.
    pub(crate) fn new(x: f32, y: f32) -> Vec2D {
        Vec2D { x, y }
    }

    /// Compute the squared length.
    pub(crate) fn len_sq(&self) -> f32 {
        self.x.powi(2) + self.y.powi(2)
    }

    /// Compute the length.
    pub(crate) fn len(&self) -> f32 {
        self.len_sq().sqrt()
    }

    /// Scale by a scalar.
    pub(crate) fn scale(&mut self, scale: f32) {
        self.x *= scale;
        self.y *= scale;
    }

    /// Normalize to a unit vector.
    ///
    /// # Panics
    ///
    /// Asserts that length of `self != 0.0`
    pub(crate) fn normalize(&mut self) {
        let l = self.len();
        assert!(l.abs() > std::f32::EPSILON);

        self.x /= l;
        self.y /= l;
    }
}

impl std::ops::Add for Vec2D {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::AddAssign for Vec2D {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl std::ops::Sub for Vec2D {
    type Output = Vec2D;

    fn sub(self, other: Vec2D) -> Vec2D {
        Vec2D::new(self.x - other.x, self.y - other.y)
    }
}

impl std::ops::SubAssign for Vec2D {
    fn sub_assign(&mut self, other: Vec2D) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl std::ops::Mul for Vec2D {
    type Output = Vec2D;

    fn mul(self, other: Vec2D) -> Vec2D {
        Vec2D::new(self.x * other.x, self.y * other.y)
    }
}

impl std::ops::MulAssign for Vec2D {
    fn mul_assign(&mut self, other: Vec2D) {
        self.x *= other.x;
        self.y *= other.y;
    }
}

impl From<Point> for Vec2D {
    fn from(p: Point) -> Vec2D {
        Vec2D::new(p.x as f32, p.y as f32)
    }
}

impl LineSegment {
    /// Create a new `LineSegment` from two `Vec2D`s.
    pub(crate) fn new(p: Vec2D, q: Vec2D) -> LineSegment {
        LineSegment { p, q }
    }

    /// Cross product between `self` and `other`.
    pub(crate) fn cross(&self, other: LineSegment) -> f32 {
        let v = self.q - self.p;
        let w = other.q - other.p;

        (v.x * w.y) - (v.y * w.x)
    }
}

/// Find the convex hull around some set of `vertices` using the Jarvis march, aka gift wrapping
/// algorithm.
///
/// The first item in the list must be on the convex hull.
///
/// # Panics
///
/// This function will panic if `vertices.len() < 2` or if more than 16 vertices are in the convex
/// hull.
pub(crate) fn convex_hull(vertices: &[Vec2D]) -> ArrayVec<[Vec2D; 16]> {
    assert!(vertices.len() >= 2);
    let mut a = (0, vertices[0]);
    let mut b = (1, vertices[1]);

    let mut output = ArrayVec::new();
    output.push(a.1);

    loop {
        for c in vertices.iter().enumerate() {
            // Recompute the `ab` line on each iteration, since `b` may be updated.
            let ab = LineSegment::new(a.1, b.1);
            let ac = LineSegment::new(a.1, *c.1);

            // The sign of the cross product tells us which side of the `a, b` line that the `a, c`
            // line will fall; Negative to the left, positive to the right.
            let cross = ab.cross(ac);

            // To handle colinear points, we compare vector lengths; longest wins.
            let ab_len = (b.1 - a.1).len_sq();
            let ac_len = (*c.1 - a.1).len_sq();

            // Record the left-most pointing vector from point `a` or the longest vector when
            // comparing the angle between colinear points.
            if cross < 0.0 || (cross.abs() <= std::f32::EPSILON && ac_len > ab_len) {
                b = (c.0, *c.1);
            }
        }

        // When we find the first vertex in the set, we have completed the convex hull.
        if b.0 == 0 {
            return output;
        } else if output.is_full() {
            panic!("Too many vertices in the convex hull.");
        }

        // Update `a` and push the next vertex
        a = b;
        output.push(a.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_intersect() {
        let rect_size = Point::new(10, 10);
        let r1 = Rect::new(rect_size, rect_size + rect_size);

        // Test intersection between equal-sized rectangles
        for y in 0..3 {
            for x in 0..3 {
                let x = x * 5 + 5;
                let y = y * 5 + 5;

                let r2 = Rect::new(Point::new(x, y), Point::new(x, y) + rect_size);

                assert!(r1.intersects(r2), "Should intersect");
                assert!(r2.intersects(r1), "Should intersect");
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

                let r2 = Rect::new(Point::new(x, y), Point::new(x, y) + rect_size);

                assert!(!r1.intersects(r2), "Should not intersect");
                assert!(!r2.intersects(r1), "Should not intersect");
            }
        }

        // Test intersection between different-sized rectangles
        let r2 = Rect::new(Point::new(0, 0), Point::new(30, 30));

        assert!(r1.intersects(r2), "Should intersect");
        assert!(r2.intersects(r1), "Should intersect");
    }

    #[test]
    fn vector2d_point_conversion() {
        let v = Vec2D::new(-2.0, 4.0);
        let p = Point::new(10, 10);

        // Point + Vec2D
        let t = Point::from(Vec2D::from(p) + v);
        assert!(t.x == 8);
        assert!(t.y == 14);

        // Point - Vec2D
        let t = Point::from(Vec2D::from(p) - v);
        assert!(t.x == 12);
        assert!(t.y == 6);

        // Point * Vec2D
        let t = Point::from(Vec2D::from(p) * v);
        assert!(t.x == 0);
        assert!(t.y == 40);
    }

    #[test]
    fn convex_hull_clockwise() {
        let actual = convex_hull(&[
            Vec2D::new(0.0, 0.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(1.0, 1.0),
            Vec2D::new(0.0, 1.0),
            Vec2D::new(0.5, 0.5),
        ]);
        let expected = vec![
            Vec2D::new(0.0, 0.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(1.0, 1.0),
            Vec2D::new(0.0, 1.0),
        ];

        assert_eq!(actual.len(), expected.len());
        for (&expected, &actual) in expected.iter().zip(actual.iter()) {
            assert!((actual.x - expected.x).abs() <= std::f32::EPSILON);
            assert!((actual.y - expected.y).abs() <= std::f32::EPSILON);
        }
    }

    #[test]
    fn convex_hull_counter_clockwise() {
        let actual = convex_hull(&[
            Vec2D::new(0.0, 0.0),
            Vec2D::new(0.0, 1.0),
            Vec2D::new(1.0, 1.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(0.5, 0.5),
        ]);
        let expected = vec![
            Vec2D::new(0.0, 0.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(1.0, 1.0),
            Vec2D::new(0.0, 1.0),
        ];

        assert_eq!(actual.len(), expected.len());
        for (&expected, &actual) in expected.iter().zip(actual.iter()) {
            assert!((actual.x - expected.x).abs() <= std::f32::EPSILON);
            assert!((actual.y - expected.y).abs() <= std::f32::EPSILON);
        }
    }

    #[test]
    fn convex_hull_unsorted() {
        let actual = convex_hull(&[
            Vec2D::new(0.0, 0.0),
            Vec2D::new(0.5, 0.5),
            Vec2D::new(1.0, 1.0),
            Vec2D::new(0.0, 1.0),
            Vec2D::new(1.0, 0.0),
        ]);
        let expected = vec![
            Vec2D::new(0.0, 0.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(1.0, 1.0),
            Vec2D::new(0.0, 1.0),
        ];

        assert_eq!(actual.len(), expected.len());
        for (&expected, &actual) in expected.iter().zip(actual.iter()) {
            assert!((actual.x - expected.x).abs() <= std::f32::EPSILON);
            assert!((actual.y - expected.y).abs() <= std::f32::EPSILON);
        }
    }

    #[test]
    fn convex_hull_colinear_clockwise() {
        let actual = convex_hull(&[
            Vec2D::new(0.0, 0.0),
            Vec2D::new(0.5, 0.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(1.0, 0.5),
            Vec2D::new(1.0, 1.0),
            Vec2D::new(0.5, 1.0),
            Vec2D::new(0.0, 1.0),
            Vec2D::new(0.0, 0.5),
            Vec2D::new(0.5, 0.5),
        ]);
        let expected = vec![
            Vec2D::new(0.0, 0.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(1.0, 1.0),
            Vec2D::new(0.0, 1.0),
        ];

        assert_eq!(actual.len(), expected.len());
        for (&expected, &actual) in expected.iter().zip(actual.iter()) {
            assert!((actual.x - expected.x).abs() <= std::f32::EPSILON);
            assert!((actual.y - expected.y).abs() <= std::f32::EPSILON);
        }
    }

    #[test]
    fn convex_hull_colinear_counter_clockwise() {
        let actual = convex_hull(&[
            Vec2D::new(0.0, 0.0),
            Vec2D::new(0.0, 0.5),
            Vec2D::new(0.0, 1.0),
            Vec2D::new(0.5, 1.0),
            Vec2D::new(1.0, 1.0),
            Vec2D::new(1.0, 0.5),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(0.5, 0.0),
            Vec2D::new(0.5, 0.5),
        ]);
        let expected = vec![
            Vec2D::new(0.0, 0.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(1.0, 1.0),
            Vec2D::new(0.0, 1.0),
        ];

        assert_eq!(actual.len(), expected.len());
        for (&expected, &actual) in expected.iter().zip(actual.iter()) {
            assert!((actual.x - expected.x).abs() <= std::f32::EPSILON);
            assert!((actual.y - expected.y).abs() <= std::f32::EPSILON);
        }
    }

    #[test]
    fn convex_hull_colinear_unsorted() {
        let actual = convex_hull(&[
            Vec2D::new(0.0, 0.0),
            Vec2D::new(0.5, 1.0),
            Vec2D::new(0.0, 0.5),
            Vec2D::new(1.0, 0.5),
            Vec2D::new(1.0, 1.0),
            Vec2D::new(0.5, 0.5),
            Vec2D::new(0.0, 1.0),
            Vec2D::new(0.5, 0.0),
            Vec2D::new(1.0, 0.0),
        ]);
        let expected = vec![
            Vec2D::new(0.0, 0.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(1.0, 1.0),
            Vec2D::new(0.0, 1.0),
        ];

        assert_eq!(actual.len(), expected.len());
        for (&expected, &actual) in expected.iter().zip(actual.iter()) {
            assert!((actual.x - expected.x).abs() <= std::f32::EPSILON);
            assert!((actual.y - expected.y).abs() <= std::f32::EPSILON);
        }
    }
}
