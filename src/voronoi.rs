use crate::Point;
use generational_arena::{Arena, Index};

pub struct Site {
    point: Point,
    index: Option<Index>,
    face: Option<Index>,
}

impl Site {
    fn new(point: &Point) -> Self {
        Site {
            point: point.clone(),
            index: None,
            face: None,
        }
    }

    fn set_index(&mut self, index: Index) {
        self.index = Some(index);
    }

    fn set_face(&mut self, face: Index) {
        self.face = Some(face);
    }

    pub fn get_x(&self) -> f64 {
        self.point.0
    }

    pub fn get_y(&self) -> f64 {
        self.point.1
    }
}

struct Vertex {
    point: Point,
}

struct HalfEdge {
    origin: Index,
    destination: Index,
    twin: Index,
    incident_face: Index,
    prev: Index,
    next: Index,
}

struct Face {
    site: Index,
    outer_component: Option<Index>,
}

impl Face {
    fn new(site: Index) -> Self {
        Face {
            site,
            outer_component: None,
        }
    }
}

pub struct Voronoi {
    pub sites: Arena<Site>,
    faces: Arena<Face>,
    vertices: Arena<Vertex>,
    half_edges: Arena<HalfEdge>,
}

impl Voronoi {
    pub fn new(points: &[Point]) -> Self {
        let mut voronoi = Voronoi {
            sites: Arena::new(),
            faces: Arena::new(),
            vertices: Arena::new(),
            half_edges: Arena::new(),
        };

        for point in points {
            let site_index = voronoi.add_site_for_point(point);
            let face_index = voronoi.faces.insert(Face::new(site_index));
            voronoi
                .sites
                .get_mut(site_index)
                .unwrap()
                .set_face(face_index);
        }

        voronoi
    }

    fn add_site_for_point(&mut self, point: &Point) -> Index {
        let site = Site::new(point);
        let site_index = self.sites.insert(site);
        self.sites
            .get_mut(site_index)
            .unwrap()
            .set_index(site_index.clone());
        site_index
    }
}
