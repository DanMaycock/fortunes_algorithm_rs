use super::*;
use cgmath::MetricSpace;
use priority_queue::PriorityQueue;

pub fn build_voronoi(points: &[cgmath::Point2<f64>]) -> Diagram {
    let mut event_queue = PriorityQueue::new();

    let mut voronoi = Diagram::default();

    for &point in points {
        voronoi.add_face(point);
    }

    let mut beachline = Beachline::new();

    for &face in voronoi.get_face_indices().iter() {
        event_queue.push(Event::site_event(voronoi.get_face_point(face).y, face));
    }

    loop {
        let event = event_queue.pop();
        match event {
            Some(event) => handle_event(
                event.event_type(),
                &mut voronoi,
                &mut beachline,
                event.y(),
                &mut event_queue,
            ),
            None => break,
        }
    }

    bound_diagram(&mut voronoi, &beachline);
    let bbox = BoundingBox::new(0.0, 1.0, 0.0, 1.0);
    bbox.intersect_diagram(&mut voronoi);
    voronoi
}

fn handle_event(
    event_type: &EventType,
    voronoi: &mut Diagram,
    beachline: &mut Beachline,
    current_y: f64,
    event_queue: &mut PriorityQueue<Event>,
) {
    match *event_type {
        EventType::SiteEvent { face } => {
            handle_site_event(face, voronoi, beachline, current_y, event_queue)
        }
        EventType::CircleEvent { point, arc } => {
            handle_circle_event(point, arc, voronoi, beachline, current_y, event_queue)
        }
    }
}

fn handle_site_event(
    face: FaceKey,
    voronoi: &mut Diagram,
    beachline: &mut Beachline,
    current_y: f64,
    event_queue: &mut PriorityQueue<Event>,
) {
    // 1 Check if beachline is empty
    if !beachline.tree.has_root() {
        beachline.create_root(face);
        return;
    }

    // 2 Look for the arc above the site
    let point = voronoi.get_face_point(face);
    let middle_arc = beachline.locate_arc_above(point, current_y, voronoi);
    delete_event(middle_arc, beachline, event_queue);

    // 3 Replace this arc by new arcs
    beachline.break_arc(middle_arc, face);
    let left_arc = beachline.tree.get_prev(middle_arc).unwrap();
    let right_arc = beachline.tree.get_next(middle_arc).unwrap();

    // 4 Add a new edge to the diagram
    let (half_edge_1, half_edge_2) = voronoi.add_edge(
        beachline.get_arc_face(left_arc).unwrap(),
        beachline.get_arc_face(middle_arc).unwrap(),
    );

    beachline.set_right_half_edge(left_arc, Some(half_edge_1));
    beachline.set_left_half_edge(middle_arc, Some(half_edge_2));
    beachline.set_right_half_edge(middle_arc, Some(half_edge_2));
    beachline.set_left_half_edge(right_arc, Some(half_edge_1));

    // 5 Check circle events
    let prev_arc = beachline.tree.get_prev(left_arc);
    if prev_arc.is_some() {
        add_event(
            prev_arc.unwrap(),
            left_arc,
            middle_arc,
            voronoi,
            beachline,
            current_y,
            event_queue,
        );
    }
    let next_arc = beachline.tree.get_next(right_arc);
    if next_arc.is_some() {
        add_event(
            middle_arc,
            right_arc,
            next_arc.unwrap(),
            voronoi,
            beachline,
            current_y,
            event_queue,
        );
    }
}

fn is_moving_right(left: cgmath::Point2<f64>, right: cgmath::Point2<f64>) -> bool {
    left.y > right.y
}

fn get_initial_x(left: cgmath::Point2<f64>, right: cgmath::Point2<f64>, moving_right: bool) -> f64 {
    if moving_right {
        left.x
    } else {
        right.x
    }
}

fn add_event(
    left_arc: NodeKey,
    middle_arc: NodeKey,
    right_arc: NodeKey,
    voronoi: &Diagram,
    beachline: &mut Beachline,
    current_y: f64,
    event_queue: &mut PriorityQueue<Event>,
) {
    let left_point = voronoi.get_face_point(beachline.get_arc_face(left_arc).unwrap());
    let middle_point = voronoi.get_face_point(beachline.get_arc_face(middle_arc).unwrap());
    let right_point = voronoi.get_face_point(beachline.get_arc_face(right_arc).unwrap());
    let center = compute_circumcircle_center(left_point, middle_point, right_point);
    let radius = center.distance(middle_point);
    let event_y = center.y + radius;

    if event_y > current_y - f64::EPSILON {
        let left_breakpoint_moving_right = is_moving_right(left_point, middle_point);
        let right_breakpoint_moving_right = is_moving_right(middle_point, right_point);
        let left_initial_x = get_initial_x(left_point, middle_point, left_breakpoint_moving_right);
        let right_initial_x =
            get_initial_x(middle_point, right_point, right_breakpoint_moving_right);

        let is_valid = ((left_breakpoint_moving_right && left_initial_x <= center.x)
            || (!left_breakpoint_moving_right && left_initial_x >= center.x))
            && (right_breakpoint_moving_right && right_initial_x <= center.x
                || !right_breakpoint_moving_right && right_initial_x >= center.x);

        if is_valid {
            let event = event_queue.push(Event::circle_event(event_y, center, middle_arc));
            beachline.set_arc_event(middle_arc, event);
        }
    }
}

fn handle_circle_event(
    point: cgmath::Point2<f64>,
    arc: NodeKey,
    voronoi: &mut Diagram,
    beachline: &mut Beachline,
    y: f64,
    event_queue: &mut PriorityQueue<Event>,
) {
    // 1 Add vertex
    let vertex = voronoi.add_vertex(point);

    // 2 Delete all events with this arc
    let left_arc = beachline.tree.get_prev(arc).unwrap();
    let right_arc = beachline.tree.get_next(arc).unwrap();

    delete_event(left_arc, beachline, event_queue);
    delete_event(right_arc, beachline, event_queue);

    // 3. Update the beachline and the diagram
    remove_arc(arc, vertex, voronoi, beachline);

    // 4. Add new circle events
    let left_arc_prev = beachline.tree.get_prev(left_arc);
    if left_arc_prev.is_some() {
        add_event(
            left_arc_prev.unwrap(),
            left_arc,
            right_arc,
            voronoi,
            beachline,
            y,
            event_queue,
        );
    }
    let right_arc_next = beachline.tree.get_next(right_arc);
    if right_arc_next.is_some() {
        add_event(
            left_arc,
            right_arc,
            right_arc_next.unwrap(),
            voronoi,
            beachline,
            y,
            event_queue,
        );
    }
}

fn delete_event(arc: NodeKey, beachline: &Beachline, event_queue: &mut PriorityQueue<Event>) {
    event_queue.remove(beachline.get_arc_event(arc));
}

fn remove_arc(arc: NodeKey, vertex: VertexKey, voronoi: &mut Diagram, beachline: &mut Beachline) {
    let prev = beachline.tree.get_prev(arc).unwrap();
    let next = beachline.tree.get_next(arc).unwrap();
    let left_half_edge = beachline.get_left_half_edge(arc).unwrap();
    let right_half_edge = beachline.get_right_half_edge(arc).unwrap();
    let prev_right_half_edge = beachline.get_right_half_edge(prev).unwrap();
    let next_left_half_edge = beachline.get_left_half_edge(next).unwrap();

    // End existing edges
    voronoi.set_half_edge_origin(prev_right_half_edge, Some(vertex));
    voronoi.set_half_edge_destination(left_half_edge, Some(vertex));
    voronoi.set_half_edge_origin(right_half_edge, Some(vertex));
    voronoi.set_half_edge_destination(next_left_half_edge, Some(vertex));

    // Join the edges of the middle arc
    voronoi.link_half_edges(left_half_edge, right_half_edge);

    // Create a new edge
    let prev_half_edge = beachline.get_right_half_edge(prev).unwrap();
    let next_half_edge = beachline.get_left_half_edge(next).unwrap();

    let (half_edge_1, half_edge_2) = voronoi.add_edge(
        beachline.get_arc_face(prev).unwrap(),
        beachline.get_arc_face(next).unwrap(),
    );

    beachline.set_right_half_edge(prev, Some(half_edge_1));
    beachline.set_left_half_edge(next, Some(half_edge_2));

    voronoi.set_half_edge_destination(half_edge_1, Some(vertex));
    voronoi.set_half_edge_origin(half_edge_2, Some(vertex));

    voronoi.link_half_edges(half_edge_1, prev_half_edge);
    voronoi.link_half_edges(next_half_edge, half_edge_2);

    // Remove the arc from the beachline
    beachline.tree.delete_node(arc);
}

fn bound_diagram(voronoi: &mut Diagram, beachline: &Beachline) {
    // Determine the bounds
    let mut left: f64 = 0.0;
    let mut right: f64 = 1.0;
    let mut top: f64 = 0.0;
    let mut bottom: f64 = 1.0;
    for point in voronoi.get_vertex_points() {
        left = left.min(point.x);
        right = right.max(point.x);
        top = top.min(point.y);
        bottom = bottom.max(point.y);
    }

    let bbox = BoundingBox::new(left, right, top, bottom);

    beachline.complete_edges(&bbox, voronoi);
}
