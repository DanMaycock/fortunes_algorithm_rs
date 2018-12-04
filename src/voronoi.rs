use generational_arena::{Arena, Index};
use vector2::Vector2;

pub struct Site {
    pub point: Vector2,
    face: Option<Index>,
}

impl Site {
    fn new(point: Vector2) -> Self {
        Site {
            point: point,
            face: None,
        }
    }

    pub fn get_x(&self) -> f64 {
        self.point.x
    }

    pub fn get_y(&self) -> f64 {
        self.point.y
    }
}

struct Vertex {
    point: Vector2,
}

struct HalfEdge {
    origin: Option<Index>,
    destination: Option<Index>,
    twin: Option<Index>,
    incident_face: Option<Index>,
    prev: Option<Index>,
    next: Option<Index>,
}

impl HalfEdge {
    fn new(incident_face: Index) -> Self {
        HalfEdge {
            origin: None,
            destination: None,
            twin: None,
            incident_face: Some(incident_face),
            prev: None,
            next: None,
        }
    }
}

struct Face {
    site: Option<Index>,
    outer_component: Option<Index>,
}

impl Face {
    fn new(site: Index) -> Self {
        Face {
            site: Some(site),
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
    pub fn new(points: &[Vector2]) -> Self {
        let mut voronoi = Voronoi {
            sites: Arena::new(),
            faces: Arena::new(),
            vertices: Arena::new(),
            half_edges: Arena::new(),
        };

        for &point in points {
            let site = voronoi.add_site_for_point(point);
            let face = voronoi.faces.insert(Face::new(site));
            voronoi.set_site_face(site, Some(face));
        }

        voronoi
    }

    // Constructs a twin pair of halfedges that represent the edge
    // Note the half edges created are not fully populated and only know which face they are
    // incident with and which half edge is their twin.
    pub fn add_edge(&mut self, left_site: Index, right_site: Index) -> (Index, Index) {
        info!(
            "Adding edge between sites at {:?} and {:?}",
            left_site, right_site
        );
        let half_edge_1 = self.create_half_edge(left_site);
        let half_edge_2 = self.create_half_edge(right_site);

        self.set_half_edge_twin(half_edge_1, Some(half_edge_2));
        self.set_half_edge_twin(half_edge_2, Some(half_edge_1));

        (half_edge_1, half_edge_2)
    }

    pub fn create_half_edge(&mut self, site: Index) -> Index {
        let face = self.get_site_face(site).unwrap();
        let new_half_edge = self.half_edges.insert(HalfEdge::new(face));

        if self.get_face_outer_component(face).is_none() {
            self.set_face_outer_component(face, Some(new_half_edge));
        }

        new_half_edge
    }

    pub fn get_site_point(&self, site: Index) -> Vector2 {
        let site = self.sites.get(site).unwrap();
        site.point
    }

    fn get_site_face(&self, site: Index) -> Option<Index> {
        let site = self.sites.get(site).unwrap();
        site.face
    }

    fn set_site_face(&mut self, site: Index, face: Option<Index>) {
        let site = self.sites.get_mut(site).unwrap();
        site.face = face;
    }

    fn get_face_outer_component(&self, face: Index) -> Option<Index> {
        let face = self.faces.get(face).unwrap();
        face.outer_component
    }

    fn set_face_outer_component(&mut self, face: Index, half_edge: Option<Index>) {
        let face = self.faces.get_mut(face).unwrap();
        face.outer_component = half_edge;
    }

    fn set_half_edge_twin(&mut self, half_edge: Index, twin_half_edge: Option<Index>) {
        let half_edge = self.half_edges.get_mut(half_edge).unwrap();
        half_edge.twin = twin_half_edge;
    }

    fn add_site_for_point(&mut self, point: Vector2) -> Index {
        let site = Site::new(point);
        let site_index = self.sites.insert(site);
        site_index
    }
}
