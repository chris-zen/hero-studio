use std::collections::BTreeMap;

use crate::song::{
  clip::{
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
