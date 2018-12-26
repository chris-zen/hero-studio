use std::sync::{Arc, RwLock};

use crate::song::{
  clips::{Clip, ClipId},
  source::audio::AudioDataSource
};


pub struct AudioData {
  source: Arc<RwLock<AudioDataSource>>,
  id: ClipId,
}

pub struct AudioClip {
  clip: Clip,

  data: AudioData,
}
