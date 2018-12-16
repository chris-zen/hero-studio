use std::collections::BTreeMap;

use crate::song::{
  clip::{
    ClipIndex,
    notes::NotesClip
  },
  io::{NotesSource, NotesSink}
};

pub struct MidiTrack {
  source: NotesSource,
  sink: NotesSink,

  clips: BTreeMap<ClipIndex, NotesClip>,
}
