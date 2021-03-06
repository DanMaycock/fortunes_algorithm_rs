# fortunes_algorithm

## Docs

This is a rust implementation of
[fortune's algorithm](https://en.wikipedia.org/wiki/Fortune%27s_algorithm) to generate a
bounded [voronoi diagram](https://en.wikipedia.org/wiki/Voronoi_diagram) of the plane.


### Implementation Details
The implementation is entirely in safe rust code.

The implementation expects the input to be a vector of points on the 2D plane normalised to the
unit square [0,1] x [0,1]. The resulting diagram is returned as a
[Doubly Connected Edge List](https://en.wikipedia.org/wiki/Doubly_connected_edge_list)
containing the Faces, Half Edges and Vertices that make up the diagram.

### Example Usage

The following code will generate a diagram from 10,000 random points.
```rust
let mut points: Vec<Vector2> = vec![];
let mut rng = rand::thread_rng();
for _ in 0..10,000 {
    points.push(Vector2::new(rng.gen(), rng.gen()));
}
let voronoi = fortunes_algorithm::generate_diagram(&points);
```
