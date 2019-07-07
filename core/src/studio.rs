use std::fmt;
use std::ops::{Deref, DerefMut};

use crate::audio;
use crate::audio::{AudioInput, AudioOutput};
use crate::config::{Config, MidiPort};
use crate::metronome::Metronome;
use crate::midi;
use crate::midi::buffer::EventIo;
use crate::midi::io::{MidiInput, MidiOutput};
use crate::pool::Pool;
use crate::song::Song;
use crate::time::{BarsTime, ClockTime};
use crate::transport::{Segment, Transport};
use crate::midi::Buffer;

const MIDI_BUFFER_CAPACITY: usize = 256 * 1024;

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
  midi_buffer: Vec<EventIo>,
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

    let midi_buffer = Vec::with_capacity(MIDI_BUFFER_CAPACITY);

    Studio {
      config,
      transport,
      metronome,
      song,
      midi_buffer,
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

  #[allow(clippy::too_many_arguments)]
  pub fn process<MidiIn, MidiOut>(
    &mut self,
    audio_frames: usize,
    _audio_input: &AudioInput,
    audio_output: &mut AudioOutput,
    midi_input: &mut MidiIn,
    midi_output: &mut MidiOut,
  ) where
    MidiIn: MidiInput,
    MidiOut: MidiOutput,
  {
    self.capture_midi_in(midi_input);

    if self.transport.is_playing() {
      let master_clock = audio_output.time;

      let mut segments = self
        .transport
        .segments_iterator(master_clock, audio_frames as u32);

      while let Some(segment) = segments.next(&self.transport) {
        self.metronome.process_segment(&segment, midi_output);
        self.song.process_segment(&segment);
      }

      self.transport.update_from_segments(&segments);

      fill_with_zero(audio_output.buffer);

//      for i in 0..audio_frames {
//        let v = i as f32 / audio_frames as f32;
//        let u = i * audio_input.channels;
//        let j = i * audio_output.channels;
//        for k in 0..audio_output.channels {
//          audio_output.buffer[j + k] = audio_input.buffer[u] + v * 0.20;
//        }
//      }
    } else {
      fill_with_zero(audio_output.buffer);
    }
  }

  fn capture_midi_in<MidiIn>(&mut self, midi_input: &mut MidiIn) where MidiIn: MidiInput {
    self.midi_buffer.clear();
    while let Some(event_io) = midi_input.pop() {
//      println!("{:?}", event_io);
      self.midi_buffer.push(event_io);
      if self.midi_buffer.len() == MIDI_BUFFER_CAPACITY {
        break;
      }
    }
  }
}

impl fmt::Debug for Studio {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Studio({:?})", self.song.get_name())
  }
}
