use generational_arena::Index;
use vector2::Vector2;

pub enum EventType {
    SiteEvent { site: Index },
    CircleEvent { point: Vector2, arc: Index },
}

pub struct Event {
    pub y: f64,
    pub event_type: EventType,
}

pub struct EventQueue {
    queue: Vec<Event>,
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue { queue: vec![] }
    }

    pub fn add_site_event(&mut self, y: f64, site: Index) {
        self.queue.push(Event {
            y,
            event_type: EventType::SiteEvent { site },
        });
        let new_index = self.queue.len() - 1;
        self.sift_up(new_index);
    }

    pub fn add_circle_event(&mut self, y: f64, point: Vector2, arc: Index) {
        self.queue.push(Event {
            y,
            event_type: EventType::CircleEvent { point, arc },
        });
        let new_index = self.queue.len() - 1;
        self.sift_up(new_index);
    }

    pub fn pop(&mut self) -> Option<Event> {
        self.queue.pop()
    }

    pub fn remove_event(&mut self, index: usize) {
        self.queue.remove(index);
    }

    fn sift_up(&mut self, index: usize) {
        if index != 0 && self.queue[index].y > self.queue[index - 1].y {
            self.queue.swap(index, index - 1);
            self.sift_up(index - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use generational_arena::Arena;

    #[test]
    fn test_add_events() {
        let mut sites = Arena::new();
        let idx1 = sites.insert(1);
        let idx2 = sites.insert(2);
        let idx3 = sites.insert(2);

        let mut events = EventQueue::new();

        // Insert an initial event
        events.add_site_event(1.0, idx1);

        // Insert a second event after the first
        events.add_site_event(2.0, idx2);

        // Ad a third before the first
        events.add_site_event(0.5, idx3);

        // Now pop the events off the queue and check they are correct
        assert_eq!(events.pop().unwrap().y, 0.5);
        assert_eq!(events.pop().unwrap().y, 1.0);
        assert_eq!(events.pop().unwrap().y, 2.0);

        assert!(events.pop().is_none());
    }

    fn test_remove_events() {
        let mut sites = Arena::new();
        let idx1 = sites.insert(1);
        let idx2 = sites.insert(2);
        let idx3 = sites.insert(2);

        let mut events = EventQueue::new();

        // Insert an initial event
        events.add_site_event(1.0, idx1);

        // Insert a second event after the first
        events.add_site_event(2.0, idx2);

        // Ad a third before the first
        events.add_site_event(0.5, idx3);

        // Remove the middle event
        events.remove_event(1);

        // Now pop the remaining events off the queue.
        assert_eq!(events.pop().unwrap().y, 0.5);
        assert_eq!(events.pop().unwrap().y, 2.0);

        assert!(events.pop().is_none());
    }
}
