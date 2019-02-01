use crate::vector2::Vector2;
use crate::voronoi::{FaceIndex, HalfEdgeIndex, Voronoi};

pub struct AdjacentVertexIterator<'a> {
    voronoi: &'a Voronoi,
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

/**
 * Wrapper round a voronoi diagram that provides methods for accessing the dual delauney graph.
 */
pub struct Delauney<'a> {
    voronoi: &'a Voronoi,
}

impl<'a> Delauney<'a> {
    pub fn new(voronoi: &'a Voronoi) -> Self {
        Delauney { voronoi }
    }

    pub fn get_vertices(&self) -> Vec<Vector2> {
        self.voronoi
            .get_faces()
            .iter()
            .map(|&face| self.voronoi.get_face_point(face))
            .collect()
    }

    pub fn get_edges(&self) -> Vec<(usize, usize)> {
        let mut edges = vec![];
        for (index, _) in self.get_vertices().iter().enumerate() {
            for adjacent_index in self.get_adjacent_vertex_iterator(index) {
                edges.push((index, adjacent_index));
            }
        }
        edges
    }

    pub fn get_vertex_point(&self, index: usize) -> Vector2 {
        self.voronoi.get_face_point(FaceIndex::new(index))
    }

    pub fn get_adjacent_vertex_iterator(&self, index: usize) -> AdjacentVertexIterator {
        let start_edge = self
            .voronoi
            .get_face_outer_component(FaceIndex::new(index))
            .unwrap();
        AdjacentVertexIterator {
            voronoi: self.voronoi,
            start_edge,
            current_edge: None,
        }
    }

    pub fn is_edge_vertex(&self, index: usize) -> bool {
        self.voronoi.is_edge_face(FaceIndex::new(index))
    }
}
