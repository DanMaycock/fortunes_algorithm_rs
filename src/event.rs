use crate::vector2::Vector2;
use crate::voronoi::FaceIndex;
use generational_arena::Index;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub enum EventType {
    SiteEvent { face: FaceIndex },
    CircleEvent { point: Vector2, arc: Index },
}

#[derive(Debug)]
pub struct Event {
    pub y: f64,
    pub index: usize,
    pub event_type: EventType,
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

pub struct EventQueue {
    queue: Vec<Rc<RefCell<Event>>>,
}

fn get_parent(index: usize) -> usize {
    (index + 1) / 2 - 1
}

fn get_left(index: usize) -> usize {
    2 * (index + 1) - 1
}

fn get_right(index: usize) -> usize {
    2 * (index + 1)
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue { queue: vec![] }
    }

    pub fn add_site_event(&mut self, y: f64, face: FaceIndex) -> Weak<RefCell<Event>> {
        let index = self.queue.len();
        let event = Rc::new(RefCell::new(Event {
            y,
            index,
            event_type: EventType::SiteEvent { face },
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
        if self.queue.is_empty() {
            None
        } else {
            self.swap(0, self.queue.len() - 1);
            let popped_event = self.queue.pop().unwrap();
            self.sift_down(0);
            match Rc::try_unwrap(popped_event) {
                Ok(event) => Some(event.into_inner()),
                Err(_rc) => panic!("Could not unwrap event Rc, another strong reference exists"),
            }
        }
    }

    fn update(&mut self, index: usize) {
        if index > 0 && *self.queue[get_parent(index)].borrow() > *self.queue[index].borrow() {
            self.sift_up(index);
        } else {
            self.sift_down(index);
        }
    }

    pub fn remove_event(&mut self, index: usize) {
        self.swap(index, self.queue.len() - 1);
        self.queue.pop();
        if index < self.queue.len() {
            self.update(index);
        }
    }

    fn sift_up(&mut self, mut index: usize) {
        while index > 0 && *self.queue[get_parent(index)].borrow() > *self.queue[index].borrow() {
            self.swap(index, get_parent(index));
            index = get_parent(index);
        }
    }

    fn sift_down(&mut self, mut index: usize) {
        loop {
            let mut new_index = index;
            let left = get_left(index);
            let right = get_right(index);
            if left < self.queue.len()
                && *self.queue[new_index].borrow() > *self.queue[left].borrow()
            {
                new_index = left;
            }
            if right < self.queue.len()
                && *self.queue[new_index].borrow() > *self.queue[right].borrow()
            {
                new_index = right;
            }
            if new_index != index {
                self.swap(index, new_index);
                index = new_index;
            } else {
                break;
            }
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
        events.add_site_event(1.0, FaceIndex::new(1));

        // Insert a second event after the first
        events.add_site_event(2.0, FaceIndex::new(1));

        // Ad a third before the first
        events.add_site_event(0.5, FaceIndex::new(1));

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
        let event = events.add_site_event(1.0, FaceIndex::new(1));

        // Insert a second event after the first
        events.add_site_event(2.0, FaceIndex::new(1));

        // Ad a third before the first
        events.add_site_event(0.5, FaceIndex::new(1));

        // Remove the middle event
        let index = event.upgrade().unwrap().borrow().index;
        events.remove_event(index);

        // Now pop the remaining events off the queue.
        assert_eq!(events.pop().unwrap().y, 0.5);
        assert_eq!(events.pop().unwrap().y, 2.0);

        assert!(events.pop().is_none());
    }

    #[test]
    fn test_real_values() {
        let mut events = EventQueue::new();
        events.add_site_event(0.11141869537040194, FaceIndex::new(1));
        events.add_site_event(0.12051964205677834, FaceIndex::new(1));
        events.add_site_event(0.149179106485832, FaceIndex::new(2));
        events.add_site_event(0.3305212298891148, FaceIndex::new(3));
        events.add_site_event(0.8253313276763707, FaceIndex::new(4));
        events.add_site_event(0.8712778711138446, FaceIndex::new(5));
        events.add_site_event(0.9233746637708448, FaceIndex::new(6));

        assert_eq!(events.pop().unwrap().y, 0.11141869537040194);
        assert_eq!(events.pop().unwrap().y, 0.12051964205677834);
        assert_eq!(events.pop().unwrap().y, 0.149179106485832);

        events.add_site_event(0.6730149742604588, FaceIndex::new(7));

        assert_eq!(events.pop().unwrap().y, 0.3305212298891148);

        events.add_site_event(3.380219501494663, FaceIndex::new(8));

        assert_eq!(events.pop().unwrap().y, 0.6730149742604588);

        events.add_site_event(1.4484342273501185, FaceIndex::new(8));

        assert_eq!(events.pop().unwrap().y, 0.8253313276763707);
        assert_eq!(events.pop().unwrap().y, 0.8712778711138446);
        assert_eq!(events.pop().unwrap().y, 0.9233746637708448);

        assert!(events.pop().is_none());
    }

    #[test]
    fn test_real_values_2() {
        let mut events = EventQueue::new();
        events.add_site_event(0.9291285618036174, FaceIndex::new(1));
        events.add_site_event(0.11376973814842917, FaceIndex::new(1));
        events.add_site_event(0.1440618044332418, FaceIndex::new(1));
        events.add_site_event(0.7657112187832171, FaceIndex::new(1));
        events.add_site_event(0.8967647496759451, FaceIndex::new(1));
        events.add_site_event(0.7105418068248269, FaceIndex::new(1));
        events.add_site_event(0.28622046504100773, FaceIndex::new(1));
        events.add_site_event(0.4102014902644908, FaceIndex::new(1));
        events.add_site_event(0.10483467797705237, FaceIndex::new(1));
        events.add_site_event(0.15793206327012377, FaceIndex::new(1));

        assert_eq!(events.pop().unwrap().y, 0.10483467797705237);
        assert_eq!(events.pop().unwrap().y, 0.11376973814842917);
        assert_eq!(events.pop().unwrap().y, 0.1440618044332418);

        let remove_event_1 = events.add_site_event(2.503990033536895, FaceIndex::new(1));
        assert_eq!(events.pop().unwrap().y, 0.15793206327012377);
        events.add_site_event(0.1654313411464461, FaceIndex::new(1));
        assert_eq!(events.pop().unwrap().y, 0.1654313411464461);
        assert_eq!(events.pop().unwrap().y, 0.28622046504100773);
        events.add_site_event(0.4123255955452093, FaceIndex::new(1));
        assert_eq!(events.pop().unwrap().y, 0.4102014902644908);

        let index = remove_event_1.upgrade().unwrap().borrow().index;
        events.remove_event(index);

        let remove_event_2 = events.add_site_event(0.456022039513554, FaceIndex::new(1));
        events.add_site_event(0.44214569318026803, FaceIndex::new(1));
        assert_eq!(events.pop().unwrap().y, 0.4123255955452093);

        let index = remove_event_2.upgrade().unwrap().borrow().index;
        events.remove_event(index);

        events.add_site_event(0.44494200367797315, FaceIndex::new(1));
        assert_eq!(events.pop().unwrap().y, 0.44214569318026803);
        let remove_event_3 = events.add_site_event(1.2623214358900197, FaceIndex::new(1));
        assert_eq!(events.pop().unwrap().y, 0.44494200367797315);
        assert_eq!(events.pop().unwrap().y, 0.7105418068248269);
        events.add_site_event(0.7118411183018967, FaceIndex::new(1));
        assert_eq!(events.pop().unwrap().y, 0.7118411183018967);
        let remove_event_4 = events.add_site_event(2.771502145276784, FaceIndex::new(1));
        assert_eq!(events.pop().unwrap().y, 0.7657112187832171);
        events.add_site_event(0.7781081082023289, FaceIndex::new(1));
        assert_eq!(events.pop().unwrap().y, 0.7781081082023289);

        let index = remove_event_3.upgrade().unwrap().borrow().index;
        events.remove_event(index);

        events.add_site_event(0.8600721837851804, FaceIndex::new(1));
        assert_eq!(events.pop().unwrap().y, 0.8600721837851804);

        let index = remove_event_4.upgrade().unwrap().borrow().index;
        events.remove_event(index);

        let remove_event_5 = events.add_site_event(1.1904311529677654, FaceIndex::new(1));
        assert_eq!(events.pop().unwrap().y, 0.8967647496759451);
        events.add_site_event(1.075984131974067, FaceIndex::new(1));
        let remove_event_6 = events.add_site_event(4.154050538474555, FaceIndex::new(1));
        assert_eq!(events.pop().unwrap().y, 0.9291285618036174);

        let index = remove_event_6.upgrade().unwrap().borrow().index;
        events.remove_event(index);

        events.add_site_event(0.9703973834012277, FaceIndex::new(1));
        events.add_site_event(0.9982671026021538, FaceIndex::new(1));
        assert_eq!(events.pop().unwrap().y, 0.9703973834012277);
        assert_eq!(events.pop().unwrap().y, 0.9982671026021538);

        let index = remove_event_5.upgrade().unwrap().borrow().index;
        events.remove_event(index);

        events.add_site_event(1.1132702929111873, FaceIndex::new(1));
        assert_eq!(events.pop().unwrap().y, 1.075984131974067);
        assert_eq!(events.pop().unwrap().y, 1.1132702929111873);

        assert!(events.pop().is_none());
    }
}
