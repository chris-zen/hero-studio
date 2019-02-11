use crate::midi::messages::Message;
use crate::time::ClockTime;

pub struct Event {
  pub timestamp: ClockTime,
  pub message: Message,
}

pub struct Buffer {
  events: Vec<Event>,
}

impl Buffer {
  pub fn new() -> Buffer {
    Buffer { events: Vec::new() }
  }

  pub fn with_capacity(capacity: usize) -> Buffer {
    Buffer {
      events: Vec::with_capacity(capacity),
    }
  }

  pub fn reset(&mut self) -> &mut Self {
    self.events.clear();
    self
  }

  pub fn push(&mut self, timestamp: ClockTime, message: Message) {
    self.events.push(Event { timestamp, message });
  }
}
