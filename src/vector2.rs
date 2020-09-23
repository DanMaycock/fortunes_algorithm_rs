use cgmath::EuclideanSpace;

pub fn get_orthogonal(a: cgmath::Vector2<f64>) -> cgmath::Vector2<f64> {
    cgmath::Vector2::new(-a.y, a.x)
}

pub fn get_det(a: cgmath::Vector2<f64>, b: cgmath::Vector2<f64>) -> f64 {
    a.x * b.y - a.y * b.x
}

pub fn compute_circumcircle_center(
    point_1: cgmath::Point2<f64>,
    point_2: cgmath::Point2<f64>,
    point_3: cgmath::Point2<f64>,
) -> cgmath::Point2<f64> {
    let v1 = get_orthogonal(point_1 - point_2);
    let v2 = get_orthogonal(point_2 - point_3);
    let delta = (point_3 - point_1) * 0.5;
    let t = get_det(delta, v2) / get_det(v1, v2);
    (point_1 + point_2.to_vec()) * 0.5 + v1 * t
}
