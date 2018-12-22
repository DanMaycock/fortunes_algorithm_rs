use crate::vector2::Vector2;
use std::f64;

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

    pub fn get_intersection(&self, origin: &Vector2, direction: &Vector2) -> Vector2 {
        assert!(self.contains(origin));
        let t1: f64 = if direction.x > 0.0 {
            (self.right - origin.x) / direction.x
        } else if direction.x < 0.0 {
            (self.left - origin.x) / direction.x
        } else {
            f64::MIN
        };
        let t2: f64 = if direction.y > 0.0 {
            (self.top - origin.y) / direction.y
        } else if direction.y < 0.0 {
            (self.bottom - origin.y) / direction.y
        } else {
            f64::MAX
        };
        let t = if t2 < t1 { t2 } else { t1 };
        *origin + (*direction * t)
    }
}
