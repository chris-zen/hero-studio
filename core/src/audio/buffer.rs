use std::ops::{Deref, DerefMut};

use crate::pool::Pool;

pub const MAX_AUDIO_BUFFER_SIZE: usize = 2 * 4 * 1024;

// pub type BufferData = [f32; MAX_AUDIO_BUFFER_SIZE];
pub type BufferData = Vec<f32>;

pub struct Buffer(BufferData);

impl Buffer {
  pub fn new() -> Buffer {
    // Buffer([0.0; MAX_AUDIO_BUFFER_SIZE])
    let mut data = Vec::new();
    unsafe { data.set_len(MAX_AUDIO_BUFFER_SIZE) }
    Buffer(data)
  }

  pub fn with_capacity(capacity: usize) -> Buffer {
    let mut data = Vec::with_capacity(capacity);
    unsafe { data.set_len(capacity) }
    Buffer(data)
  }

  pub fn slice(&self, size: usize) -> &[f32] {
    &self.0[0..size]
  }

  pub fn slice_mut(&mut self, size: usize) -> &mut [f32] {
    &mut self.0[0..size]
  }
}

impl Deref for Buffer {
  type Target = BufferData;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Buffer {
  fn deref_mut(&mut self) -> &mut BufferData {
    &mut self.0
  }
}

pub fn new_buffer_pool(pool_capacity: usize, buffer_capacity: usize) -> Pool<Buffer> {
  let allocator = Box::new(move || Box::new(Buffer::with_capacity(buffer_capacity)));
  let reset = Box::new(|_item: &mut Buffer| {});
  Pool::new(pool_capacity, allocator, reset)
}
