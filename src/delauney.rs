use super::*;
use petgraph::Graph;

pub type DelauneyGraph = Graph<DelauneyVertex, ()>;

pub struct DelauneyVertex {
    pub position: cgmath::Point2<f64>,
    pub is_edge: bool,
    pub area: f64,
}

impl DelauneyVertex {
    fn new<V: Into<cgmath::Point2<f64>>>(position: V, is_edge: bool, area: f64) -> Self {
        DelauneyVertex {
            position: position.into(),
            is_edge,
            area,
        }
    }
}

pub struct AdjacentFaceIterator<'a> {
    voronoi: &'a Diagram,
    start_edge: HalfEdgeKey,
    current_edge: Option<HalfEdgeKey>,
}

impl<'a> Iterator for AdjacentFaceIterator<'a> {
    type Item = FaceKey;

    fn next(&mut self) -> Option<FaceKey> {
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
                        .unwrap(),
                )
            } else {
                self.next()
            }
        } else {
            None
        }
    }
}

fn get_adjacent_face_iterator(voronoi: &Diagram, index: FaceKey) -> AdjacentFaceIterator {
    let start_edge = voronoi.get_face_outer_component(index).unwrap();
    AdjacentFaceIterator {
        voronoi,
        start_edge,
        current_edge: None,
    }
}

pub fn get_delauney_graph(voronoi: &Diagram) -> DelauneyGraph {
    let mut graph = Graph::new();

    let mut face_to_node_index_map = HashMap::new();

    for face in voronoi.get_face_indices() {
        let node_index = graph.add_node(DelauneyVertex::new(
            voronoi.get_face_point(face),
            voronoi.is_face_on_border(face),
            voronoi.get_face_area(face),
        ));
        face_to_node_index_map.insert(face, node_index);
    }

    for face in voronoi.get_face_indices() {
        let index = face_to_node_index_map[&face];
        for adjacent_face in get_adjacent_face_iterator(voronoi, face) {
            let adjacent_index = face_to_node_index_map[&adjacent_face];
            graph.add_edge(index, adjacent_index, ());
        }
    }

    graph
}
