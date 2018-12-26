use std::collections::BTreeMap;

use crate::song::{
  clips::{
    ClipIndex,
    pianoroll::NotesClip
  },
  io::{AudioSink, NotesSource}
};

pub struct InstrumentTrack {
  source: NotesSource,
  sink: AudioSink,

  clips: BTreeMap<ClipIndex, NotesClip>,
}
