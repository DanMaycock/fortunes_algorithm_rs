extern crate voronoi;

use voronoi::Point;

fn main() {
    let points: Vec<Point> = vec![(0.2, 0.2), (0.4, 0.3), (0.7, 0.5), (0.8, 0.9)];

    let diagram = voronoi::generate_diagram(&points);
}
