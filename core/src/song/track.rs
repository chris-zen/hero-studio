pub mod audio;
pub mod instrument;
pub mod notes;

use crate::color::Color;

use crate::song::track::{
  audio::AudioTrack,
  instrument::InstrumentTrack,
  notes::MidiTrack
};

pub enum TrackMedia {
  Midi(MidiTrack),
  Audio(AudioTrack),
  Instrument(InstrumentTrack),
}

pub struct Track {
  name: String,
  color: Color,
  mute: bool,
  solo: bool,
  rec: bool,

  volume: f64,
  pan: f64,

  media: TrackMedia,
}
