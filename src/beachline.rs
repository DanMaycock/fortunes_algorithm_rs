use generational_arena::{Arena, Index};

use crate::boundingbox::BoundingBox;
use crate::event::Event;
use crate::vector2::Vector2;
use crate::voronoi::{HalfEdgeIndex, SiteIndex, Voronoi};
use std::cell::RefCell;
use std::f64;
use std::rc::Weak;

#[derive(PartialEq, Copy, Clone, Debug)]
enum Color {
    RED,
    BLACK,
}

#[derive(Debug)]
pub struct Arc {
    parent: Option<Index>,
    left: Option<Index>,
    right: Option<Index>,

    site: Option<SiteIndex>,
    left_half_edge: Option<HalfEdgeIndex>,
    right_half_edge: Option<HalfEdgeIndex>,

    prev: Option<Index>,
    next: Option<Index>,

    event: Weak<RefCell<Event>>,

    color: Color,
}

impl Arc {
    fn new(site: SiteIndex) -> Self {
        Arc {
            // Tree structure
            parent: None,
            left: None,
            right: None,

            // Data
            site: Some(site),
            left_half_edge: None,
            right_half_edge: None,

            event: Weak::new(),

            // Optimisation
            prev: None,
            next: None,
            color: Color::RED,
        }
    }
}

pub struct Beachline {
    arcs: Box<Arena<Arc>>,
    pub root: Option<Index>,
}

impl Beachline {
    pub fn new() -> Self {
        Beachline {
            arcs: Box::new(Arena::new()),
            root: None,
        }
    }

    pub fn has_root(&self) -> bool {
        self.root.is_some()
    }

    pub fn create_root(&mut self, site: SiteIndex) {
        let root = self.arcs.insert(Arc::new(site));
        self.set_color(root, Color::BLACK);
        self.root = Some(root);
    }

    pub fn locate_arc_above(&self, point: Vector2, y: f64, voronoi: &Voronoi) -> Index {
        info!("Searching for arc above point at {:?}", point);
        let mut current_arc = self.root.unwrap();
        let mut found = false;
        while !found {
            // Check for the special case where the site for the arc is at the current y
            let site = self.get_site(current_arc).unwrap();
            let current_arc_focus = voronoi.get_site_point(site);
            info!(
                "Current arc is at {:?} with focus at {:?}",
                current_arc, current_arc_focus
            );
            if current_arc_focus.y == y {
                if point.x < current_arc_focus.x {
                    current_arc = self.get_left(current_arc).unwrap();
                } else if point.x > current_arc_focus.x {
                    current_arc = self.get_right(current_arc).unwrap();
                } else {
                    panic!("Two sites located at the same point");
                }
            } else {
                let prev = self.get_prev(current_arc);
                let next = self.get_next(current_arc);

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
                info!(
                    "Breakpoints left: {}, right: {}.",
                    breakpoint_left, breakpoint_right
                );

                if point.x < breakpoint_left {
                    current_arc = self.get_left(current_arc).unwrap();
                } else if point.x > breakpoint_right {
                    current_arc = self.get_right(current_arc).unwrap();
                } else {
                    found = true;
                }
            }
        }
        info!("Found arc above with index {:?}", current_arc);
        current_arc
    }

    pub fn break_arc(&mut self, arc: Index, new_site: SiteIndex) -> Index {
        let arc_site = self.get_site(arc).unwrap();
        // Create a new subtree
        let middle_arc = self.arcs.insert(Arc::new(new_site));

        let left_arc = self.arcs.insert(Arc::new(arc_site));
        let left_half_edge = self.get_left_half_edge(arc);
        self.set_left_half_edge(left_arc, left_half_edge);

        let right_arc = self.arcs.insert(Arc::new(arc_site));
        let right_half_edge = self.get_right_half_edge(arc);
        self.set_right_half_edge(right_arc, right_half_edge);

        // Insert the subtree in the beachline
        self.replace(arc, middle_arc);
        self.insert_before(middle_arc, left_arc);
        self.insert_after(middle_arc, right_arc);

        // Delete the old arc
        self.arcs.remove(arc);
        middle_arc
    }

    pub fn complete_edges(&self, bbox: &BoundingBox, voronoi: &mut Voronoi) {
        if self.has_root() {
            let mut left_arc = self.get_leftmost_arc(self.root.unwrap());
            let mut right_arc = self.get_next(left_arc);
            while right_arc.is_some() {
                let left_site = self.get_site(left_arc).unwrap();
                let right_site = self.get_site(right_arc.unwrap()).unwrap();
                info!(
                    "Completing edge between sites {} and {}",
                    left_site, right_site
                );

                let left_point = voronoi.get_site_point(left_site);
                let right_point = voronoi.get_site_point(right_site);

                let direction = (left_point - right_point).get_orthogonal();
                let origin = (left_point + right_point) * 0.5;
                let intersection = bbox.get_intersection(&origin, &direction);

                let vertex = voronoi.create_vertex(intersection);

                voronoi.set_half_edge_origin(
                    self.get_right_half_edge(left_arc).unwrap(),
                    Some(vertex),
                );
                voronoi.set_half_edge_destination(
                    self.get_left_half_edge(right_arc.unwrap()).unwrap(),
                    Some(vertex),
                );

                left_arc = right_arc.unwrap();
                right_arc = self.get_next(right_arc.unwrap());
            }
        }
    }

    fn replace(&mut self, old_arc: Index, new_arc: Index) {
        info!(
            "Replacing beachline arc at {:?} with arc at {:?}",
            old_arc, new_arc
        );
        self.transplant(old_arc, Some(new_arc));

        let left = self.get_left(old_arc);
        self.set_left(new_arc, left);
        if left.is_some() {
            self.set_parent(left.unwrap(), Some(new_arc));
        }
        let right = self.get_right(old_arc);
        self.set_right(new_arc, right);
        if right.is_some() {
            self.set_parent(right.unwrap(), Some(new_arc));
        }
        let prev = self.get_prev(old_arc);
        self.set_prev(new_arc, prev);
        if prev.is_some() {
            self.set_next(prev.unwrap(), Some(new_arc));
        }
        let next = self.get_next(old_arc);
        self.set_next(new_arc, next);
        if next.is_some() {
            self.set_prev(next.unwrap(), Some(new_arc));
        }

        let color = self.get_color(old_arc);
        self.set_color(new_arc, color);
    }

    fn insert_before(&mut self, existing_arc: Index, new_arc: Index) {
        let existing_arc_prev = self.get_prev(existing_arc);
        if self.get_left(existing_arc).is_none() {
            self.set_left(existing_arc, Some(new_arc));
            self.set_parent(new_arc, Some(existing_arc));
        } else {
            self.set_right(existing_arc_prev.unwrap(), Some(new_arc));
            self.set_parent(new_arc, existing_arc_prev);
        }
        self.set_prev(new_arc, existing_arc_prev);
        if existing_arc_prev.is_some() {
            self.set_next(existing_arc_prev.unwrap(), Some(new_arc));
        }
        self.set_next(new_arc, Some(existing_arc));
        self.set_prev(existing_arc, Some(new_arc));

        // Balance the tree
        // self.insertFixup(new_arc)
    }

    fn insert_after(&mut self, existing_arc: Index, new_arc: Index) {
        let existing_arc_next = self.get_next(existing_arc);
        if self.get_right(existing_arc).is_none() {
            self.set_right(existing_arc, Some(new_arc));
            self.set_parent(new_arc, Some(existing_arc));
        } else {
            self.set_left(existing_arc_next.unwrap(), Some(new_arc));
            self.set_parent(new_arc, existing_arc_next);
        }

        self.set_next(new_arc, existing_arc_next);
        match self.get_next(new_arc) {
            Some(next) => self.set_prev(next, Some(new_arc)),
            None => {}
        }
        self.set_prev(new_arc, Some(existing_arc));
        self.set_next(existing_arc, Some(new_arc));

        // Balance the tree
        // self.insertFixup(arc_to_insert_index)
    }

    pub fn remove_arc(&mut self, arc: Index) {
        let left = self.get_left(arc);
        let right = self.get_right(arc);
        let original_arc_color = self.get_color(arc);
        if left.is_none() {
            // There is no arc to the left, replace the arc we are removing with that to the right
            self.transplant(arc, right);
        } else if right.is_none() {
            // There is no arc to the right, replace the arc we are removing with that to the left
            self.transplant(arc, left);
        } else {
            // In the middle of the tree, we will replace the arc we are removing with the left
            // most node of the right subtree
            let minimum = self.get_leftmost_arc(right.unwrap());
            let min_parent = self.get_parent(minimum);
            let min_right = self.get_right(minimum);
            if min_parent.unwrap() != arc {
                self.transplant(minimum, min_right);
                self.set_right(minimum, right);
                self.set_parent(right.unwrap(), Some(minimum));
            }
            self.transplant(arc, Some(minimum));
            self.set_left(minimum, left);
            self.set_parent(left.unwrap(), Some(minimum));
            self.set_color(minimum, original_arc_color);
        }
        // TODO check if tree need rebalancing
        let prev = self.get_prev(arc);
        let next = self.get_next(arc);

        if prev.is_some() {
            self.set_next(prev.unwrap(), next);
        }
        if next.is_some() {
            self.set_prev(next.unwrap(), prev);
        }
        self.arcs.remove(arc);
    }

    /// Gets the leftmost arc of the subtree with root at the given arc
    fn get_leftmost_arc(&self, arc: Index) -> Index {
        let mut current_arc = arc;
        loop {
            let next_arc = self.get_left(current_arc);
            if next_arc.is_none() {
                break;
            } else {
                current_arc = next_arc.unwrap();
            }
        }
        current_arc
    }

    /// Swaps out one arc in the tree for another
    /// Updates the parent arc
    fn transplant(&mut self, old_arc: Index, new_arc: Option<Index>) {
        let parent = self.get_parent(old_arc);
        if parent.is_none() {
            self.root = new_arc;
        } else if self.get_left(parent.unwrap()) == Some(old_arc) {
            self.set_left(parent.unwrap(), new_arc);
        } else {
            self.set_right(parent.unwrap(), new_arc);
        }

        if new_arc.is_some() {
            self.set_parent(new_arc.unwrap(), parent);
        }
    }

    fn set_right(&mut self, arc: Index, right: Option<Index>) {
        let arc = self.arcs.get_mut(arc).unwrap();
        arc.right = right;
    }

    pub fn get_right(&self, arc: Index) -> Option<Index> {
        let arc = self.arcs.get(arc).unwrap();
        arc.right
    }

    fn set_left(&mut self, arc: Index, left: Option<Index>) {
        let arc = self.arcs.get_mut(arc).unwrap();
        arc.left = left;
    }

    pub fn get_left(&self, arc: Index) -> Option<Index> {
        if !self.arcs.contains(arc) {
            panic!("get_left: No arc exists at {:?}", arc);
        }
        let arc = self.arcs.get(arc).unwrap();
        arc.left
    }

    fn set_parent(&mut self, arc: Index, parent: Option<Index>) {
        if !self.arcs.contains(arc) {
            panic!("No arc exists at {:?}", arc);
        }
        let arc = self.arcs.get_mut(arc).unwrap();
        arc.parent = parent;
    }

    fn get_parent(&self, arc: Index) -> Option<Index> {
        if !self.arcs.contains(arc) {
            panic!("No arc exists at {:?}", arc);
        }
        let arc = self.arcs.get(arc).unwrap();
        arc.parent
    }

    pub fn get_site(&self, arc: Index) -> Option<SiteIndex> {
        if !self.arcs.contains(arc) {
            panic!("No arc exists at {:?}", arc);
        }
        let arc = self.arcs.get(arc).unwrap();
        arc.site
    }

    pub fn set_left_half_edge(&mut self, arc: Index, left_half_edge: Option<HalfEdgeIndex>) {
        if !self.arcs.contains(arc) {
            panic!("No arc exists at {:?}", arc);
        }
        let arc = self.arcs.get_mut(arc).unwrap();
        arc.left_half_edge = left_half_edge;
    }

    pub fn get_left_half_edge(&self, arc: Index) -> Option<HalfEdgeIndex> {
        if !self.arcs.contains(arc) {
            panic!("No arc exists at {:?}", arc);
        }
        let arc = self.arcs.get(arc).unwrap();
        arc.left_half_edge
    }

    pub fn set_right_half_edge(&mut self, arc: Index, right_half_edge: Option<HalfEdgeIndex>) {
        let arc = self.arcs.get_mut(arc).unwrap();
        arc.right_half_edge = right_half_edge;
    }

    pub fn get_right_half_edge(&self, arc: Index) -> Option<HalfEdgeIndex> {
        let arc = self.arcs.get(arc).unwrap();
        arc.right_half_edge
    }

    fn set_prev(&mut self, arc: Index, prev: Option<Index>) {
        let arc = self.arcs.get_mut(arc).unwrap();
        arc.prev = prev;
    }

    pub fn get_prev(&self, arc: Index) -> Option<Index> {
        let arc = self.arcs.get(arc).unwrap();
        arc.prev
    }

    fn set_next(&mut self, arc: Index, next: Option<Index>) {
        let arc = self.arcs.get_mut(arc).unwrap();
        arc.next = next;
    }

    pub fn get_next(&self, arc: Index) -> Option<Index> {
        let arc = self.arcs.get(arc).unwrap();
        arc.next
    }

    fn set_color(&mut self, arc: Index, color: Color) {
        let arc = self.arcs.get_mut(arc).unwrap();
        arc.color = color;
    }

    fn get_color(&self, arc: Index) -> Color {
        let arc = self.arcs.get(arc).unwrap();
        arc.color
    }

    pub fn set_arc_event(&mut self, arc: Index, event: Weak<RefCell<Event>>) {
        let arc = self.arcs.get_mut(arc).unwrap();
        arc.event = event;
    }

    pub fn get_arc_event(&self, arc: Index) -> Weak<RefCell<Event>> {
        let arc = self.arcs.get(arc).unwrap();
        arc.event.clone()
    }
}

fn compute_breakpoint(point1: Vector2, point2: Vector2, y: f64) -> f64 {
    info!("Computing breakpoint between {:?} and {:?}", point1, point2);
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
