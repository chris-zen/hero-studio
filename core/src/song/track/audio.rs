use std::collections::BTreeMap;

use crate::song::{
  clips::{
    ClipIndex,
    audio::AudioClip
  },
  io::{AudioSource, AudioSink}
};

pub struct AudioTrack {
  source: AudioSource,
  sink: AudioSink,

  clips: BTreeMap<ClipIndex, AudioClip>,
}
