pub mod buffer;
pub use buffer::{new_buffer_pool, Buffer};

use crate::time::ClockTime;

pub struct AudioInput<'a> {
  pub time: ClockTime,
  pub channels: usize,
  pub buffer: &'a [f32],
}

impl<'a> AudioInput<'a> {
  pub fn new(time: ClockTime, channels: usize, buffer: &'a [f32]) -> Self {
    AudioInput {
      time,
      channels,
      buffer,
    }
  }
}

pub struct AudioOutput<'a> {
  pub time: ClockTime,
  pub channels: usize,
  pub buffer: &'a mut [f32],
}

impl<'a> AudioOutput<'a> {
  pub fn new(time: ClockTime, channels: usize, buffer: &'a mut [f32]) -> Self {
    AudioOutput {
      time,
      channels,
      buffer,
    }
  }
}
