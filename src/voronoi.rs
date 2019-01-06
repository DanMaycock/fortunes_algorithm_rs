use crate::typedvector::{TypedIndex, TypedVec};
use crate::vector2::Vector2;

pub type SiteIndex = TypedIndex<Site>;
pub type VertexIndex = TypedIndex<Vertex>;
pub type HalfEdgeIndex = TypedIndex<HalfEdge>;
pub type FaceIndex = TypedIndex<Face>;

pub struct Site {
    pub point: Vector2,
    face: Option<FaceIndex>,
}

impl Site {
    fn new(point: Vector2) -> Self {
        Site {
            point: point,
            face: None,
        }
    }

    pub fn x(&self) -> f64 {
        self.point.x
    }

    pub fn y(&self) -> f64 {
        self.point.y
    }
}

pub struct Vertex {
    point: Vector2,
}

impl Vertex {
    fn new(point: Vector2) -> Self {
        Vertex { point }
    }
}

#[derive(Debug)]
pub struct HalfEdge {
    origin: Option<VertexIndex>,
    destination: Option<VertexIndex>,
    twin: Option<HalfEdgeIndex>,
    incident_face: Option<FaceIndex>,
    prev: Option<HalfEdgeIndex>,
    next: Option<HalfEdgeIndex>,
}

impl HalfEdge {
    fn new(incident_face: FaceIndex) -> Self {
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

pub struct Face {
    site: Option<SiteIndex>,
    outer_component: Option<HalfEdgeIndex>,
}

impl Face {
    fn new(site: SiteIndex) -> Self {
        Face {
            site: Some(site),
            outer_component: None,
        }
    }
}

pub struct Voronoi {
    pub sites: TypedVec<Site>,
    faces: TypedVec<Face>,
    vertices: TypedVec<Vertex>,
    half_edges: TypedVec<HalfEdge>,
}

impl Voronoi {
    pub fn new(points: &[Vector2]) -> Self {
        let mut voronoi = Voronoi {
            sites: TypedVec::new(),
            faces: TypedVec::new(),
            vertices: TypedVec::new(),
            half_edges: TypedVec::new(),
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
    pub fn add_edge(
        &mut self,
        left_site: SiteIndex,
        right_site: SiteIndex,
    ) -> (HalfEdgeIndex, HalfEdgeIndex) {
        let half_edge_1 = self.create_half_edge(self.get_site_face(left_site).unwrap());
        let half_edge_2 = self.create_half_edge(self.get_site_face(right_site).unwrap());

        self.set_half_edge_twin(half_edge_1, Some(half_edge_2));
        self.set_half_edge_twin(half_edge_2, Some(half_edge_1));

        (half_edge_1, half_edge_2)
    }

    pub fn create_half_edge(&mut self, face: FaceIndex) -> HalfEdgeIndex {
        let new_half_edge = self.half_edges.insert(HalfEdge::new(face));

        if self.get_face_outer_component(face).is_none() {
            self.set_face_outer_component(face, Some(new_half_edge));
        }

        new_half_edge
    }

    pub fn create_vertex(&mut self, point: Vector2) -> VertexIndex {
        self.vertices.insert(Vertex::new(point))
    }

    pub fn get_site_point(&self, site: SiteIndex) -> Vector2 {
        let site = self.sites.get(site).unwrap();
        site.point
    }

    pub fn get_site_face(&self, site: SiteIndex) -> Option<FaceIndex> {
        let site = self.sites.get(site).unwrap();
        site.face
    }

    fn set_site_face(&mut self, site: SiteIndex, face: Option<FaceIndex>) {
        let site = self.sites.get_mut(site).unwrap();
        site.face = face;
    }

    pub fn get_face_outer_component(&self, face: FaceIndex) -> Option<HalfEdgeIndex> {
        let face = self.faces.get(face).unwrap();
        face.outer_component
    }

    fn set_face_outer_component(&mut self, face: FaceIndex, half_edge: Option<HalfEdgeIndex>) {
        let face = self.faces.get_mut(face).unwrap();
        face.outer_component = half_edge;
    }

    pub fn get_face_site(&self, face: FaceIndex) -> Option<SiteIndex> {
        let face = self.faces.get(face).unwrap();
        face.site
    }

    pub fn get_half_edge_twin(&self, half_edge: HalfEdgeIndex) -> Option<HalfEdgeIndex> {
        let half_edge = self.half_edges.get(half_edge).unwrap();
        half_edge.twin
    }

    fn set_half_edge_twin(
        &mut self,
        half_edge: HalfEdgeIndex,
        twin_half_edge: Option<HalfEdgeIndex>,
    ) {
        let half_edge = self.half_edges.get_mut(half_edge).unwrap();
        half_edge.twin = twin_half_edge;
    }

    pub fn get_half_edge_incident_face(&self, half_edge: HalfEdgeIndex) -> Option<FaceIndex> {
        let half_edge = self.half_edges.get(half_edge).unwrap();
        half_edge.incident_face
    }

    fn add_site_for_point(&mut self, point: Vector2) -> SiteIndex {
        let site = Site::new(point);
        let site_index = self.sites.insert(site);
        site_index
    }

    pub fn get_half_edge_prev(&self, half_edge: HalfEdgeIndex) -> Option<HalfEdgeIndex> {
        let half_edge = self.half_edges.get(half_edge).unwrap();
        half_edge.prev
    }

    pub fn set_half_edge_prev(
        &mut self,
        half_edge: HalfEdgeIndex,
        prev_half_edge: Option<HalfEdgeIndex>,
    ) {
        let half_edge = self.half_edges.get_mut(half_edge).unwrap();
        half_edge.prev = prev_half_edge;
    }

    pub fn get_half_edge_next(&self, half_edge: HalfEdgeIndex) -> Option<HalfEdgeIndex> {
        let half_edge = self.half_edges.get(half_edge).unwrap();
        half_edge.next
    }

    pub fn set_half_edge_next(
        &mut self,
        half_edge: HalfEdgeIndex,
        next_half_edge: Option<HalfEdgeIndex>,
    ) {
        let half_edge = self.half_edges.get_mut(half_edge).unwrap();
        half_edge.next = next_half_edge;
    }

    pub fn set_half_edge_origin(&mut self, half_edge: HalfEdgeIndex, origin: Option<VertexIndex>) {
        let half_edge = self.half_edges.get_mut(half_edge).unwrap();
        half_edge.origin = origin;
    }

    pub fn get_half_edge_origin(&self, half_edge: HalfEdgeIndex) -> Option<VertexIndex> {
        let half_edge = self.half_edges.get(half_edge).unwrap();
        half_edge.origin
    }

    pub fn set_half_edge_destination(
        &mut self,
        half_edge: HalfEdgeIndex,
        destination: Option<VertexIndex>,
    ) {
        let half_edge = self.half_edges.get_mut(half_edge).unwrap();
        half_edge.destination = destination;
    }

    pub fn get_half_edge_destination(&self, half_edge: HalfEdgeIndex) -> Option<VertexIndex> {
        let half_edge = self.half_edges.get(half_edge).unwrap();
        half_edge.destination
    }

    pub fn get_vertex_point(&self, vertex: VertexIndex) -> Vector2 {
        let vertex = self.vertices.get(vertex).unwrap();
        vertex.point
    }

    pub fn get_voronoi_vertices(&self) -> Vec<Vector2> {
        self.vertices
            .iter()
            .map(|(_, vertex)| vertex.point)
            .collect()
    }

    pub fn get_delauney_vertices(&self) -> Vec<Vector2> {
        self.faces
            .iter()
            .map(|(_, face)| self.get_site_point(face.site.unwrap()))
            .collect()
    }
}
