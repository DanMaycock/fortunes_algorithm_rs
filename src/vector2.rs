use std::ops::{Add, Mul, Sub};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Sub for Vector2 {
    type Output = Vector2;

    fn sub(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Add for Vector2 {
    type Output = Vector2;

    fn add(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Mul<f64> for Vector2 {
    type Output = Vector2;

    fn mul(self, scalar: f64) -> Vector2 {
        Vector2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}
impl Vector2 {
    pub fn new(x: f64, y: f64) -> Self {
        Vector2 { x, y }
    }

    pub fn scalar_mul(self, scalar: f64) -> Vector2 {
        Vector2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }

    pub fn get_orthogonal(self) -> Self {
        Vector2 {
            x: -self.y,
            y: self.x,
        }
    }

    pub fn get_det(self, other: Vector2) -> f64 {
        self.x * other.y - self.y * other.x
    }

    pub fn get_norm(self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn get_distance(self, other: Vector2) -> f64 {
        (self - other).get_norm()
    }
}

pub fn compute_circumcircle_center(
    point_1: Vector2,
    point_2: Vector2,
    point_3: Vector2,
) -> Vector2 {
    let v1 = (point_1 - point_2).get_orthogonal();
    let v2 = (point_2 - point_3).get_orthogonal();
    let delta = (point_3 - point_1) * 0.5;
    let t = delta.get_det(v2) / v1.get_det(v2);
    (point_1 + point_2) * 0.5 + v1 * t
}
