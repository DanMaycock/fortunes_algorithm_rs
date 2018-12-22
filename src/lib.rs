#[macro_use]
extern crate log;

mod beachline;
mod boundingbox;
mod event;
pub mod typedvector;
pub mod vector2;
pub mod voronoi;

use crate::beachline::Beachline;
use crate::boundingbox::BoundingBox;
use crate::event::{EventQueue, EventType};
use crate::vector2::{compute_circumcircle_center, Vector2};
use crate::voronoi::{SiteIndex, VertexIndex, Voronoi};
use generational_arena::Index;
use std::f64;

pub fn generate_diagram(points: &[Vector2]) -> Voronoi {
    let mut event_queue = Box::new(EventQueue::new());

    let mut voronoi = Voronoi::new(points);

    let mut beachline = Beachline::new();

    for (index, site) in voronoi.sites.iter() {
        event_queue.add_site_event(site.y(), index);
    }

    loop {
        let event = event_queue.pop();
        match event {
            Some(event) => handle_event(
                event.event_type,
                &mut voronoi,
                &mut beachline,
                event.y,
                &mut event_queue,
            ),
            None => break,
        }
    }

    bound_diagram(&mut voronoi, &beachline);

    voronoi
}

fn handle_event(
    event_type: EventType,
    voronoi: &mut Voronoi,
    beachline: &mut Beachline,
    current_y: f64,
    event_queue: &mut EventQueue,
) {
    match event_type {
        EventType::SiteEvent { site } => {
            handle_site_event(site, voronoi, beachline, current_y, event_queue)
        }
        EventType::CircleEvent { point, arc } => {
            handle_circle_event(point, arc, voronoi, beachline, current_y, event_queue)
        }
    }
}

fn handle_site_event(
    site: SiteIndex,
    voronoi: &mut Voronoi,
    beachline: &mut Beachline,
    current_y: f64,
    event_queue: &mut EventQueue,
) {
    let point = voronoi.get_site_point(site);
    info!(
        "handling site event for site: {} with point at {:?}",
        site, point
    );

    // 1 Check if beachline is empty
    info!("1. Check if beachline is empty");
    if !beachline.has_root() {
        info!("Empty beachline creating root");
        beachline.create_root(site);
        return;
    }

    // 2 Look for the arc above the site
    info!("2. Looking for arc above the site");
    let site_point = voronoi.get_site_point(site);
    let arc_to_break = beachline.locate_arc_above(site_point, current_y, voronoi);
    delete_event(arc_to_break, beachline, event_queue);

    // 3 Replace this arc by new arcs
    info!("3. Replacing arc with new arcs");
    let middle_arc = beachline.break_arc(arc_to_break, site);
    let left_arc = beachline.get_prev(middle_arc).unwrap();
    let right_arc = beachline.get_next(middle_arc).unwrap();

    // 4 Add a new edge to the diagram
    info!("4. Add a new edge to the voronoi diagram");
    let (half_edge_1, half_edge_2) = voronoi.add_edge(
        beachline.get_site(left_arc).unwrap(),
        beachline.get_site(middle_arc).unwrap(),
    );

    beachline.set_right_half_edge(left_arc, Some(half_edge_1));
    beachline.set_left_half_edge(middle_arc, Some(half_edge_2));
    beachline.set_right_half_edge(middle_arc, Some(half_edge_2));
    beachline.set_left_half_edge(right_arc, Some(half_edge_1));

    // 5 Check circle events
    info!("5. Check for any new circle events");
    let prev_arc = beachline.get_prev(left_arc);
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
    let next_arc = beachline.get_next(right_arc);
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

fn is_moving_right(left: Vector2, right: Vector2) -> bool {
    left.y > right.y
}

fn get_initial_x(left: Vector2, right: Vector2, moving_right: bool) -> f64 {
    match moving_right {
        true => left.x,
        false => right.x,
    }
}

fn add_event(
    left_arc: Index,
    middle_arc: Index,
    right_arc: Index,
    voronoi: &Voronoi,
    beachline: &mut Beachline,
    current_y: f64,
    event_queue: &mut EventQueue,
) {
    info!(
        "Checking if an event need to be added for the arcs at {:?}, {:?} and {:?}",
        left_arc, middle_arc, right_arc
    );
    let left_point = voronoi.get_site_point(beachline.get_site(left_arc).unwrap());
    let middle_point = voronoi.get_site_point(beachline.get_site(middle_arc).unwrap());
    let right_point = voronoi.get_site_point(beachline.get_site(right_arc).unwrap());
    let center = compute_circumcircle_center(left_point, middle_point, right_point);
    let radius = center.get_distance(middle_point);
    let event_y = center.y + radius;
    trace!(
        "Potential circle event center at {:?} with event Y at {}",
        center,
        event_y
    );
    trace!(
        "Left point: {:?}, Middle point: {:?}, Right Point: {:?}",
        left_point,
        middle_point,
        right_point
    );

    if event_y > current_y - f64::EPSILON {
        let left_breakpoint_moving_right = is_moving_right(left_point, middle_point);
        let right_breakpoint_moving_right = is_moving_right(middle_point, right_point);
        let left_initial_x = get_initial_x(left_point, middle_point, left_breakpoint_moving_right);
        let right_initial_x =
            get_initial_x(middle_point, right_point, right_breakpoint_moving_right);

        trace!(
            "Left: Intial x = {}, moving right: {}",
            left_initial_x,
            left_breakpoint_moving_right
        );
        trace!(
            "Right: Intial x = {}, moving right: {}",
            right_initial_x,
            right_breakpoint_moving_right
        );
        let is_valid = ((left_breakpoint_moving_right && left_initial_x <= center.x)
            || (!left_breakpoint_moving_right && left_initial_x >= center.x))
            && (right_breakpoint_moving_right && right_initial_x <= center.x
                || !right_breakpoint_moving_right && right_initial_x >= center.x);

        if is_valid {
            info!(
                "Adding cicle event at {}, with center at {:?}",
                event_y, center
            );
            let event = event_queue.add_circle_event(event_y, center, middle_arc);
            beachline.set_arc_event(middle_arc, event);
        } else {
            info!("Event is not valid");
        }
    } else {
        info!("Event Y is behind the beachline y value so ignoring");
    }
}

fn handle_circle_event(
    point: Vector2,
    arc: Index,
    voronoi: &mut Voronoi,
    beachline: &mut Beachline,
    y: f64,
    event_queue: &mut EventQueue,
) {
    info!("handling circle event at {:?}", point);
    // 1 Add vertex
    let vertex = voronoi.create_vertex(point);

    // 2 Delete all events with this arc
    let left_arc = beachline.get_prev(arc).unwrap();
    let right_arc = beachline.get_next(arc).unwrap();

    delete_event(left_arc, beachline, event_queue);
    delete_event(right_arc, beachline, event_queue);

    // 3. Update the beachline and the diagram
    remove_arc(arc, vertex, voronoi, beachline);

    // 4. Add new circle events
    let left_arc_prev = beachline.get_prev(left_arc);
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
    let right_arc_next = beachline.get_next(right_arc);
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

fn delete_event(arc: Index, beachline: &Beachline, event_queue: &mut EventQueue) {
    let weak_event = beachline.get_arc_event(arc);
    match weak_event.upgrade() {
        Some(event) => {
            let event_index = event.borrow().index;
            event_queue.remove_event(event_index);
        }
        None => (),
    }
}

fn remove_arc(arc: Index, vertex: VertexIndex, voronoi: &mut Voronoi, beachline: &mut Beachline) {
    let prev = beachline.get_prev(arc).unwrap();
    let next = beachline.get_next(arc).unwrap();
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
    voronoi.set_half_edge_next(left_half_edge, Some(right_half_edge));
    voronoi.set_half_edge_prev(right_half_edge, Some(left_half_edge));

    // Create a new edge
    let prev_half_edge = beachline.get_right_half_edge(prev);
    let next_half_edge = beachline.get_left_half_edge(next);

    let (half_edge_1, half_edge_2) = voronoi.add_edge(
        beachline.get_site(prev).unwrap(),
        beachline.get_site(next).unwrap(),
    );

    beachline.set_right_half_edge(prev, Some(half_edge_1));
    beachline.set_left_half_edge(next, Some(half_edge_2));

    voronoi.set_half_edge_destination(half_edge_1, Some(vertex));
    voronoi.set_half_edge_origin(half_edge_2, Some(vertex));
    voronoi.set_half_edge_next(half_edge_1, prev_half_edge);
    voronoi.set_half_edge_prev(prev_half_edge.unwrap(), Some(half_edge_1));
    voronoi.set_half_edge_next(next_half_edge.unwrap(), Some(half_edge_2));
    voronoi.set_half_edge_prev(half_edge_2, next_half_edge);

    // Remove the arc from the beachline
    beachline.remove_arc(arc);
}

fn bound_diagram(voronoi: &mut Voronoi, beachline: &Beachline) {
    // Determine the bounds
    let mut left: f64 = 0.0;
    let mut right: f64 = 1.0;
    let mut top: f64 = 0.0;
    let mut bottom: f64 = 1.0;
    for point in voronoi.get_voronoi_vertices() {
        left = left.min(point.x);
        right = right.max(point.x);
        top = top.min(point.y);
        bottom = bottom.max(point.y);
    }

    let bbox = BoundingBox::new(left, right, top, bottom);

    beachline.complete_edges(&bbox, voronoi);
}
