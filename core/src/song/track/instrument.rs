use std::collections::BTreeMap;

use crate::song::{
  clips::{pianoroll::NotesClip, ClipIndex},
  io::{AudioSink, NotesSource},
};

pub struct InstrumentTrack {
  source: NotesSource,
  sink: AudioSink,

  clips: BTreeMap<ClipIndex, NotesClip>,
}
