use crate::boundingbox::BoundingBox;
use crate::event::Event;
use crate::vector2::Vector2;
use crate::voronoi::{HalfEdgeIndex, SiteIndex, Voronoi};
use binary_search_tree::Tree;
use generational_arena::Index;
use std::cell::RefCell;
use std::f64;
use std::rc::Weak;

#[derive(PartialEq, Copy, Clone, Debug)]
enum Color {
    RED,
    BLACK,
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum NodeType {
    LeftChild,
    RightChild,
    Orphan,
}

#[derive(Debug, Clone)]
pub struct Arc {
    site: Option<SiteIndex>,
    left_half_edge: Option<HalfEdgeIndex>,
    right_half_edge: Option<HalfEdgeIndex>,

    event: Weak<RefCell<Event>>,
}

impl Arc {
    fn new(site: SiteIndex) -> Self {
        Arc {
            site: Some(site),
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

    pub fn create_root(&mut self, site: SiteIndex) -> Index {
        self.tree.create_root(Arc::new(site))
    }

    pub fn locate_arc_above(&self, point: Vector2, y: f64, voronoi: &Voronoi) -> Index {
        let mut current_arc = self.tree.root.unwrap();
        let mut found = false;
        while !found {
            // Check for the special case where the site for the node is at the current y
            let site = self.get_site(current_arc).unwrap();
            let current_arc_focus = voronoi.get_site_point(site);
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
                    let prev_site = self.get_site(prev.unwrap()).unwrap();
                    compute_breakpoint(
                        voronoi.get_site_point(prev_site),
                        voronoi.get_site_point(site),
                        y,
                    )
                } else {
                    f64::NEG_INFINITY
                };
                let breakpoint_right = if next.is_some() {
                    let next_site = self.get_site(next.unwrap()).unwrap();
                    compute_breakpoint(
                        voronoi.get_site_point(site),
                        voronoi.get_site_point(next_site),
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

    pub fn break_arc(&mut self, node: Index, new_site: SiteIndex) {
        let left_half_edge = self.get_left_half_edge(node);
        let right_half_edge = self.get_right_half_edge(node);
        let arc_site = self.get_site(node).unwrap();

        // Replace contents of existing node to an arc for a new site
        self.tree.set_contents(node, Arc::new(new_site));

        // Create new tree nodes either side of the existing node
        let left_arc = self.tree.insert_before(node, Arc::new(arc_site));
        self.set_left_half_edge(left_arc, left_half_edge);

        let right_arc = self.tree.insert_after(node, Arc::new(arc_site));
        self.set_right_half_edge(right_arc, right_half_edge);
    }

    pub fn complete_edges(&self, bbox: &BoundingBox, voronoi: &mut Voronoi) {
        if self.tree.has_root() {
            let mut left_node = self.tree.get_leftmost_node();
            let mut right_node = self.tree.get_next(left_node.unwrap());
            while right_node.is_some() {
                let left_site = self.get_site(left_node.unwrap()).unwrap();
                let right_site = self.get_site(right_node.unwrap()).unwrap();

                let left_point = voronoi.get_site_point(left_site);
                let right_point = voronoi.get_site_point(right_site);

                let direction = (left_point - right_point).get_orthogonal();
                let origin = (left_point + right_point) * 0.5;
                let intersection = bbox.get_intersection(&origin, &direction);

                let vertex = voronoi.create_vertex(intersection);

                voronoi.set_half_edge_origin(
                    self.get_right_half_edge(left_node.unwrap()).unwrap(),
                    Some(vertex),
                );
                voronoi.set_half_edge_destination(
                    self.get_left_half_edge(right_node.unwrap()).unwrap(),
                    Some(vertex),
                );

                left_node = right_node;
                right_node = self.tree.get_next(left_node.unwrap());
            }
        }
    }

    pub fn get_site(&self, node: Index) -> Option<SiteIndex> {
        let arc = self.tree.get_contents(node);
        arc.site
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
