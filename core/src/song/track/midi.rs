use std::collections::BTreeMap;

use crate::song::{
  clips::{
    ClipIndex,
    pianoroll::NotesClip
  },
  io::{NotesSource, NotesSink}
};

pub struct MidiTrack {
  source: NotesSource,
  sink: NotesSink,

  clips: BTreeMap<ClipIndex, NotesClip>
}

impl MidiTrack {

}