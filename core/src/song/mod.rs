pub mod clips;
pub mod io;
pub mod source;
pub mod track;
pub mod transport;

use crate::time::{SampleRate, Signature, TicksTime, BarsTime};

use self::{
  track::{Track, TrackMedia},
  transport::{Segment, Transport},
};

pub struct Metronome {
  signature: Signature,
  period: TicksTime,
  ticks: TicksTime
}

impl Metronome {
  pub fn new(transport: &Transport) -> Metronome {
    let signature = *transport.get_signature();
    let rythm = signature.get_num_beats() as u64;
    let period = BarsTime::from_bars(1).to_ticks(signature) / rythm;
    Metronome {
      signature: signature,
      period: period,
      ticks: TicksTime::zero()
    }
  }

  pub fn update_signature(&mut self, signature: Signature) {
    self.signature = signature;
    // TODO
  }

  pub fn update_position(&mut self, position: TicksTime) {
    // TODO
  }

  pub fn process_segment(&mut self, segment: &Segment) {
    let mut segment_length = segment.end_ticks - segment.start_ticks;
    if self.ticks >= segment_length {
      self.ticks -= segment_length;
    }
    else {
      segment_length -= self.ticks;
      while segment_length > TicksTime::zero() {
        let tick_time = segment.end_ticks - segment_length;
        println!("Metronome: tick {:?}", BarsTime::from_ticks(tick_time, self.signature));
        if self.period < segment_length {
          segment_length -= self.period;
          self.ticks = self.period;
        }
        else {
          self.ticks = self.period - segment_length;
          segment_length = TicksTime::zero();
        }
      }
    }
  }
}

pub struct Song {
  name: String,

  transport: Transport,

  metronome: Metronome,

  tracks: Vec<Track>,
}

impl Song {
  pub fn new<T>(name: T, sample_rate: SampleRate) -> Song
  where
    T: Into<String>,
  {
    let transport = Transport::new(sample_rate);
    let metronome = Metronome::new(&transport);

    Song {
      name: name.into(),

      transport: transport,

      metronome: metronome,

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

    self.metronome.process_segment(segment);

    for track in self.tracks.iter_mut() {
      // let clips = track.clips_in_range(start_ticks, end_ticks);
      match &track.media {
        TrackMedia::Midi(midi_track) => {
          // prepare buffer for midi_track.sink

        }
        TrackMedia::Audio(_audio_track) => {}
        TrackMedia::Instrument(_instrument_track) => {}
      }
    }
  }
}
