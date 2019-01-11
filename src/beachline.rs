use crate::boundingbox::{BoundingBox, Side};
use crate::event::Event;
use crate::vector2::Vector2;
use crate::voronoi::{FaceIndex, HalfEdgeIndex, Voronoi};
use binary_search_tree::Tree;
use generational_arena::Index;
use std::cell::RefCell;
use std::f64;
use std::rc::Weak;

#[derive(Debug, Clone)]
pub struct Arc {
    face: Option<FaceIndex>,
    left_half_edge: Option<HalfEdgeIndex>,
    right_half_edge: Option<HalfEdgeIndex>,

    event: Weak<RefCell<Event>>,
}

impl Arc {
    fn new(face: FaceIndex) -> Self {
        Arc {
            face: Some(face),
            left_half_edge: None,
            right_half_edge: None,

            event: Weak::new(),
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

    pub fn create_root(&mut self, face: FaceIndex) -> Index {
        self.tree.create_root(Arc::new(face))
    }

    pub fn locate_arc_above(&self, point: Vector2, y: f64, voronoi: &Voronoi) -> Index {
        let mut current_arc = self.tree.root.unwrap();
        let mut found = false;
        while !found {
            // Check for the special case where the site for the node is at the current y
            let face = self.get_arc_face(current_arc).unwrap();
            let current_arc_focus = voronoi.get_face_point(face);
            if current_arc_focus.y == y {
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

    pub fn break_arc(&mut self, node: Index, new_face: FaceIndex) {
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

    pub fn complete_edges(&self, bbox: &BoundingBox, voronoi: &mut Voronoi) {
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

                let direction = (left_point - right_point).get_orthogonal();
                let origin = (left_point + right_point) * 0.5;
                let intersection = bbox.get_intersection(&origin, &direction);

                let vertex = voronoi.create_vertex(intersection.0);

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
                let face = voronoi.get_half_edge_incident_face(arriving_edge).unwrap();
                if departing_side == arriving_side {
                    // Both arriving and departing edges are on the same side so no need to add a corner vertex
                    let new_edge = voronoi.create_half_edge(face);

                    voronoi.set_half_edge_origin(
                        new_edge,
                        voronoi.get_half_edge_origin(arriving_edge),
                    );
                    voronoi.set_half_edge_destination(
                        new_edge,
                        voronoi.get_half_edge_destination(departing_edge),
                    );

                    voronoi.link_half_edges(departing_edge, new_edge);
                    voronoi.link_half_edges(new_edge, arriving_edge);
                } else {
                    // Arriving and departing edges are on different sides so we need to add a corner vertex
                    // First we have to figure out which corner to add
                    let new_corner = if departing_side == Side::Top && arriving_side == Side::Left
                        || departing_side == Side::Left && arriving_side == Side::Top
                    {
                        // Top left
                        voronoi.create_vertex(bbox.get_top_left())
                    } else if departing_side == Side::Top && arriving_side == Side::Right
                        || departing_side == Side::Right && arriving_side == Side::Top
                    {
                        // Top Right
                        voronoi.create_vertex(bbox.get_top_right())
                    } else if departing_side == Side::Bottom && arriving_side == Side::Left
                        || departing_side == Side::Left && arriving_side == Side::Bottom
                    {
                        // Bottom left
                        voronoi.create_vertex(bbox.get_bottom_left())
                    } else if departing_side == Side::Bottom && arriving_side == Side::Right
                        || departing_side == Side::Right && arriving_side == Side::Bottom
                    {
                        // Bottom Right
                        voronoi.create_vertex(bbox.get_bottom_right())
                    } else {
                        panic!(
                            "Invalid corner combination {:?} and {:?}",
                            departing_side, arriving_side
                        );
                    };

                    // We need an edge from the arriving half edge to the corner and from the corner to the departing half edge
                    let first_edge = voronoi.create_half_edge(face);
                    let second_edge = voronoi.create_half_edge(face);

                    voronoi.set_half_edge_origin(
                        first_edge,
                        voronoi.get_half_edge_destination(departing_edge),
                    );
                    voronoi.set_half_edge_destination(first_edge, Some(new_corner));
                    voronoi.set_half_edge_origin(second_edge, Some(new_corner));
                    voronoi.set_half_edge_destination(
                        second_edge,
                        voronoi.get_half_edge_origin(arriving_edge),
                    );

                    voronoi.link_half_edges(departing_edge, first_edge);
                    voronoi.link_half_edges(first_edge, second_edge);
                    voronoi.link_half_edges(second_edge, arriving_edge);
                }
            }
        }
    }

    pub fn get_arc_face(&self, node: Index) -> Option<FaceIndex> {
        let arc = self.tree.get_contents(node);
        arc.face
    }

    pub fn set_left_half_edge(&mut self, node: Index, left_half_edge: Option<HalfEdgeIndex>) {
        let arc = self.tree.get_mut_contents(node);
        arc.left_half_edge = left_half_edge;
    }

    pub fn get_left_half_edge(&self, node: Index) -> Option<HalfEdgeIndex> {
        let arc = self.tree.get_contents(node);
        arc.left_half_edge
    }

    pub fn set_right_half_edge(&mut self, node: Index, right_half_edge: Option<HalfEdgeIndex>) {
        let arc = self.tree.get_mut_contents(node);
        arc.right_half_edge = right_half_edge;
    }

    pub fn get_right_half_edge(&self, node: Index) -> Option<HalfEdgeIndex> {
        let arc = self.tree.get_contents(node);
        arc.right_half_edge
    }

    pub fn set_arc_event(&mut self, node: Index, event: Weak<RefCell<Event>>) {
        let arc = self.tree.get_mut_contents(node);
        arc.event = event;
    }

    pub fn get_arc_event(&self, node: Index) -> Weak<RefCell<Event>> {
        let arc = self.tree.get_contents(node);
        arc.event.clone()
    }
}

fn compute_breakpoint(point1: Vector2, point2: Vector2, y: f64) -> f64 {
    let d1 = 1.0 / (2.0 * (point1.y - y));
    let d2 = 1.0 / (2.0 * (point2.y - y));
    let a = d1 - d2;

    let b = 2.0 * (point2.x * d2 - point1.x * d1);
    let c = (point1.y.powi(2) + point1.x.powi(2) - y.powi(2)) * d1
        - (point2.y.powi(2) + point2.x.powi(2) - y.powi(2)) * d2;
    if a == 0.0 {
        // Special case where we have a linear equation
        -c / b
    } else if point1.y == y {
        // Special case where point 1 is on the beachline y
        point1.x
    } else if point2.y == y {
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
        assert_eq!(
            compute_breakpoint(Vector2::new(0.4, 0.5), Vector2::new(0.6, 0.5), 0.8),
            0.5
        );

        assert_eq!(
            compute_breakpoint(Vector2::new(0.25, 0.5), Vector2::new(0.5, 0.25), 0.75),
            0.5
        );

        assert_eq!(
            compute_breakpoint(Vector2::new(0.5, 0.2), Vector2::new(0.6, 0.5), 0.5),
            0.5
        );
    }
}
