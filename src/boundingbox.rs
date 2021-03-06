use super::*;
use std::f64;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
    None,
}

impl Side {
    // Iterates round the sides in an anti clockwise direction
    fn next(self) -> Side {
        match self {
            Side::Left => Side::Bottom,
            Side::Top => Side::Left,
            Side::Right => Side::Top,
            Side::Bottom => Side::Right,
            Side::None => Side::None,
        }
    }
}

#[derive(Debug)]
pub struct BoundingBox {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

impl BoundingBox {
    pub fn new(left: f64, right: f64, top: f64, bottom: f64) -> Self {
        BoundingBox {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn contains(&self, point: &cgmath::Point2<f64>) -> bool {
        (point.x >= self.left)
            && (point.x <= self.right)
            && (point.y >= self.top)
            && (point.y <= self.bottom)
    }

    pub fn get_intersection(&self, origin: &cgmath::Point2<f64>, direction: &cgmath::Vector2<f64>) -> (cgmath::Point2<f64>, Side) {
        assert!(self.contains(origin));
        let (t1, side1) = if direction.x < 0.0 {
            ((self.right - origin.x) / direction.x, Side::Right)
        } else if direction.x > 0.0 {
            ((self.left - origin.x) / direction.x, Side::Left)
        } else {
            (f64::MIN, Side::None)
        };

        let (t2, side2) = if direction.y > 0.0 {
            ((self.top - origin.y) / direction.y, Side::Top)
        } else if direction.y < 0.0 {
            ((self.bottom - origin.y) / direction.y, Side::Bottom)
        } else {
            (f64::MAX, Side::None)
        };

        let (t, side) = if t2.abs() < t1.abs() {
            (t2, side2)
        } else {
            (t1, side1)
        };

        (*origin + (*direction * t), side)
    }

    pub fn get_corner(&self, side_1: Side, side_2: Side) -> cgmath::Point2<f64> {
        match (side_1, side_2) {
            (Side::Top, Side::Left) | (Side::Left, Side::Top) => self.get_top_left(),
            (Side::Top, Side::Right) | (Side::Right, Side::Top) => self.get_top_right(),
            (Side::Bottom, Side::Left) | (Side::Left, Side::Bottom) => self.get_bottom_left(),
            (Side::Bottom, Side::Right) | (Side::Right, Side::Bottom) => self.get_bottom_right(),
            _ => panic!("Invalid corner sides"),
        }
    }

    pub fn get_top_left(&self) -> cgmath::Point2<f64> {
        cgmath::Point2::new(self.left, self.top)
    }

    pub fn get_top_right(&self) -> cgmath::Point2<f64> {
        cgmath::Point2::new(self.right, self.top)
    }

    pub fn get_bottom_left(&self) -> cgmath::Point2<f64> {
        cgmath::Point2::new(self.left, self.bottom)
    }

    pub fn get_bottom_right(&self) -> cgmath::Point2<f64> {
        cgmath::Point2::new(self.right, self.bottom)
    }

    pub fn get_intersections(
        &self,
        origin: &cgmath::Point2<f64>,
        destination: &cgmath::Point2<f64>,
    ) -> Vec<(cgmath::Point2<f64>, Side)> {
        let mut intersections = vec![];
        let direction = *destination - *origin;
        // Left
        if origin.x < self.left || destination.x < self.left {
            let t = (self.left - origin.x) / direction.x;
            if t > 0.0 && t < 1.0 {
                let intersection_pt = *origin + (direction * t);
                if intersection_pt.y >= self.top && intersection_pt.y <= self.bottom {
                    intersections.push((intersection_pt, Side::Left));
                }
            }
        }
        // Right
        if origin.x > self.right || destination.x > self.right {
            let t = (self.right - origin.x) / direction.x;
            if t > 0.0 && t < 1.0 {
                let intersection_pt = *origin + (direction * t);
                if intersection_pt.y >= self.top && intersection_pt.y <= self.bottom {
                    intersections.push((intersection_pt, Side::Right));
                }
            }
        }
        // Top
        if origin.y < self.top || destination.y < self.top {
            let t = (self.top - origin.y) / direction.y;
            if t > 0.0 && t < 1.0 {
                let intersection_pt = *origin + (direction * t);
                if intersection_pt.x <= self.right && intersection_pt.x >= self.left {
                    intersections.push((intersection_pt, Side::Top));
                }
            }
        }
        // Bottom
        if origin.y > self.bottom || destination.y > self.bottom {
            let t = (self.bottom - origin.y) / direction.y;
            if t > 0.0 && t < 1.0 {
                let intersection_pt = *origin + (direction * t);
                if intersection_pt.x <= self.right && intersection_pt.x >= self.left {
                    intersections.push((intersection_pt, Side::Bottom));
                }
            }
        }

        intersections
    }

    pub fn intersect_diagram(&self, voronoi: &mut Diagram) {
        let mut vertices_to_remove = vec![];
        let mut half_edges_to_remove = vec![];
        let mut processed_half_edges = vec![];
        for face in voronoi.get_face_indices() {
            let start_half_edge = voronoi.get_face_outer_component(face).unwrap();
            let mut outgoing_half_edge: Option<HalfEdgeKey> = None;
            let mut outgoing_side = Side::None;
            let mut incoming_half_edge: Option<HalfEdgeKey> = None;
            let mut incoming_side = Side::None;
            let mut half_edge = start_half_edge;
            loop {
                let origin = voronoi.get_half_edge_origin(half_edge).unwrap();
                let destination = voronoi.get_half_edge_destination(half_edge).unwrap();
                let inside = self.contains(&voronoi.get_vertex_point(origin));
                let next_inside = self.contains(&voronoi.get_vertex_point(destination));

                let next_half_edge = voronoi.get_half_edge_next(half_edge).unwrap();

                if !inside || !next_inside {
                    let intersections = self.get_intersections(
                        &voronoi.get_vertex_point(origin),
                        &voronoi.get_vertex_point(destination),
                    );
                    if !inside && !next_inside {
                        // Both points are outside the box
                        if intersections.is_empty() {
                            // The edge is outside the box
                            vertices_to_remove.push(origin);
                            if Some(half_edge) == voronoi.get_face_outer_component(face) {
                                // Update the outer component before we delete the half_edge
                                voronoi.set_face_outer_component(
                                    face,
                                    voronoi.get_half_edge_next(half_edge),
                                );
                            }

                            half_edges_to_remove.push(half_edge);
                        } else if intersections.len() == 2 {
                            // The edge crosses the bounds of the box twice
                            vertices_to_remove.push(origin);
                            let half_edge_twin = voronoi.get_half_edge_twin(half_edge);
                            if half_edge_twin.is_some()
                                && processed_half_edges.contains(&half_edge_twin.unwrap())
                            {
                                voronoi.set_half_edge_origin(
                                    half_edge,
                                    voronoi.get_half_edge_destination(half_edge_twin.unwrap()),
                                );
                                voronoi.set_half_edge_destination(
                                    half_edge,
                                    voronoi.get_half_edge_origin(half_edge_twin.unwrap()),
                                );
                            } else {
                                let origin = voronoi.add_vertex(intersections[0].0);
                                let destination = voronoi.add_vertex(intersections[1].0);
                                voronoi.set_half_edge_origin(half_edge, Some(origin));
                                voronoi.set_half_edge_destination(half_edge, Some(destination));
                            }
                            if outgoing_half_edge.is_some() {
                                self.link_vertices(
                                    voronoi,
                                    outgoing_half_edge.unwrap(),
                                    outgoing_side,
                                    half_edge,
                                    intersections[0].1,
                                )
                            }
                            outgoing_half_edge = Some(half_edge);
                            outgoing_side = intersections[1].1;
                            processed_half_edges.push(half_edge);
                        } else {
                            panic!(
                                "An edge that begins inside the box but ends outside can only have a single intersection, origin {:?}, destination {:?}",
                                 &voronoi.get_vertex_point(origin),
                                  &voronoi.get_vertex_point(destination)
                                  );
                        }
                    } else if inside && !next_inside {
                        // Edge is going outside the box
                        if intersections.len() == 1 {
                            let half_edge_twin = voronoi.get_half_edge_twin(half_edge);
                            if half_edge_twin.is_some()
                                && processed_half_edges.contains(&half_edge_twin.unwrap())
                            {
                                voronoi.set_half_edge_destination(
                                    half_edge,
                                    voronoi.get_half_edge_origin(half_edge_twin.unwrap()),
                                );
                            } else {
                                let destination = voronoi.add_vertex(intersections[0].0);
                                voronoi.set_half_edge_destination(half_edge, Some(destination));
                            }
                            if incoming_half_edge.is_some() {
                                self.link_vertices(
                                    voronoi,
                                    half_edge,
                                    intersections[0].1,
                                    incoming_half_edge.unwrap(),
                                    incoming_side,
                                )
                            }
                            outgoing_half_edge = Some(half_edge);
                            outgoing_side = intersections[0].1;
                            processed_half_edges.push(half_edge);
                        } else {
                            panic!(
                                "An edge that begins inside the box but ends outside can only have a single intersection, origin {:?}, destination {:?}", 
                                &voronoi.get_vertex_point(origin), 
                                &voronoi.get_vertex_point(destination)
                                );
                        }
                    } else if !inside && next_inside {
                        // Edge is coming into the box
                        if intersections.len() == 1 {
                            vertices_to_remove.push(origin);
                            let half_edge_twin = voronoi.get_half_edge_twin(half_edge);
                            if half_edge_twin.is_some()
                                && processed_half_edges.contains(&half_edge_twin.unwrap())
                            {
                                voronoi.set_half_edge_origin(
                                    half_edge,
                                    voronoi.get_half_edge_destination(half_edge_twin.unwrap()),
                                );
                            } else {
                                let origin = voronoi.add_vertex(intersections[0].0);
                                voronoi.set_half_edge_origin(half_edge, Some(origin));
                            }
                            if outgoing_half_edge.is_some() {
                                self.link_vertices(
                                    voronoi,
                                    outgoing_half_edge.unwrap(),
                                    outgoing_side,
                                    half_edge,
                                    intersections[0].1,
                                )
                            }
                            incoming_half_edge = Some(half_edge);
                            incoming_side = intersections[0].1;
                            processed_half_edges.push(half_edge);
                        } else {
                            panic!(
                                "An edge that begins inside the box but ends outside can only have a single intersection, origin {:?}, destination {:?}", 
                                &voronoi.get_vertex_point(origin), 
                                &voronoi.get_vertex_point(destination)
                            );
                        }
                    }
                }
                if next_half_edge == start_half_edge {
                    // Back where we started so break out of the loop
                    break;
                }
                half_edge = next_half_edge;
            }
        }
        for half_edge in half_edges_to_remove {
            voronoi.remove_half_edge(half_edge);
        }
        for vertex in vertices_to_remove {
            voronoi.remove_vertex(vertex);
        }
    }

    pub fn link_vertices(
        &self,
        voronoi: &mut Diagram,
        start_edge: HalfEdgeKey,
        start_side: Side,
        end_edge: HalfEdgeKey,
        end_side: Side,
    ) {
        let mut edge = start_edge;
        let mut side = start_side;
        let incident_face = voronoi.get_half_edge_incident_face(edge).unwrap();
        while side != end_side {
            let new_edge = voronoi.add_half_edge(incident_face);
            voronoi.link_half_edges(edge, new_edge);
            voronoi.set_half_edge_origin(new_edge, voronoi.get_half_edge_destination(edge));
            let destination = voronoi.add_vertex(self.get_corner(side, side.next()));
            voronoi.set_half_edge_destination(new_edge, Some(destination));
            side = side.next();
            edge = new_edge;
        }
        let new_edge = voronoi.add_half_edge(incident_face);
        voronoi.link_half_edges(edge, new_edge);
        voronoi.link_half_edges(new_edge, end_edge);
        voronoi.set_half_edge_origin(new_edge, voronoi.get_half_edge_destination(edge));
        voronoi.set_half_edge_destination(new_edge, voronoi.get_half_edge_origin(end_edge));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contains_test() {
        let bbox = BoundingBox::new(0.0, 1.0, 0.0, 1.0);

        assert_eq!(bbox.contains(&cgmath::Point2::new(0.5, 0.5)), true);
        assert_eq!(bbox.contains(&cgmath::Point2::new(1.5, 0.5)), false);
        assert_eq!(bbox.contains(&cgmath::Point2::new(-0.5, 0.5)), false);
        assert_eq!(bbox.contains(&cgmath::Point2::new(0.5, 1.5)), false);
        assert_eq!(bbox.contains(&cgmath::Point2::new(0.5, -0.5)), false);
    }

    #[test]
    fn intersections_test() {
        let bbox = BoundingBox::new(0.0, 1.0, 0.0, 1.0);

        let origin = cgmath::Point2::new(1.5, 0.5);
        let destination = cgmath::Point2::new(0.5, 0.5);

        let intersections = bbox.get_intersections(&origin, &destination);
        assert_eq!(intersections.len(), 1);

        let origin = cgmath::Point2::new(0.5, 1.5);
        let destination = cgmath::Point2::new(0.5, 0.5);

        let intersections = bbox.get_intersections(&origin, &destination);
        assert_eq!(intersections.len(), 1);

        let origin = cgmath::Point2::new(0.5, -0.5);
        let destination = cgmath::Point2::new(0.5, 0.5);

        let intersections = bbox.get_intersections(&origin, &destination);
        assert_eq!(intersections.len(), 1);

        let origin = cgmath::Point2::new(-0.5, 0.5);
        let destination = cgmath::Point2::new(0.5, 0.5);

        let intersections = bbox.get_intersections(&origin, &destination);
        assert_eq!(intersections.len(), 1);

        let origin = cgmath::Point2::new(-0.5, 0.5);
        let destination = cgmath::Point2::new(1.5, 0.5);

        let intersections = bbox.get_intersections(&origin, &destination);
        assert_eq!(intersections.len(), 2);

        let origin = cgmath::Point2::new(0.5, -0.5);
        let destination = cgmath::Point2::new(0.5, 1.5);

        let intersections = bbox.get_intersections(&origin, &destination);
        assert_eq!(intersections.len(), 2);
    }
}
