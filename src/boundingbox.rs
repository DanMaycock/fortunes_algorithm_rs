use crate::vector2::Vector2;
use std::f64;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
    None,
}

#[derive(Debug)]
pub struct BoundingBox {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

impl BoundingBox {
    pub fn new(left: f64, right: f64, top: f64, bottom: f64) -> Self {
        BoundingBox {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn contains(&self, point: &Vector2) -> bool {
        (point.x >= self.left)
            && (point.x <= self.right)
            && (point.y >= self.top)
            && (point.y <= self.bottom)
    }

    pub fn get_intersection(&self, origin: &Vector2, direction: &Vector2) -> (Vector2, Side) {
        assert!(self.contains(origin));
        let (t1, side1) = if direction.x < 0.0 {
            ((self.right - origin.x) / direction.x, Side::Right)
        } else if direction.x > 0.0 {
            ((self.left - origin.x) / direction.x, Side::Left)
        } else {
            (f64::MIN, Side::None)
        };

        let (t2, side2) = if direction.y > 0.0 {
            ((self.top - origin.y) / direction.y, Side::Top)
        } else if direction.y < 0.0 {
            ((self.bottom - origin.y) / direction.y, Side::Bottom)
        } else {
            (f64::MAX, Side::None)
        };

        let (t, side) = if t2.abs() < t1.abs() {
            (t2, side2)
        } else {
            (t1, side1)
        };

        (*origin + (*direction * t), side)
    }

    pub fn get_top_left(&self) -> Vector2 {
        Vector2::new(self.left, self.top)
    }

    pub fn get_top_right(&self) -> Vector2 {
        Vector2::new(self.right, self.top)
    }

    pub fn get_bottom_left(&self) -> Vector2 {
        Vector2::new(self.left, self.bottom)
    }

    pub fn get_bottom_right(&self) -> Vector2 {
        Vector2::new(self.right, self.bottom)
    }
}
