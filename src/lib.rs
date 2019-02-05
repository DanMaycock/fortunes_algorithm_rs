#![warn(clippy::all)]
#![forbid(unsafe_code)]
mod beachline;
mod boundingbox;
pub mod delauney;
mod event;
pub mod typedvector;
pub mod vector2;
pub mod voronoi;
mod voronoi_builder;

use beachline::Beachline;
use boundingbox::BoundingBox;
use event::Event;
use event::EventType;
use generational_arena::Index;
use std::f64;
use vector2::{compute_circumcircle_center, Vector2};
use voronoi::{FaceIndex, HalfEdgeIndex, VertexIndex, Voronoi};

pub fn generate_diagram(points: &[Vector2]) -> Voronoi {
    voronoi_builder::build_voronoi(points)
}

pub fn lloyds_relaxation(points: &[Vector2], iterations: usize) -> Vec<Vector2> {
    let mut points = points.to_vec();
    for _ in 0..iterations {
        let voronoi = generate_diagram(&points);
        points.clear();
        for face in voronoi.get_faces() {
            points.push(voronoi.calculate_face_center(face));
        }
    }
    points
}
