use super::*;
use petgraph::{graph::node_index, Graph};

type DelauneyGraph = Graph<DelauneyVertex, ()>;

pub struct DelauneyVertex {
    position: Vector2,
    is_edge: bool,
    area: f64,
}

impl DelauneyVertex {
    fn new(position: Vector2, is_edge: bool, area: f64) -> Self {
        DelauneyVertex {
            position,
            is_edge,
            area,
        }
    }

    pub fn position(&self) -> Vector2 {
        self.position
    }

    pub fn is_edge(&self) -> bool {
        self.is_edge
    }

    pub fn area(&self) -> f64 {
        self.area
    }
}

pub struct AdjacentVertexIterator<'a> {
    voronoi: &'a Diagram,
    start_edge: HalfEdgeIndex,
    current_edge: Option<HalfEdgeIndex>,
}

impl<'a> Iterator for AdjacentVertexIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.current_edge.is_none() {
            self.current_edge = Some(self.start_edge);
        } else {
            self.current_edge = self.voronoi.get_half_edge_next(self.current_edge.unwrap());
            if self.current_edge == Some(self.start_edge) {
                self.current_edge = None;
            }
        }
        if self.current_edge.is_some() {
            let twin = self.voronoi.get_half_edge_twin(self.current_edge.unwrap());
            if twin.is_some() {
                Some(
                    self.voronoi
                        .get_half_edge_incident_face(twin.unwrap())
                        .unwrap()
                        .into(),
                )
            } else {
                self.next()
            }
        } else {
            None
        }
    }
}

fn get_adjacent_vertex_iterator(voronoi: &Diagram, index: FaceIndex) -> AdjacentVertexIterator {
    let start_edge = voronoi.get_face_outer_component(index).unwrap();
    AdjacentVertexIterator {
        voronoi: voronoi,
        start_edge,
        current_edge: None,
    }
}

pub fn get_delauney_graph(voronoi: &Diagram) -> DelauneyGraph {
    let mut graph = Graph::new();

    for face in voronoi.get_face_indices() {
        graph.add_node(DelauneyVertex::new(
            voronoi.get_face_point(face),
            voronoi.is_face_on_border(face),
            voronoi.get_face_area(face),
        ));
    }

    for &face in voronoi.get_face_indices().iter() {
        for adjacent_index in get_adjacent_vertex_iterator(voronoi, face) {
            graph.add_edge(node_index(face.into()), node_index(adjacent_index), ());
        }
    }

    graph
}
