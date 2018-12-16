use std::collections::BTreeMap;

use crate::song::{
  clip::{
    ClipIndex,
    notes::NotesClip
  },
  io::{AudioSink, NotesSource}
};

pub struct InstrumentTrack {
  source: NotesSource,
  sink: AudioSink,

  clips: BTreeMap<ClipIndex, NotesClip>,
}
