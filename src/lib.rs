extern crate generational_arena;
#[macro_use]
extern crate log;

mod beachline;
mod event;
pub mod vector2;
mod voronoi;

use beachline::Beachline;
use event::{EventQueue, EventType};
use generational_arena::Index;
use vector2::{compute_circumcircle_center, Vector2};
use voronoi::Voronoi;

pub fn generate_diagram(points: &[Vector2]) -> Voronoi {
    let mut event_queue = EventQueue::new();

    let mut voronoi = Voronoi::new(points);

    let mut beachline = Beachline::new();

    for (site_index, site) in &voronoi.sites {
        event_queue.add_site_event(site.get_y(), site_index);
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

    voronoi
}

fn handle_event(
    event_type: EventType,
    voronoi: &mut Voronoi,
    beachline: &mut Beachline,
    current_y: f64,
    event_queue: &mut EventQueue,
) {
    info!("event at {}", current_y);
    match event_type {
        EventType::SiteEvent { site } => {
            handle_site_event(site, voronoi, beachline, current_y, event_queue)
        }
        EventType::CircleEvent { point, arc } => {
            handle_circle_event(point, arc, voronoi, beachline, current_y)
        }
    }
}

fn handle_site_event(
    site_index: Index,
    voronoi: &mut Voronoi,
    beachline: &mut Beachline,
    current_y: f64,
    event_queue: &mut EventQueue,
) {
    info!("handling site event for site at: {:?}", site_index);
    // 1 Check if beachline is empty
    if !beachline.has_root() {
        info!("Empty beachline creating root");
        beachline.create_root(site_index);
        return;
    }

    // 2 Look for the arc above the site
    let site_point = voronoi.get_site_point(site_index);
    let arc_index = beachline.locate_arc_above(site_point, current_y, &voronoi.sites);
    // Todo delete events?

    // 3 Replace this arc by new arcs
    let middle_arc = beachline.break_arc(arc_index, site_index);
    let left_arc = beachline.get_prev(middle_arc).unwrap();
    let right_arc = beachline.get_next(middle_arc).unwrap();

    // 4 Add a new edge to the diagram
    let (half_edge_1, half_edge_2) = voronoi.add_edge(
        beachline.get_site(left_arc).unwrap(),
        beachline.get_site(middle_arc).unwrap(),
    );

    beachline.set_right_half_edge(left_arc, Some(half_edge_1));
    beachline.set_left_half_edge(middle_arc, Some(half_edge_2));
    beachline.set_right_half_edge(middle_arc, Some(half_edge_2));
    beachline.set_left_half_edge(right_arc, Some(half_edge_1));

    // 5 Check circle events
    let prev_arc = beachline.get_prev(left_arc);
    if prev_arc.is_some() {
        add_event(
            prev_arc.unwrap(),
            middle_arc,
            right_arc,
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
    left.y < right.y
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
    beachline: &Beachline,
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
    let event_y = center.y - radius;

    if event_y >= current_y {
        let left_breakpoint_moving_right = is_moving_right(left_point, right_point);
        let right_breakpoint_moving_right = is_moving_right(middle_point, right_point);
        let left_initial_x = get_initial_x(left_point, middle_point, left_breakpoint_moving_right);
        let right_initial_x =
            get_initial_x(middle_point, right_point, right_breakpoint_moving_right);

        let is_valid = ((left_breakpoint_moving_right && left_initial_x < center.x)
            || (!left_breakpoint_moving_right && left_initial_x > center.x))
            && (right_breakpoint_moving_right && right_initial_x < center.x
                || !right_breakpoint_moving_right && right_initial_x > center.x);

        if is_valid {
            info!(
                "Adding cicle event at {}, with center at {:?}",
                event_y, center
            );
            event_queue.add_circle_event(event_y, center, middle_arc);
        }
    }
}

fn handle_circle_event(
    point: Vector2,
    arc: Index,
    voronoi: &mut Voronoi,
    beachline: &mut Beachline,
    y: f64,
) {
}
