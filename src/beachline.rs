use super::*;
use crate::vector2::get_orthogonal;
use binary_search_tree::{NodeKey, Tree};
use cgmath::EuclideanSpace;
use priority_queue::QueueIndex;
use std::f64;

#[derive(Debug, Clone)]
pub struct Arc {
    face: Option<FaceKey>,
    left_half_edge: Option<HalfEdgeKey>,
    right_half_edge: Option<HalfEdgeKey>,

    event_index: QueueIndex,
}

impl Arc {
    fn new(face: FaceKey) -> Self {
        Arc {
            face: Some(face),
            left_half_edge: None,
            right_half_edge: None,

            event_index: QueueIndex::new(),
        }
    }
}

pub struct Beachline {
    pub tree: Tree<Arc>,
}

impl Beachline {
    pub fn new() -> Self {
        Beachline { tree: Tree::new() }
    }

    pub fn create_root(&mut self, face: FaceKey) -> NodeKey {
        self.tree.create_root(Arc::new(face))
    }

    pub fn locate_arc_above(
        &self,
        point: cgmath::Point2<f64>,
        y: f64,
        voronoi: &Diagram,
    ) -> NodeKey {
        let mut current_arc = self.tree.root.unwrap();
        let mut found = false;
        while !found {
            // Check for the special case where the site for the node is at the current y
            let face = self.get_arc_face(current_arc).unwrap();
            let current_arc_focus = voronoi.get_face_point(face);
            if (current_arc_focus.y - y).abs() < f64::EPSILON {
                if point.x < current_arc_focus.x {
                    current_arc = self.tree.get_left(current_arc).unwrap();
                } else if point.x > current_arc_focus.x {
                    current_arc = self.tree.get_right(current_arc).unwrap();
                } else {
                    panic!("Two sites located at the same point");
                }
            } else {
                let prev = self.tree.get_prev(current_arc);
                let next = self.tree.get_next(current_arc);

                let breakpoint_left = if prev.is_some() {
                    let prev_face = self.get_arc_face(prev.unwrap()).unwrap();
                    compute_breakpoint(
                        voronoi.get_face_point(prev_face),
                        voronoi.get_face_point(face),
                        y,
                    )
                } else {
                    f64::NEG_INFINITY
                };
                let breakpoint_right = if next.is_some() {
                    let next_face = self.get_arc_face(next.unwrap()).unwrap();
                    compute_breakpoint(
                        voronoi.get_face_point(face),
                        voronoi.get_face_point(next_face),
                        y,
                    )
                } else {
                    f64::INFINITY
                };

                if point.x < breakpoint_left {
                    current_arc = self.tree.get_left(current_arc).unwrap();
                } else if point.x > breakpoint_right {
                    current_arc = self.tree.get_right(current_arc).unwrap();
                } else {
                    found = true;
                }
            }
        }
        current_arc
    }

    pub fn break_arc(&mut self, node: NodeKey, new_face: FaceKey) {
        let left_half_edge = self.get_left_half_edge(node);
        let right_half_edge = self.get_right_half_edge(node);
        let arc_face = self.get_arc_face(node).unwrap();

        // Replace contents of existing node to an arc for a new site
        self.tree.set_contents(node, Arc::new(new_face));

        // Create new tree nodes either side of the existing node
        let left_arc = self.tree.insert_before(node, Arc::new(arc_face));
        self.set_left_half_edge(left_arc, left_half_edge);

        let right_arc = self.tree.insert_after(node, Arc::new(arc_face));
        self.set_right_half_edge(right_arc, right_half_edge);
    }

    pub fn complete_edges(&self, bbox: &BoundingBox, voronoi: &mut Diagram) {
        let mut departing_edges = vec![];
        let mut arriving_edges = vec![];
        if self.tree.has_root() {
            let mut left_node = self.tree.get_leftmost_node();
            let mut right_node = self.tree.get_next(left_node.unwrap());
            while right_node.is_some() {
                let left_face = self.get_arc_face(left_node.unwrap()).unwrap();
                let right_face = self.get_arc_face(right_node.unwrap()).unwrap();

                let left_point = voronoi.get_face_point(left_face);
                let right_point = voronoi.get_face_point(right_face);

                let direction = get_orthogonal(left_point - right_point);
                let origin = (left_point + right_point.to_vec()) * 0.5;
                let intersection = bbox.get_intersection(&origin, &direction);

                let vertex = voronoi.add_vertex(intersection.0);

                let arriving_edge = self.get_right_half_edge(left_node.unwrap()).unwrap();
                voronoi.set_half_edge_origin(arriving_edge, Some(vertex));
                let departing_edge = self.get_left_half_edge(right_node.unwrap()).unwrap();
                voronoi.set_half_edge_destination(departing_edge, Some(vertex));

                // Store the vertex on the boundary
                departing_edges.push((departing_edge, intersection.1));
                arriving_edges.push((arriving_edge, intersection.1));

                left_node = right_node;
                right_node = self.tree.get_next(left_node.unwrap());
            }
            for (departing_edge, departing_side) in departing_edges {
                // Find the corresponding arriving edge
                let mut current_edge = departing_edge;
                while voronoi.get_half_edge_prev(current_edge).is_some() {
                    current_edge = voronoi.get_half_edge_prev(current_edge).unwrap();
                }
                let &(arriving_edge, arriving_side) = arriving_edges
                    .iter()
                    .find(|&&(edge, _)| edge == current_edge)
                    .unwrap();

                // The arriving and departing vertices should have the same incident face
                debug_assert_eq!(
                    voronoi.get_half_edge_incident_face(departing_edge),
                    voronoi.get_half_edge_incident_face(arriving_edge)
                );

                bbox.link_vertices(
                    voronoi,
                    departing_edge,
                    departing_side,
                    arriving_edge,
                    arriving_side,
                );
            }
        }
    }

    pub fn get_arc_face(&self, node: NodeKey) -> Option<FaceKey> {
        let arc = self.tree.get_contents(node);
        arc.face
    }

    pub fn set_left_half_edge(&mut self, node: NodeKey, left_half_edge: Option<HalfEdgeKey>) {
        let arc = self.tree.get_mut_contents(node);
        arc.left_half_edge = left_half_edge;
    }

    pub fn get_left_half_edge(&self, node: NodeKey) -> Option<HalfEdgeKey> {
        let arc = self.tree.get_contents(node);
        arc.left_half_edge
    }

    pub fn set_right_half_edge(&mut self, node: NodeKey, right_half_edge: Option<HalfEdgeKey>) {
        let arc = self.tree.get_mut_contents(node);
        arc.right_half_edge = right_half_edge;
    }

    pub fn get_right_half_edge(&self, node: NodeKey) -> Option<HalfEdgeKey> {
        let arc = self.tree.get_contents(node);
        arc.right_half_edge
    }

    pub fn set_arc_event(&mut self, node: NodeKey, event: QueueIndex) {
        let arc = self.tree.get_mut_contents(node);
        arc.event_index = event;
    }

    pub fn get_arc_event(&self, node: NodeKey) -> QueueIndex {
        let arc = self.tree.get_contents(node);
        arc.event_index.clone()
    }
}

fn compute_breakpoint(point1: cgmath::Point2<f64>, point2: cgmath::Point2<f64>, y: f64) -> f64 {
    let d1 = 1.0 / (2.0 * (point1.y - y));
    let d2 = 1.0 / (2.0 * (point2.y - y));
    let a = d1 - d2;

    let b = 2.0 * (point2.x * d2 - point1.x * d1);
    let c = (point1.y.powi(2) + point1.x.powi(2) - y.powi(2)) * d1
        - (point2.y.powi(2) + point2.x.powi(2) - y.powi(2)) * d2;
    if a == 0.0 {
        // Special case where we have a linear equation
        -c / b
    } else if (point1.y - y).abs() < f64::EPSILON {
        // Special case where point 1 is on the beachline y
        point1.x
    } else if (point2.y - y).abs() < f64::EPSILON {
        // Special case where point 2 is on the beachline y
        point2.y
    } else {
        let delta = b.powi(2) - 4.0 * a * c;
        (-b - f64::sqrt(delta)) / (2.0 * a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_breakpoint_test() {
        assert!(
            (compute_breakpoint(
                cgmath::Point2::new(0.4, 0.5),
                cgmath::Point2::new(0.6, 0.5),
                0.8
            ) - 0.5)
                .abs()
                < f64::EPSILON
        );

        assert!(
            (compute_breakpoint(
                cgmath::Point2::new(0.25, 0.5),
                cgmath::Point2::new(0.5, 0.25),
                0.75
            ) - 0.5)
                .abs()
                < f64::EPSILON
        );

        assert!(
            (compute_breakpoint(
                cgmath::Point2::new(0.5, 0.2),
                cgmath::Point2::new(0.6, 0.5),
                0.5
            ) - 0.5)
                .abs()
                < f64::EPSILON
        );
    }
}
