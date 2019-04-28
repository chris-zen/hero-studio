use std::ops::{Deref, DerefMut};

pub type Allocator<T> = Fn() -> Box<T>;
pub type Reset<T> = Fn(&mut T);

pub struct Pool<T> {
  allocator: Box<Allocator<T>>,
  reset: Box<Reset<T>>,
  items: Vec<Box<T>>,
}

impl<T> Pool<T> {
  pub fn new(capacity: usize,
             allocator: Box<Allocator<T>>,
             reset: Box<Reset<T>>) -> Pool<T> {

    let mut items = Vec::<Box<T>>::with_capacity(capacity);
    for _ in 0..capacity {
      items.push((allocator)());
    }

    Pool {
      allocator,
      reset,
      items,
    }
  }

  pub fn get(&mut self) -> Option<Box<T>> {
    self.items.pop()
  }

  pub fn get_or_alloc(&mut self) -> Box<T> {
    self.items.pop().unwrap_or_else(|| (self.allocator)())
  }

  pub fn release(&mut self, mut item: Box<T>) {
    (self.reset)(&mut item);
    self.items.push(item);
  }
}
