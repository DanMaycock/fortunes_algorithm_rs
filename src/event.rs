use crate::vector2::Vector2;
use crate::voronoi::SiteIndex;
use generational_arena::Index;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub enum EventType {
    SiteEvent { site: SiteIndex },
    CircleEvent { point: Vector2, arc: Index },
}

#[derive(Debug)]
pub struct Event {
    pub y: f64,
    pub index: usize,
    pub event_type: EventType,
}

pub struct EventQueue {
    queue: Vec<Rc<RefCell<Event>>>,
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue { queue: vec![] }
    }

    pub fn add_site_event(&mut self, y: f64, site: SiteIndex) -> Weak<RefCell<Event>> {
        let index = self.queue.len();
        let event = Rc::new(RefCell::new(Event {
            y,
            index,
            event_type: EventType::SiteEvent { site },
        }));
        let weak_event = Rc::downgrade(&event);
        self.queue.push(event);
        self.sift_up(index);
        weak_event
    }

    pub fn add_circle_event(&mut self, y: f64, point: Vector2, arc: Index) -> Weak<RefCell<Event>> {
        let index = self.queue.len();
        let event = Rc::new(RefCell::new(Event {
            y,
            index,
            event_type: EventType::CircleEvent { point, arc },
        }));
        let weak_event = Rc::downgrade(&event);
        self.queue.push(event);
        self.sift_up(index);
        weak_event
    }

    pub fn pop(&mut self) -> Option<Event> {
        match self.queue.pop() {
            Some(event) => {
                let event = Rc::try_unwrap(event);
                match event {
                    Ok(event) => Some(event.into_inner()),
                    Err(_) => panic!("Could not unwrap event Rc, another strong reference exists"),
                }
            }
            None => None,
        }
    }

    pub fn remove_event(&mut self, index: usize) {
        self.swap(index, self.queue.len() - 1);
        self.queue.pop();
        if self.queue.len() > 1 {
            self.sift_down(index);
        }
    }

    fn sift_up(&mut self, mut index: usize) {
        while index != 0 && self.queue[index].borrow().y > self.queue[index - 1].borrow().y {
            self.swap(index, index - 1);
            index = index - 1;
        }
    }

    fn sift_down(&mut self, mut index: usize) {
        while index < self.queue.len() - 1
            && self.queue[index].borrow().y < self.queue[index + 1].borrow().y
        {
            self.swap(index, index + 1);
            index = index + 1;
        }
    }

    fn swap(&mut self, idx_1: usize, idx_2: usize) {
        self.queue.swap(idx_1, idx_2);
        self.queue[idx_1].borrow_mut().index = idx_1;
        self.queue[idx_2].borrow_mut().index = idx_2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add_events() {
        let mut events = EventQueue::new();

        // Insert an initial event
        events.add_site_event(1.0, SiteIndex::new(1));

        // Insert a second event after the first
        events.add_site_event(2.0, SiteIndex::new(1));

        // Ad a third before the first
        events.add_site_event(0.5, SiteIndex::new(1));

        // Now pop the events off the queue and check they are correct
        assert_eq!(events.pop().unwrap().y, 0.5);
        assert_eq!(events.pop().unwrap().y, 1.0);
        assert_eq!(events.pop().unwrap().y, 2.0);

        assert!(events.pop().is_none());
    }

    #[test]
    fn test_remove_events() {
        let mut events = EventQueue::new();

        // Insert an initial event
        events.add_site_event(1.0, SiteIndex::new(1));

        // Insert a second event after the first
        events.add_site_event(2.0, SiteIndex::new(1));

        // Ad a third before the first
        events.add_site_event(0.5, SiteIndex::new(1));

        // Remove the middle event
        events.remove_event(1);

        // Now pop the remaining events off the queue.
        assert_eq!(events.pop().unwrap().y, 0.5);
        assert_eq!(events.pop().unwrap().y, 2.0);

        assert!(events.pop().is_none());
    }

    #[test]
    fn test_real_values() {
        let mut events = EventQueue::new();
        events.add_site_event(0.11141869537040194, SiteIndex::new(1));
        events.add_site_event(0.12051964205677834, SiteIndex::new(1));
        events.add_site_event(0.149179106485832, SiteIndex::new(2));
        events.add_site_event(0.3305212298891148, SiteIndex::new(3));
        events.add_site_event(0.8253313276763707, SiteIndex::new(4));
        events.add_site_event(0.8712778711138446, SiteIndex::new(5));
        events.add_site_event(0.9233746637708448, SiteIndex::new(6));

        assert_eq!(events.pop().unwrap().y, 0.11141869537040194);
        assert_eq!(events.pop().unwrap().y, 0.12051964205677834);
        assert_eq!(events.pop().unwrap().y, 0.149179106485832);

        events.add_site_event(0.6730149742604588, SiteIndex::new(7));

        assert_eq!(events.pop().unwrap().y, 0.3305212298891148);

        events.add_site_event(3.380219501494663, SiteIndex::new(8));

        assert_eq!(events.pop().unwrap().y, 0.6730149742604588);

        events.add_site_event(1.4484342273501185, SiteIndex::new(8));

        assert_eq!(events.pop().unwrap().y, 0.8253313276763707);
        assert_eq!(events.pop().unwrap().y, 0.8712778711138446);
        assert_eq!(events.pop().unwrap().y, 0.9233746637708448);
    }
}
