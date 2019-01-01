use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

pub mod clips;
pub mod io;
pub mod metronome;
pub mod source;
pub mod track;
pub mod transport;

use crate::config::Config;
use crate::midi::bus::MidiBusLock;
use crate::time::{BarsTime, SampleRate, Signature, TicksTime};

use self::{
  metronome::Metronome,
  track::{Track, TrackMedia},
  transport::{Segment, Transport},
};

pub struct Song<'a> {
  name: String,

  config: &'a Config,

  transport: Transport,

  metronome: Metronome<'a>,

  tracks: Vec<Track>,

  midi_bus: MidiBusLock,
}

impl<'a> Song<'a> {
  pub fn new<T>(name: T, config: &'a Config, midi_bus: MidiBusLock) -> Song
  where
    T: Into<String>,
  {
    let transport = Transport::new(config.audio.sample_rate);

    let metronome = Metronome::new(&config.metronome, &transport, Arc::clone(&midi_bus));

    Song {
      name: name.into(),
      config: &config,
      transport: transport,
      metronome,
      tracks: Vec::new(),
      midi_bus,
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

  pub fn set_loop_enabled(&mut self, enabled: bool) {
    self.transport.set_loop_enabled(enabled);
  }

  pub fn set_loop_start(&mut self, position: BarsTime) {
    self.transport.set_loop_start(position);
  }

  pub fn set_loop_end(&mut self, position: BarsTime) {
    self.transport.set_loop_end(position)
  }

  pub fn play(&mut self, restart: bool) -> bool {
    self.transport.play(restart);
    self.transport.is_playing()
  }

  pub fn stop(&mut self) {
    self.transport.stop();
  }

  ///! Process song play
  pub fn process(&mut self, _output_host_time: f64, samples: u32) {
    if self.transport.is_playing() {
      let mut segments = self.transport.segments_iterator(samples);
      while let Some(segment) = segments.next(&self.transport) {
        self.process_segment(&segment);
      }
      self.transport.update_from_segments(&segments);
    }

    // TODO some devices might need to keep track of time even when not playing
  }

  fn process_segment(&mut self, segment: &Segment) {
    println!(
      "=> Segment [{:06?}, {:06?}) [{:010?}, {:010?})",
      u64::from(segment.start_ticks),
      u64::from(segment.end_ticks),
      segment.start_time.units(),
      segment.end_time.units()
    );

    self.metronome.process_segment(segment, &self.transport);

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
