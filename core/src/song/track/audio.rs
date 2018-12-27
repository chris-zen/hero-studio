use std::collections::BTreeMap;

use crate::song::{
  clips::{audio::AudioClip, ClipIndex},
  io::{AudioSink, AudioSource},
};

pub struct AudioTrack {
  source: AudioSource,
  sink: AudioSink,

  clips: BTreeMap<ClipIndex, AudioClip>,
}
