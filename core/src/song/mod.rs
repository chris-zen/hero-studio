use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

pub mod clips;
pub mod io;
pub mod source;
pub mod track;

use crate::config::Config;
use crate::metronome::Metronome;
use crate::midi::bus::MidiBusLock;
use crate::time::{BarsTime, SampleRate, Signature, TicksTime};
use crate::transport::{Segment, Transport};

use self::track::{Track, TrackMedia};

pub struct Song {
  name: String,
  tracks: Vec<Track>,
}

impl Song {
  pub fn new<T>(name: T, _config: &Config) -> Song
  where
    T: Into<String>,
  {
    Song {
      name: name.into(),
      tracks: Vec::new(),
    }
  }

  pub fn set_name<T>(&mut self, name: T)
  where
    T: Into<String>,
  {
    self.name = name.into();
  }

  pub fn get_name(&self) -> &str {
    self.name.as_str()
  }

  pub fn process_segment(&mut self, _segment: &Segment) {
    // println!(
    //   "=> Segment T [{:06?}, {:06?}) <{:06?}> C [{:010?}, {:010?}) <{:010?}> @ PT {:06?} PC {:010?}",
    //   u64::from(segment.start_ticks),
    //   u64::from(segment.end_ticks),
    //   u64::from(segment.segment_ticks),
    //   segment.start_time.units(),
    //   segment.end_time.units(),
    //   segment.segment_time.units(),
    //   u64::from(segment.play_ticks),
    //   segment.play_time.units()
    // );

    for track in self.tracks.iter_mut() {
      // let clips = track.clips_in_range(start_ticks, end_ticks);
      match &track.media {
        TrackMedia::Midi(_midi_track) => {
          // prepare buffer for midi_track.sink
        }
        TrackMedia::Audio(_audio_track) => {}
        TrackMedia::Instrument(_instrument_track) => {}
      }
    }
  }
}
