extern crate generational_arena;

mod beachline;
mod event;
mod voronoi;

use beachline::Beachline;
use event::{Event, EventQueue, EventType};
use generational_arena::Index;
use voronoi::Voronoi;

pub type Point = (f64, f64);

struct FortunesAlgorithm {
    voronoi: Voronoi,
    event_queue: EventQueue,
    beachline: Beachline,
}

pub fn generate_diagram(points: &[Point]) -> Voronoi {
    let mut event_queue = EventQueue::new();

    let mut voronoi = Voronoi::new(points);

    let mut beachline = Beachline::new();

    for (site_index, site) in &voronoi.sites {
        event_queue.add_site_event(site.get_y(), site_index);
    }

    loop {
        let event = event_queue.pop();
        match event {
            Some(event) => handle_event(event, &mut voronoi, &mut beachline),
            None => break,
        }
    }

    voronoi
}

fn handle_event(event: Event, voronoi: &mut Voronoi, beachline: &mut Beachline) {
    match event.event_type {
        EventType::SiteEvent { site } => handle_site_event(site, voronoi, beachline),
        EventType::CircleEvent { point, arc } => {
            handle_circle_event(point, arc, voronoi, beachline)
        }
    }
}

fn handle_site_event(site: Index, voronoi: &mut Voronoi, beachline: &mut Beachline) {}

fn handle_circle_event(point: Point, arc: Index, voronoi: &mut Voronoi, beachline: &mut Beachline) {
}
