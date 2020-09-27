#![warn(clippy::all)]
#![forbid(unsafe_code)]
//!# Docs
//!
//! This is a rust implementation of
//! [fortune's algorithm](https://en.wikipedia.org/wiki/Fortune%27s_algorithm) to generate a
//! bounded [voronoi diagram](https://en.wikipedia.org/wiki/Voronoi_diagram) of the plane.
//!
//!
//!## Implementation Details
//! The implementation is entirely in safe rust code.
//!
//! The implementation expects the input to be a vector of points on the 2D plane normalised to the
//! unit square [0,1] x [0,1]. The resulting diagram is returned as a
//! [Doubly Connected Edge List](https://en.wikipedia.org/wiki/Doubly_connected_edge_list)
//! containing the Faces, Half Edges and Vertices that make up the diagram.
//!
//!## Example Usage
//!
//! The following code will generate a diagram from 10,000 random points.
//! ```rust
//! let mut points: Vec<cgmath::Point2> = vec![];
//! let mut rng = rand::thread_rng();
//! for _ in 0..10,000 {
//!     points.push(cgmath::Point2::new(rng.gen(), rng.gen()));
//! }
//! let voronoi = fortunes_algorithm::generate_diagram(&points);
//! ```
mod beachline;
mod boundingbox;
mod delauney;
pub mod diagram;
mod event;
pub mod vector2;
mod voronoi_builder;

use beachline::Beachline;
use binary_search_tree::NodeKey;
use boundingbox::BoundingBox;
use diagram::{Diagram, FaceKey, HalfEdgeKey, VertexKey};
use event::Event;
use event::EventType;
use std::{collections::HashMap, f64};
use vector2::compute_circumcircle_center;
pub use delauney::{DelauneyGraph, DelauneyVertex, get_delauney_graph};
pub use voronoi_builder::build_voronoi;

/// Perform [Lloyd's algorithm](https://en.wikipedia.org/wiki/Lloyd%27s_algorithm) on the supplied points.
///
/// This will attempt to spread the supplied points more evenly by calculating the voronoi diagram
/// of the plane and generate a new series of points which are the centers of the resultant regions.
/// # Arguments
/// * `points` - The initial points, these should be in the range [0, 1] X [0,1].
/// * `iterations` - The number of iterations of that we should perform.
pub fn lloyds_relaxation(
    points: &[cgmath::Point2<f64>],
    iterations: usize,
) -> Vec<cgmath::Point2<f64>> {
    let mut points = points.to_vec();
    for _ in 0..iterations {
        let voronoi = build_voronoi(&points);
        points.clear();
        for face in voronoi.get_face_indices() {
            points.push(voronoi.calculate_face_center(face));
        }
    }
    points
}
