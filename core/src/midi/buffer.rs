use crate::midi::messages::Message;
use crate::time::ClockTime;
use crate::pool::Pool;
use super::bus::BusAddress;

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

  pub fn iter(&self) -> std::slice::Iter<'_, Event> {
    self.events.iter()
  }

  pub fn reset(&mut self) -> &mut Self {
    self.events.clear();
    self
  }

  pub fn push(&mut self, timestamp: ClockTime, message: Message) {
    self.events.push(Event { timestamp, message });
  }
}

pub fn new_buffer_pool(pool_capacity: usize, buffer_capacity: usize) -> Pool<Buffer> {
  let allocator = Box::new(move || Box::new(Buffer::with_capacity(buffer_capacity)));
  let reset = Box::new(|item: &mut Buffer| drop(item.reset()));
  Pool::new(pool_capacity, allocator, reset)
}

pub struct Io {
  pub address: BusAddress,
  pub buffer: Option<Box<Buffer>>,
}

pub type IoVec = Vec<Io>;

pub fn new_io_vec_pool(pool_capacity: usize, vec_capacity: usize) -> Pool<IoVec> {
  let allocator = Box::new(move || Box::new(Vec::with_capacity(vec_capacity)));
  let reset = Box::new(|vec: &mut IoVec| vec.clear());
  Pool::new(pool_capacity, allocator, reset)
}

