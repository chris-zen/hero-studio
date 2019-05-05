pub mod audio;
pub mod instrument;
pub mod midi;

use crate::color::Color;

use crate::song::{
  clips::Clip,
  track::{audio::AudioTrack, instrument::InstrumentTrack, midi::MidiTrack},
};

use crate::time::TicksTime;

pub enum TrackMedia {
  Midi(MidiTrack),
  Audio(AudioTrack),
  Instrument(InstrumentTrack),
}

pub struct Track {
  pub name: String,
  pub color: Color,
  pub mute: bool,
  pub solo: bool,
  pub rec: bool,

  pub volume: f64,
  pub pan: f64,

  pub media: TrackMedia,

  clips: Vec<Clip>,
}

impl Track {
  pub fn new<T>(name: T, color: Color, media: TrackMedia) -> Track
  where
    T: Into<String>,
  {
    Track {
      name: name.into(),
      color,
      mute: false,
      solo: false,
      rec: false,
      volume: 1.0,
      pan: 0.0,
      media,
      clips: Vec::new(),
    }
  }

  pub fn clips_in_range(&self, start: TicksTime, until: TicksTime) -> impl Iterator<Item = &Clip> {
    // TODO use an Interval Tree (http://www.davismol.net/2016/02/07/data-structures-augmented-interval-tree-to-search-for-interval-overlapping/)
    self.clips.iter().filter(move |clip| {
      let end = clip.start + clip.length;
      clip.start < until && end >= start
    })
  }
}
