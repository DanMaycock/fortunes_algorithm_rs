use generational_arena::{Arena, Index};

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
    event: Option<Index>,

    prev: Option<Index>,
    next: Option<Index>,

    color: Color,
}

impl Arc {
    fn new() -> Self {
        Arc {
            parent: None,
            left: None,
            right: None,
            site: None,
            left_half_edge: None,
            right_half_edge: None,
            event: None,
            prev: None,
            next: None,
            color: Color::RED,
        }
    }
}

pub struct Beachline {
    arcs: Arena<Arc>,
    root: Index,
}

impl Beachline {
    pub fn new() -> Self {
        let mut arcs = Arena::new();
        let mut root = Arc::new();
        root.color = Color::BLACK;
        let root_index = arcs.insert(root);
        Beachline {
            arcs,
            root: root_index,
        }
    }
}
