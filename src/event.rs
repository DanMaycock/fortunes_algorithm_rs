use super::*;
use std::cmp::Ordering;

#[derive(Debug)]
pub enum EventType {
    SiteEvent { face: FaceKey },
    CircleEvent { point: Vector2, arc: NodeKey },
}

#[derive(Debug)]
pub struct Event {
    y: f64,
    event_type: EventType,
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<Ordering> {
        self.y.partial_cmp(&other.y)
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Event) -> bool {
        self.y == other.y
    }
}

impl Event {
    pub fn site_event(y: f64, face: FaceKey) -> Self {
        Event {
            y,
            event_type: EventType::SiteEvent { face },
        }
    }

    pub fn circle_event(y: f64, point: Vector2, arc: NodeKey) -> Self {
        Event {
            y,
            event_type: EventType::CircleEvent { point, arc },
        }
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn event_type(&self) -> &EventType {
        &self.event_type
    }
}
