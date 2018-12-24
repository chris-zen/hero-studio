use std::sync::{Arc, RwLock};

use crate::song::{
  clip::{Clip, ClipId},
  source::notes::NotesSource
};

pub struct Notes {
  source: Arc<RwLock<NotesSource>>,
  id: ClipId,
}

pub struct NotesClip {
  clip: Clip,

  notes: Notes,
}
