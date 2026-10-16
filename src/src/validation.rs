//! Feed the main implementation's semantic events to independent cartridge execution.

use std::cell::RefCell;
use std::rc::Rc;

use adventure_trace::{Event, Moment, Point, Record, Session};
use rawemulator::Validator;

pub struct Validation {
    pending: Rc<RefCell<Vec<Record>>>,
    validator: Validator,
}

impl Validation {
    pub fn attach(session: &mut Session) -> Result<Self, String> {
        let pending = Rc::new(RefCell::new(Vec::new()));
        let output = Rc::clone(&pending);
        session.observe_records(move |record| output.borrow_mut().push(record));
        let validator = Validator::new(vec![Record {
            at: Moment {
                frame: 0,
                point: Point::Boot,
                ordinal: 0,
            },
            event: Event::Begin,
        }])?;
        Ok(Self { pending, validator })
    }

    // Draining each boundary bounds live validation storage to one picture's events.
    pub fn catch_up(&mut self, frame: u64) -> Result<(), String> {
        self.validator.catch_up(self.pending.take(), frame)
    }

    pub fn finish(&mut self, frame: u64) -> Result<(), String> {
        self.validator.finish(self.pending.take(), frame)
    }
}
