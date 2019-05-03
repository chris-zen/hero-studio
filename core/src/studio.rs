use std::fmt;
use std::ops::{Deref, DerefMut};

use crate::audio;
use crate::config::{Config, MidiPort};
use crate::metronome::Metronome;
use crate::midi;
use crate::pool::Pool;
use crate::song::Song;
use crate::time::{BarsTime, ClockTime};
use crate::transport::{Segment, Transport};

fn fill_with_zero(s: &mut [f32]) {
  for d in s {
    *d = 0.0;
  }
}

pub struct Studio {
  config: Config,
  transport: Transport,
  metronome: Metronome,
  song: Song,
}

unsafe impl Send for Studio {}

impl Studio {
  pub fn new(config: Config) -> Studio {
    let song = Song::new("untitled", &config);

    let sample_rate = config.audio.sample_rate;
    let transport = Transport::new(sample_rate);

    let metronome_config = config.metronome.clone();
    let signature = *transport.get_signature();
    let metronome = Metronome::new(metronome_config, signature);

    Studio {
      config,
      transport,
      metronome,
      song,
    }
  }

  pub fn config(&self) -> &Config {
    &self.config
  }

  pub fn song(&self) -> &Song {
    &self.song
  }

  pub fn song_mut(&mut self) -> &mut Song {
    &mut self.song
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

  pub fn process(
    &mut self,
    frames: usize,
    _input_time: ClockTime,
    _audio_input_channels: usize,
    _audio_input: &audio::Buffer,
    audio_output_channels: usize,
    _audio_output: &mut audio::Buffer,
    midi_buffer_pool: &mut Pool<midi::Buffer>,
    // TODO midi_input: &midi::IoVec,
    midi_output: &mut midi::BufferIoVec,
  ) {
    if self.transport.is_playing() {
      let mut metronome_buffer = midi_buffer_pool.get_or_alloc();

      let base_time = ClockTime::zero();
      let mut segments = self.transport.segments_iterator(base_time, frames as u32);
      while let Some(segment) = segments.next(&self.transport) {
        self
          .metronome
          .process_segment(&segment, &mut metronome_buffer);
        self.song.process_segment(&segment);
      }
      self.transport.update_from_segments(&segments);

      midi_output.push(midi::BufferIo {
        endpoint: self.metronome.endpoint(),
        buffer: Some(metronome_buffer),
      });

      let out = _audio_output.slice_mut(frames * audio_output_channels);
      fill_with_zero(out);

      for i in 0..frames {
        let v = i as f32 / frames as f32;
        let u = i * _audio_input_channels;
        let j = i * audio_output_channels;
        for k in 0..audio_output_channels {
          _audio_output[j + k] = _audio_input[u] + v * 0.20;
        }
      }
    } else {
      let out = _audio_output.slice_mut(frames * audio_output_channels);
      fill_with_zero(out);
    }
  }

  // pub fn audio_handler(
  //   &mut self,
  //   audio_time: AudioTime,
  //   frames: usize,
  //   _in_buffer: &[f32],
  //   _out_buffer: &mut [f32],
  // ) {
  //   // retrieve midi events from the armed track's input port
  //   // retrieve midi events from the tracks from time.output
  //   // schedule midi events to the output ports
  //   // process the audio for time.output taking into consideration the midi events

  //   if self.transport.is_playing() {
  //     let mut metronome_buffer = self.midi_buffer_pool.get_or_alloc();

  //     let master_clock = audio_time.output;
  //     let mut segments = self.transport.segments_iterator(master_clock, frames as u32);
  //     while let Some(segment) = segments.next(&self.transport) {

  //       self.metronome.process_segment(&segment, &mut metronome_buffer);
  //       self.song.process_segment(&segment);
  //     }
  //     self.transport.update_from_segments(&segments);

  //   }

  //   // TODO some devices might need to keep track of time even when not playing
  // }
}

impl fmt::Debug for Studio {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Studio({:?})", self.song.get_name())
  }
}
