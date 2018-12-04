use generational_arena::{Arena, Index};

use std::f64;
use vector2::Vector2;

use voronoi::Site;

#[derive(PartialEq, Copy, Clone)]
enum Color {
    RED,
    BLACK,
}

pub struct Arc {
    parent: Option<Index>,
    left: Option<Index>,
    right: Option<Index>,

    site: Option<Index>,
    left_half_edge: Option<Index>,
    right_half_edge: Option<Index>,

    prev: Option<Index>,
    next: Option<Index>,

    color: Color,
}

impl Arc {
    fn new(site: Index) -> Self {
        Arc {
            // Tree structure
            parent: None,
            left: None,
            right: None,

            // Data
            site: Some(site),
            left_half_edge: None,
            right_half_edge: None,

            // Optimisation
            prev: None,
            next: None,
            color: Color::RED,
        }
    }
}

pub struct Beachline {
    arcs: Arena<Arc>,
    root: Option<Index>,
}

impl Beachline {
    pub fn new() -> Self {
        Beachline {
            arcs: Arena::new(),
            root: None,
        }
    }

    pub fn has_root(&self) -> bool {
        self.root.is_some()
    }

    pub fn create_root(&mut self, site: Index) {
        let root = self.arcs.insert(Arc::new(site));
        self.set_color(root, Color::BLACK);
        self.root = Some(root);
        info!("Creating root arc at {:?}", root);
    }

    pub fn locate_arc_above(&self, point: Vector2, y: f64, sites: &Arena<Site>) -> Index {
        info!("Searching for arc above point at {:?}", point);
        let mut node_index = self.root.unwrap();
        let mut found = false;
        while !found {
            let node = self.arcs.get(node_index).unwrap();
            let breakpoint_left = match node.prev {
                Some(node_index) => {
                    let prev_node = self.arcs.get(node_index).unwrap();
                    let prev_site = sites.get(prev_node.site.unwrap()).unwrap();
                    compute_breakpoint(prev_site.point, point, y)
                }
                None => f64::NEG_INFINITY,
            };
            let breakpoint_right = match node.next {
                Some(node_index) => {
                    let next_node = self.arcs.get(node_index).unwrap();
                    let next_site = sites.get(next_node.site.unwrap()).unwrap();
                    compute_breakpoint(next_site.point, point, y)
                }
                None => f64::INFINITY,
            };
            if point.x < breakpoint_left {
                node_index = node.left.unwrap();
            } else if point.x > breakpoint_right {
                node_index = node.right.unwrap();
            } else {
                found = true;
            }
        }
        info!("Found arc above with index {:?}", node_index);
        node_index
    }

    pub fn break_arc(&mut self, arc: Index, new_site: Index) -> Index {
        info!(
            "Breaking arc with index {:?} to insert arc for site {:?}",
            arc, new_site
        );
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

    fn replace(&mut self, old_arc: Index, new_arc: Index) {
        info!(
            "Replacing beachline arc at {:?} with arc at {:?}",
            old_arc, new_arc
        );
        let parent = self.get_parent(old_arc);
        if parent.is_none() {
            self.root = Some(new_arc)
        } else if self.get_left(parent.unwrap()) == Some(old_arc) {
            self.set_left(parent.unwrap(), Some(new_arc))
        } else {
            self.set_right(parent.unwrap(), Some(new_arc))
        }

        self.set_parent(new_arc, parent);

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
        self.set_prev(new_arc, left);
        if prev.is_some() {
            self.set_next(prev.unwrap(), Some(new_arc));
        }
        let next = self.get_next(old_arc);
        self.set_next(new_arc, left);
        if next.is_some() {
            self.set_prev(next.unwrap(), Some(new_arc));
        }

        let color = self.get_color(old_arc);
        self.set_color(new_arc, color);
    }

    fn insert_before(&mut self, existing_arc: Index, new_arc: Index) {
        let existing_arc_prev = self.get_prev(existing_arc);
        match self.get_left(existing_arc) {
            Some(_) => {
                let prev = self.get_prev(existing_arc).unwrap();
                self.set_right(prev, Some(new_arc));
                self.set_parent(new_arc, existing_arc_prev);
            }
            None => {
                self.set_left(existing_arc, Some(new_arc));
                self.set_parent(new_arc, Some(existing_arc));
            }
        };
        self.set_prev(new_arc, existing_arc_prev);
        match self.get_prev(new_arc) {
            Some(prev) => self.set_next(prev, Some(new_arc)),
            None => {}
        }
        self.set_next(new_arc, Some(existing_arc));
        self.set_prev(existing_arc, Some(new_arc));

        // Balance the tree
        // self.insertFixup(arc_to_insert_index)
    }

    fn insert_after(&mut self, existing_arc: Index, new_arc: Index) {
        let existing_arc_next = self.get_next(existing_arc);
        match self.get_right(existing_arc) {
            Some(_) => {
                let next = self.get_next(existing_arc).unwrap();
                self.set_left(next, Some(new_arc));
                self.set_parent(new_arc, existing_arc_next);
            }
            None => {
                self.set_right(existing_arc, Some(new_arc));
                self.set_parent(new_arc, Some(existing_arc));
            }
        };
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

    fn set_right(&mut self, arc: Index, right: Option<Index>) {
        let arc = self.arcs.get_mut(arc).unwrap();
        arc.right = right;
    }

    fn get_right(&self, arc: Index) -> Option<Index> {
        let arc = self.arcs.get(arc).unwrap();
        arc.right
    }

    fn set_left(&mut self, arc: Index, left: Option<Index>) {
        let arc = self.arcs.get_mut(arc).unwrap();
        arc.left = left;
    }

    fn get_left(&self, arc: Index) -> Option<Index> {
        let arc = self.arcs.get(arc).unwrap();
        arc.left
    }

    fn set_parent(&mut self, arc: Index, parent: Option<Index>) {
        let arc = self.arcs.get_mut(arc).unwrap();
        arc.parent = parent;
    }

    fn get_parent(&self, arc: Index) -> Option<Index> {
        let arc = self.arcs.get(arc).unwrap();
        arc.parent
    }

    pub fn get_site(&self, arc: Index) -> Option<Index> {
        let arc = self.arcs.get(arc).unwrap();
        arc.site
    }

    pub fn set_left_half_edge(&mut self, arc: Index, left_half_edge: Option<Index>) {
        let arc = self.arcs.get_mut(arc).unwrap();
        arc.left_half_edge = left_half_edge;
    }

    pub fn get_left_half_edge(&self, arc: Index) -> Option<Index> {
        let arc = self.arcs.get(arc).unwrap();
        arc.left_half_edge
    }

    pub fn set_right_half_edge(&mut self, arc: Index, right_half_edge: Option<Index>) {
        let arc = self.arcs.get_mut(arc).unwrap();
        arc.right_half_edge = right_half_edge;
    }

    pub fn get_right_half_edge(&self, arc: Index) -> Option<Index> {
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
}

fn compute_breakpoint(point1: Vector2, point2: Vector2, y: f64) -> f64 {
    let d1 = 1.0 / (2.0 * (point1.y - 1.0));
    let d2 = 1.0 / (2.0 * (point2.y - 1.0));
    let a = d1 - d2;
    let b = 2.0 * (point2.x * d2 - point1.x * d1);
    let c = (point1.y.powi(2) + point1.x.powi(2) - y.powi(2)) * d1
        - (point2.y.powi(2) + point2.x.powi(2) - y.powi(2)) * d2;
    let delta = b.powi(2) - 4.0 * a * c;
    (-b + f64::sqrt(delta)) / (2.0 * a)
}
