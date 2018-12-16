pub mod audio;
pub mod notes;

use crate::song::time::{Signature, TicksTime};

pub type ClipId = u64;

pub type ClipIndex = usize;

pub struct Clip {
  name: String,
  signature: Signature,
  start: TicksTime,
  length: TicksTime,
}
