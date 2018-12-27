use std::collections::BTreeMap;

use crate::song::{
  clips::{pianoroll::NotesClip, ClipIndex},
  io::{NotesSink, NotesSource},
};

pub struct MidiTrack {
  source: NotesSource,
  sink: NotesSink,

  clips: BTreeMap<ClipIndex, NotesClip>,
}

impl MidiTrack {}
