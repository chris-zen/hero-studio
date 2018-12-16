use std::rc::Rc;

use crate::song::{
  clip::{Clip, ClipId},
  source::notes::NotesSource
};

pub struct Notes {
  source: Rc<NotesSource>,
  id: ClipId,
}

pub struct NotesClip {
  clip: Clip,

  notes: Notes,
}
