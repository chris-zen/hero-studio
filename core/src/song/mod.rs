pub mod io;
pub mod source;
pub mod clip;
pub mod track;

use std::collections::BTreeMap;

use crate::time::{
  Signature,
  Tempo,
  SampleRate,
  ClockTime,
  TicksTime,
  BarsTime,
  clock
};

use self::track::Track;

const DEFAULT_TEMPO: u16 = 120;
const DEFAULT_SIGNATURE_NUM_BEATS: u8 = 4;
const DEFAULT_SIGNATURE_NOTE_VALUE: u8 = 4;

pub struct Song {
  name: String,

  tempo: Tempo,
  signature: Signature,

  sample_rate: SampleRate,
  time_per_sample: ClockTime,
  time_error_per_sample: f64,
  time_error_accum: f64,

  playing: bool,
  start_time: ClockTime,
  current_time: ClockTime,
  next_time: ClockTime,

  loop_enabled: bool,
  loop_start_time: ClockTime,
  loop_end_time: ClockTime,

  tracks: Vec<Track>

  // TODO arrangement
}

impl Song {
  pub fn new<T>(name: T, sample_rate: SampleRate) -> Song where T: Into<String> {
    let mut song = Song {
      name: name.into(),

      tempo: Tempo::new(DEFAULT_TEMPO),
      signature: Signature::new(DEFAULT_SIGNATURE_NUM_BEATS, DEFAULT_SIGNATURE_NOTE_VALUE),

      sample_rate: sample_rate,
      time_per_sample: ClockTime::zero(),
      time_error_per_sample: 0.0,
      time_error_accum: 0.0,

      playing: false,
      start_time: ClockTime::zero(),
      current_time: ClockTime::zero(),
      next_time: ClockTime::zero(),

      loop_enabled: true,
      loop_start_time: ClockTime::zero(),
      loop_end_time: ClockTime::zero(),

      tracks: Vec::new()
    };
    song.update_timing_constants();
    song
  }

  pub fn set_name<T>(&mut self, name: T) where T: Into<String> {
    self.name = name.into();
  }

  pub fn get_name(&self) -> &str {
    self.name.as_str()
  }

  pub fn set_tempo(&mut self, tempo: Tempo) {
    self.tempo = tempo;
  }

  pub fn get_tempo(&self) -> &Tempo {
    &self.tempo
  }

  pub fn set_signature(&mut self, signature: Signature) {
    self.signature = signature;
  }

  pub fn get_signature(&self) -> &Signature {
    &self.signature
  }

  pub fn set_sample_rate(&mut self, sample_rate: SampleRate) {
    self.sample_rate = sample_rate;
    self.update_timing_constants();
  }

  pub fn get_sample_rate(&self) -> &SampleRate {
    &self.sample_rate
  }

  pub fn is_playing(&self) -> bool {
    self.playing
  }

  pub fn play(&mut self, restart: bool) -> bool {
    self.playing = !self.playing;
    if restart {
      self.current_time = self.start_time;
      self.next_time = self.current_time;
    }
    self.playing
  }

  pub fn stop(&mut self) {
    self.playing = false;
  }

  pub fn set_loop_enabled(&mut self, enabled: bool) {
    self.loop_enabled = enabled;
  }

  pub fn is_loop_enabled(&self) -> bool {
    self.loop_enabled
  }

  pub fn set_loop_start_time(&mut self, time: ClockTime) {
    self.loop_start_time = time;
  }

  pub fn get_loop_start_time(&self) -> ClockTime {
    self.loop_start_time
  }

  pub fn set_loop_end_time(&mut self, time: ClockTime) {
    self.loop_end_time = time;
  }

  pub fn get_loop_end_time(&self) -> ClockTime {
    self.loop_end_time
  }

  ///! Process song play
  pub fn process(&mut self, samples: u32) {
    if self.playing {
      self.current_time = self.next_time;

      let mut start_time = self.current_time;
      let mut remaining_time = self.time_per_sample * samples as u32;
      let total_error = self.time_error_accum + self.time_error_per_sample * samples as f64;
      if total_error >= 1.0 {
        let correction = total_error.round();
        self.time_error_accum = total_error - correction;
        remaining_time += ClockTime::new(correction as clock::UnitType);
        println!("[{:013?}] Err: {:?} Correction {:?}", self.current_time.units(), self.time_error_accum, correction);
      }
      else {
        self.time_error_accum = total_error;
        println!("[{:013?}] Err: {:?}", self.current_time.units(), self.time_error_accum);
      }

      while remaining_time > ClockTime::zero() {
        self.next_time += remaining_time;
        if self.loop_to_start(start_time, self.next_time) {
          self.process_fragment(start_time, self.loop_end_time);
          remaining_time = self.next_time - self.loop_end_time;
          start_time = self.loop_start_time;
          self.next_time = start_time;
        }
        else {
          self.process_fragment(start_time, self.next_time);
          remaining_time = ClockTime::zero();
        }
      }
    }

    // TODO some devices might need to keep track of time even when not playing
  }

  fn process_fragment(&mut self, start_time: ClockTime, end_time: ClockTime) {
    println!("=> Processing [{:013?}, {:013?})", start_time.units(), end_time.units());
  }

  ///! Update timing constants that change sporadically (ex. changes on sample rate, tempo, signature, ...)
  fn update_timing_constants(&mut self) {
    self.time_per_sample = ClockTime::from_samples(1, self.sample_rate);
    self.time_error_per_sample = ClockTime::error_per_sample(self.sample_rate);
    println!("Time error per sample = {:?} clock units", self.time_error_per_sample);
  }

  ///! Determine whether or not not to move the song position to the start of the loop
  fn loop_to_start(&self, prev_time: ClockTime, next_time: ClockTime) -> bool {
    self.loop_enabled &&
      prev_time < self.loop_end_time &&
      self.loop_end_time <= next_time
  }
}
