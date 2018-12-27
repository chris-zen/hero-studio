pub mod audio;
pub mod pianoroll;
pub mod stepper;

use crate::time::{Signature, TicksTime};

pub type ClipId = u64;

pub type ClipIndex = usize;

pub struct Clip {
  pub uuid: ClipId,
  pub name: String,
  pub signature: Signature,
  pub start: TicksTime,
  pub length: TicksTime,
}
