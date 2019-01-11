use crate::typedvector::{TypedIndex, TypedVec};
use crate::vector2::Vector2;

pub type VertexIndex = TypedIndex<Vertex>;
pub type HalfEdgeIndex = TypedIndex<HalfEdge>;
pub type FaceIndex = TypedIndex<Face>;

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

pub struct EdgeIterator<'a> {
    voronoi: &'a Voronoi,
    start_edge: HalfEdgeIndex,
    current_edge: Option<HalfEdgeIndex>,
}

impl<'a> Iterator for EdgeIterator<'a> {
    type Item = HalfEdgeIndex;

    fn next(&mut self) -> Option<HalfEdgeIndex> {
        if self.current_edge.is_none() {
            self.current_edge = Some(self.start_edge);
        } else {
            self.current_edge = self.voronoi.get_half_edge_next(self.current_edge.unwrap());
            if self.current_edge == Some(self.start_edge) {
                self.current_edge = None;
            }
        }
        self.current_edge
    }
}

pub struct Face {
    point: Vector2,
    outer_component: Option<HalfEdgeIndex>,
}

impl Face {
    fn new(point: Vector2) -> Self {
        Face {
            point,
            outer_component: None,
        }
    }
}

pub struct Voronoi {
    faces: TypedVec<Face>,
    vertices: TypedVec<Vertex>,
    half_edges: TypedVec<HalfEdge>,
}

impl Voronoi {
    pub fn new(points: &[Vector2]) -> Self {
        let mut voronoi = Voronoi {
            faces: TypedVec::new(),
            vertices: TypedVec::new(),
            half_edges: TypedVec::new(),
        };

        for &point in points {
            voronoi.faces.insert(Face::new(point));
        }

        voronoi
    }

    // Returns the index of every face in the diagram
    pub fn get_faces(&self) -> Vec<FaceIndex> {
        self.faces.iter().map(|(index, _)| index).collect()
    }

    // Returns the index of every face in the diagram
    pub fn get_vertices(&self) -> Vec<VertexIndex> {
        self.vertices.iter().map(|(index, _)| index).collect()
    }

    pub fn outer_edge_iter(&self, face: FaceIndex) -> EdgeIterator {
        let start_edge = self.get_face_outer_component(face).unwrap();
        EdgeIterator {
            voronoi: self,
            start_edge,
            current_edge: None,
        }
    }

    // Constructs a twin pair of halfedges that represent the edge
    // Note the half edges created are not fully populated and only know which face they are
    // incident with and which half edge is their twin.
    pub fn add_edge(
        &mut self,
        left_face: FaceIndex,
        right_face: FaceIndex,
    ) -> (HalfEdgeIndex, HalfEdgeIndex) {
        let half_edge_1 = self.create_half_edge(left_face);
        let half_edge_2 = self.create_half_edge(right_face);

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

    pub fn get_face_point(&self, face: FaceIndex) -> Vector2 {
        let site = self.faces.get(face).unwrap();
        site.point
    }

    pub fn get_face_outer_component(&self, face: FaceIndex) -> Option<HalfEdgeIndex> {
        let face = self.faces.get(face).unwrap();
        face.outer_component
    }

    pub fn set_face_outer_component(&mut self, face: FaceIndex, half_edge: Option<HalfEdgeIndex>) {
        let face = self.faces.get_mut(face).unwrap();
        face.outer_component = half_edge;
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

    pub fn get_half_edge_prev(&self, half_edge: HalfEdgeIndex) -> Option<HalfEdgeIndex> {
        let half_edge = self.half_edges.get(half_edge).unwrap();
        half_edge.prev
    }

    pub fn get_half_edge_next(&self, half_edge: HalfEdgeIndex) -> Option<HalfEdgeIndex> {
        let half_edge = self.half_edges.get(half_edge).unwrap();
        half_edge.next
    }

    // Links two half edges so that they are consectutive. 
    pub fn link_half_edges(&mut self, prev: HalfEdgeIndex, next: HalfEdgeIndex) {
        self.set_half_edge_prev(next, Some(prev));
        self.set_half_edge_next(prev, Some(next));
    }

    fn set_half_edge_prev(
        &mut self,
        half_edge: HalfEdgeIndex,
        prev_half_edge: Option<HalfEdgeIndex>,
    ) {
        let half_edge = self.half_edges.get_mut(half_edge).unwrap();
        half_edge.prev = prev_half_edge;
    }

    fn set_half_edge_next(
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
}
