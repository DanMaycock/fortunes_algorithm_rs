use super::*;
use slotmap::{new_key_type, SlotMap};

new_key_type! { pub struct VertexKey; }
new_key_type! { pub struct HalfEdgeKey; }
new_key_type! { pub struct FaceKey; }

/// A Vertex of the diagram.
///
/// Consists only of the points at which the vertex is located.
#[derive(Clone, Copy)]
struct Vertex {
    point: Vector2,
}

impl Vertex {
    fn new(point: Vector2) -> Self {
        Vertex { point }
    }
}

/// A Half Edge of the diagram.
///
/// Stores the index of the origin and destination vertex of the half_edge as well as the index of
/// the face that is incident to the half edge.
/// It also stores the index of the twin half edge, which is the half edge running in the opposite
/// direction. That is the twin's origin is at the destination vertex and the twin's destination at
/// the origin vertex.
/// Finally it stores the indexes of the previous and next half edges, these are the half edges that
/// immediately proceed and follow this half edge around the same incident face. The previous half
/// edge's destination is the origin of this one and the next half edge's origin is the destination
/// of this one.
#[derive(Clone, Copy)]
struct HalfEdge {
    origin: Option<VertexKey>,
    destination: Option<VertexKey>,
    incident_face: Option<FaceKey>,
    twin: Option<HalfEdgeKey>,
    prev: Option<HalfEdgeKey>,
    next: Option<HalfEdgeKey>,
}

impl HalfEdge {
    fn new(incident_face: FaceKey) -> Self {
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

/// A face of the diagram.
///
/// Consists of a point within the face, for a voronoi diagram this is the point that the region is
/// all the part of the plane closer to take point than any other. It all holds the index of a
/// single bordering half edge.
#[derive(Clone, Copy)]
struct Face {
    point: Vector2,
    outer_component: Option<HalfEdgeKey>,
}

impl Face {
    fn new(point: Vector2) -> Self {
        Face {
            point,
            outer_component: None,
        }
    }
}

/// An iterator around the the half edges around the same face. That is a half edges that define a
/// single polygon in the diagram.
pub struct EdgeIterator<'a> {
    diagram: &'a Diagram,
    start_edge: HalfEdgeKey,
    current_edge: Option<HalfEdgeKey>,
}

impl<'a> Iterator for EdgeIterator<'a> {
    type Item = HalfEdgeKey;

    fn next(&mut self) -> Option<HalfEdgeKey> {
        if self.current_edge.is_none() {
            self.current_edge = Some(self.start_edge);
        } else {
            self.current_edge = self.diagram.get_half_edge_next(self.current_edge.unwrap());
            if self.current_edge == Some(self.start_edge) {
                self.current_edge = None;
            }
        }
        self.current_edge
    }
}

/// A diagram represented by a doubly connected edge list.
/// At it's most basic this is a struct that contains the Faces, Vertices and Half Edges that define
/// the diagram and the methods to manipulate and access them.
#[derive(Default)]
pub struct Diagram {
    faces: SlotMap<FaceKey, Face>,
    vertices: SlotMap<VertexKey, Vertex>,
    half_edges: SlotMap<HalfEdgeKey, HalfEdge>,
}

impl Diagram {
    /// Constructs a new empty diagram
    pub fn new() -> Self {
        Diagram {
            faces: SlotMap::with_key(),
            vertices: SlotMap::with_key(),
            half_edges: SlotMap::with_key(),
        }
    }

    /// Adds a new face to the diagram
    /// # Arguments
    /// * `point` - the point associated with the face
    pub fn add_face(&mut self, point: Vector2) {
        self.faces.insert(Face::new(point));
    }

    /// Returns the index of every face in the diagram
    pub fn get_face_indices(&self) -> Vec<FaceKey> {
        self.faces.iter().map(|(index, _)| index).collect()
    }

    /// Returns the location of every vertex in the diagram
    pub fn get_vertex_points(&self) -> Vec<Vector2> {
        self.vertices
            .iter()
            .map(|(_, vertex)| vertex.point)
            .collect()
    }

    /// Returns a vector with the index of the origin and destination vertices for every edge in
    /// the diagram
    pub fn get_edge_vertices(&self) -> Vec<(VertexKey, VertexKey)> {
        let mut edges = vec![];
        for face in self.get_face_indices() {
            for edge in self.outer_edge_iter(face) {
                if self.get_half_edge_origin(edge).is_some()
                    && self.get_half_edge_destination(edge).is_some()
                {
                    let origin = self.get_half_edge_origin(edge).unwrap();
                    let destination = self.get_half_edge_destination(edge).unwrap();
                    edges.push((origin, destination));
                }
            }
        }
        edges
    }

    /// Returns a vector with the start and end points of every half edge in the diagram
    pub fn get_edge_endpoints(&self) -> Vec<(Vector2, Vector2)> {
        let mut edges = vec![];
        for face in self.get_face_indices() {
            for edge in self.outer_edge_iter(face) {
                if self.get_half_edge_origin(edge).is_some()
                    && self.get_half_edge_destination(edge).is_some()
                {
                    let origin = self.get_half_edge_origin(edge).unwrap();
                    let destination = self.get_half_edge_destination(edge).unwrap();
                    edges.push((
                        self.get_vertex_point(origin),
                        self.get_vertex_point(destination),
                    ));
                }
            }
        }
        edges
    }

    /// Returns a EdgeIterator for a face in the diagram.
    ///
    /// If the diagram has been completed this will allow for iterating through each half edge
    /// defining a region in turn.
    /// # Arguments
    /// * `face` - the index of the face to create the iterator for.
    ///
    /// # Panics
    /// If the face index is invalid.
    pub fn outer_edge_iter(&self, face: FaceKey) -> EdgeIterator {
        let start_edge = self.get_face_outer_component(face).unwrap();
        EdgeIterator {
            diagram: self,
            start_edge,
            current_edge: None,
        }
    }

    /// Helper function to add a new edge to the diagram.
    ///
    /// This is done by constructing the twin pair of half_edges that represent the edge.
    /// # Remarks
    /// The half edges created are not fully populated and only know which face they are
    /// incident with and which half edge is their twin.
    ///
    /// # Arguments
    /// * `left_face` - one of the faces that the edge will be between.
    /// * `right_face` - the other face that the edge will be between.
    pub fn add_edge(
        &mut self,
        left_face: FaceKey,
        right_face: FaceKey,
    ) -> (HalfEdgeKey, HalfEdgeKey) {
        let half_edge_1 = self.add_half_edge(left_face);
        let half_edge_2 = self.add_half_edge(right_face);

        self.set_half_edge_twin(half_edge_1, Some(half_edge_2));
        self.set_half_edge_twin(half_edge_2, Some(half_edge_1));

        (half_edge_1, half_edge_2)
    }

    /// Add a new half edge in the diagram.
    ///
    /// # Arguments
    /// * `face` - the face incident to the new half edge.
    pub fn add_half_edge(&mut self, face: FaceKey) -> HalfEdgeKey {
        let new_half_edge = self.half_edges.insert(HalfEdge::new(face));

        if self.get_face_outer_component(face).is_none() {
            self.set_face_outer_component(face, Some(new_half_edge));
        }

        new_half_edge
    }

    pub fn remove_half_edge(&mut self, half_edge: HalfEdgeKey) {
        self.half_edges.remove(half_edge);
    }

    fn set_half_edge_twin(&mut self, half_edge: HalfEdgeKey, twin_half_edge: Option<HalfEdgeKey>) {
        let half_edge = self.half_edges.get_mut(half_edge).unwrap();
        half_edge.twin = twin_half_edge;
    }

    /// Add a new vertex to the diagram.
    ///
    /// # Arguments
    /// * `point` - the location of the vertex.
    pub fn add_vertex(&mut self, point: Vector2) -> VertexKey {
        self.vertices.insert(Vertex::new(point))
    }

    pub fn remove_vertex(&mut self, vertex: VertexKey) {
        self.vertices.remove(vertex);
    }

    /// Returns the point associated with a face.
    ///
    /// # Arguments
    /// * `face` - the index of the face.
    ///
    /// # Panics
    /// If the face index in invalid
    pub fn get_face_point(&self, face: FaceKey) -> Vector2 {
        let site = self.faces.get(face).unwrap();
        site.point
    }

    /// Returns the outer half edge associated with a face.
    ///
    /// # Arguments
    /// * `face` - the index of the face.
    ///
    /// # Panics
    /// If the face index in invalid
    pub fn get_face_outer_component(&self, face: FaceKey) -> Option<HalfEdgeKey> {
        let face = self.faces.get(face).unwrap();
        face.outer_component
    }

    /// Sets the outer half edge of a face.
    ///
    /// # Arguments
    /// * `face` - the index of the face.
    /// * `half_edge` - the index of the half edge to set as the outer component
    ///
    /// # Panics
    ///  If the face index is invalid.
    pub fn set_face_outer_component(&mut self, face: FaceKey, half_edge: Option<HalfEdgeKey>) {
        let face = self.faces.get_mut(face).unwrap();
        face.outer_component = half_edge;
    }

    /// Gets the index of the twin of a half edge.
    ///
    /// # Arguments
    /// * `half_edge` - the index of the half edge.
    ///
    /// # Panics
    /// If the half edge index is invalid.
    pub fn get_half_edge_twin(&self, half_edge: HalfEdgeKey) -> Option<HalfEdgeKey> {
        let half_edge = self.half_edges.get(half_edge).unwrap();
        half_edge.twin
    }

    /// Gets the index of the incident face of a half edge.
    ///
    /// # Arguments
    /// * `half_edge` - the index of the half edge.
    ///
    /// # Panics
    /// If the half edge index is invalid
    pub fn get_half_edge_incident_face(&self, half_edge: HalfEdgeKey) -> Option<FaceKey> {
        let half_edge = self.half_edges.get(half_edge).unwrap();
        half_edge.incident_face
    }

    /// Gets the previous half edge of a half edge,
    ///
    ///  # Arguments
    ///  * `half_edge` - the index of the half edge we want to find the previous half edge to.
    ///
    /// # Panics
    /// If the half edge index is invalid.
    pub fn get_half_edge_prev(&self, half_edge: HalfEdgeKey) -> Option<HalfEdgeKey> {
        let half_edge = self.half_edges.get(half_edge).unwrap();
        half_edge.prev
    }

    /// Gets the next half edge of a half edge,
    ///
    ///  # Arguments
    ///  * `half_edge` - the index of the half edge we want to find the next half edge to.
    ///
    /// # Panics
    /// If the half edge index is invalid.
    pub fn get_half_edge_next(&self, half_edge: HalfEdgeKey) -> Option<HalfEdgeKey> {
        let half_edge = self.half_edges.get(half_edge).unwrap();
        half_edge.next
    }

    /// Links two half edges so that they are consectutive.
    ///
    /// I.e one will beome the previous of the other and the other will become the next of the
    /// first.
    ///
    /// # Arguments
    /// * `prev` - the preceeding half edge.
    /// * `next` - the subsequent half edge.
    ///
    /// # Panics
    /// If either of the indices are invalid.
    pub fn link_half_edges(&mut self, prev: HalfEdgeKey, next: HalfEdgeKey) {
        self.set_half_edge_prev(next, Some(prev));
        self.set_half_edge_next(prev, Some(next));
    }

    fn set_half_edge_prev(&mut self, half_edge: HalfEdgeKey, prev_half_edge: Option<HalfEdgeKey>) {
        let half_edge = self.half_edges.get_mut(half_edge).unwrap();
        half_edge.prev = prev_half_edge;
    }

    fn set_half_edge_next(&mut self, half_edge: HalfEdgeKey, next_half_edge: Option<HalfEdgeKey>) {
        let half_edge = self.half_edges.get_mut(half_edge).unwrap();
        half_edge.next = next_half_edge;
    }

    /// Sets the origin vertex of a half edge
    /// # Arguments
    /// * `half_edge` - the half_edge we are setting the origin on.
    /// * `origin` - the index of the vertex we are setting as the origin
    ///
    /// # Panics
    /// If the half edge index is invalid.
    pub fn set_half_edge_origin(&mut self, half_edge: HalfEdgeKey, origin: Option<VertexKey>) {
        let half_edge = self.half_edges.get_mut(half_edge).unwrap();
        half_edge.origin = origin;
    }

    /// Returns the index of the origin vertex of a half edge.
    /// # Arguments
    /// * `half_edge` - the index of the half edge to return the origin vertex of.
    ///
    /// # Panics
    /// If the half edge index is invalid.
    pub fn get_half_edge_origin(&self, half_edge: HalfEdgeKey) -> Option<VertexKey> {
        let half_edge = self.half_edges.get(half_edge).unwrap();
        half_edge.origin
    }

    /// Returns the location of the origin of a half edge.
    /// # Arguments
    /// * `half_edge` - the index of the half edge to return the origin point of.
    ///
    /// # Panics
    /// If the half edge index is invalid or the origin vertex index stored in the half edge is
    /// invalid.
    pub fn get_half_edge_origin_point(&self, half_edge: HalfEdgeKey) -> Vector2 {
        let half_edge = self.half_edges.get(half_edge).unwrap();
        self.get_vertex_point(half_edge.origin.unwrap())
    }

    /// Sets the destination vertex of a half edge
    /// # Arguments
    /// * `half_edge` - the half_edge we are setting the origin on.
    /// * `detination` - the index of the vertex we are setting as the origin
    ///
    /// # Panics
    /// If the half edge index is invalid.
    pub fn set_half_edge_destination(
        &mut self,
        half_edge: HalfEdgeKey,
        destination: Option<VertexKey>,
    ) {
        let half_edge = self.half_edges.get_mut(half_edge).unwrap();
        half_edge.destination = destination;
    }

    /// Returns the index of the destination vertex of a half edge.
    /// # Arguments
    /// * `half_edge` - the index of the half edge to return the origin vertex of.
    ///
    /// # Panics
    /// If the half edge index is invalid.
    pub fn get_half_edge_destination(&self, half_edge: HalfEdgeKey) -> Option<VertexKey> {
        let half_edge = self.half_edges.get(half_edge).unwrap();
        half_edge.destination
    }

    /// Returns the point at which a vertex is located.
    /// # Arguments
    /// * `vertex` - the index of the vertex to return the point for.
    ///
    /// # Panics
    /// If the vertex index is invalid.
    pub fn get_vertex_point(&self, vertex: VertexKey) -> Vector2 {
        let vertex = self.vertices.get(vertex).unwrap();
        vertex.point
    }

    /// Calculates the centroid or geometric center of a face in the diagram.
    ///
    /// This is done by taking the arithmetic mean position of all the points around the face.
    /// # Arguments
    /// * `face` - the index of the face to calculate the area of.
    ///
    /// # Panics
    /// If the face index is invalid.
    pub fn calculate_face_center(&self, face: FaceKey) -> Vector2 {
        let mut acc = Vector2::new(0.0, 0.0);
        let mut c = 0;
        for edge in self.outer_edge_iter(face) {
            acc = acc + self.get_half_edge_origin_point(edge);
            c += 1;
        }
        acc * (1.0 / f64::from(c))
    }

    /// Calculates the area of a face in the diagram.
    /// # Arguments
    /// * `face` - the index of the face to calculate the area of.
    ///
    /// # Panics
    /// If the face index is invalid.
    pub fn get_face_area(&self, face: FaceKey) -> f64 {
        self.outer_edge_iter(face)
            .fold(0.0, |acc, edge| {
                let origin = self.get_vertex_point(self.get_half_edge_origin(edge).unwrap());
                let destination =
                    self.get_vertex_point(self.get_half_edge_destination(edge).unwrap());
                acc + origin.x * destination.y - destination.x * origin.y
            })
            .abs()
            * 0.5
    }

    /// If a face has a adjacent edge that is part of the border of the diagram.
    /// # Arguments
    /// * `face` - the index of the face to check.
    ///
    /// # Panics
    /// If the face index is invalid.
    pub fn is_face_on_border(&self, face: FaceKey) -> bool {
        for edge in self.outer_edge_iter(face) {
            if self.get_half_edge_twin(edge).is_none() {
                return true;
            }
        }
        false
    }
}
