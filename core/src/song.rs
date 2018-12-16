pub mod time;

pub mod io;
pub mod source;
pub mod clip;
pub mod track;

use std::collections::BTreeMap;

use self::time::{Signature, Tempo, TicksTime};

use self::track::Track;

pub type SceneIndex = usize;

pub struct Scene {
  name: String
}

pub struct Song {
  name: String,

  tempo: Tempo,
  signature: Signature,

  tracks: Vec<Track>,

  scenes: BTreeMap<SceneIndex, Scene>
}
