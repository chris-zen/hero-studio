pub mod io;
pub mod source;
pub mod clips;
pub mod track;
pub mod transport;

use crate::time::{
  SampleRate,
};

use self::{
  transport::{
    Transport,
    Segment
  },
  track::{
    Track,
    TrackMedia
  },
};


pub struct Song {
  name: String,

  transport: Transport,

  tracks: Vec<Track>
}

impl Song {
  pub fn new<T>(name: T, sample_rate: SampleRate) -> Song where T: Into<String> {
    Song {
      name: name.into(),

      transport: Transport::new(sample_rate),

      tracks: Vec::new()
    }
  }

  pub fn set_name<T>(&mut self, name: T) where T: Into<String> {
    self.name = name.into();
  }

  pub fn get_name(&self) -> &str {
    self.name.as_str()
  }

  pub fn get_transport_mut(&mut self) -> &mut Transport {
    &mut self.transport
  }

  pub fn play(&mut self, restart: bool) -> bool {
    self.transport.play(restart);
    self.transport.is_playing()
  }

  pub fn stop(&mut self) {
    self.transport.stop();
  }

  ///! Process song play
  pub fn process(&mut self, samples: u32) {
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
    println!("=> Segment [{:06?}, {:06?}) [{:010?}, {:010?})",
             u64::from(segment.start_ticks), u64::from(segment.end_ticks),
             segment.start_time.units(), segment.end_time.units());

    for track in self.tracks.iter_mut() {
      // let clips = track.clips_in_range(start_ticks, end_ticks);
      match &track.media {
        TrackMedia::Midi(midi_track) => {
          // prepare buffer for midi_track.sink

        },
        TrackMedia::Audio(_audio_track) => {

        },
        TrackMedia::Instrument(_instrument_track) => {

        }
      }
    }
  }
}
