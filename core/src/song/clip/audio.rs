use std::rc::Rc;

use crate::song::{
  clip::{Clip, ClipId},
  source::audio::AudioDataSource
};


pub struct AudioData {
  source: Rc<AudioDataSource>,
  id: ClipId,
}

pub struct AudioClip {
  clip: Clip,

  data: AudioData,
}
