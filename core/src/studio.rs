use std::fmt;

use crate::config::{Config, ConfigLock};
use crate::midi::bus::MidiBusLock;
use crate::song::Song;

pub type Seconds = f64;

#[derive(Debug, Clone, Copy)]
pub struct ProcessingTime {
  pub current: Seconds,
  pub input: Seconds,
  pub output: Seconds,
}

impl ProcessingTime {
  pub fn new(current: Seconds, input: Seconds, output: Seconds) -> ProcessingTime {
    ProcessingTime {
      current,
      input,
      output,
    }
  }
}

pub struct Studio {
  config: ConfigLock,
  midi_bus: MidiBusLock,
  song: Song,
}

unsafe impl<'a> Send for Studio {}
use std::rc::Rc;
impl Studio {
  pub fn new(config: ConfigLock, midi_bus: MidiBusLock) -> Studio {
    let song = Song::new("untitled", config.clone(), midi_bus.clone());

    // TODO Create virtual ports according to the configuration ??? not sure if here or at main

    Studio {
      config,
      midi_bus,
      song,
    }
  }

  pub fn song(&self) -> &Song {
    &self.song
  }

  pub fn song_mut(&mut self) -> &mut Song {
    &mut self.song
  }

  pub fn play(&mut self, restart: bool) {
    self.song.play(restart);
  }

  pub fn midi_handler(&mut self) {}

  pub fn audio_handler(
    &mut self,
    time: ProcessingTime,
    frames: usize,
    _in_buffer: &[f32],
    _out_buffer: &mut [f32],
  ) {
    // retrieve midi events from the armed track's input port
    // retrieve midi events from the tracks from time.output
    // schedule midi events to the output ports
    // process the audio for time.output taking into consideration the midi events

    self.song.process(time.output, frames as u32);
  }
}

impl fmt::Debug for Studio {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Studio({:?})", self.song.get_name())
  }
}
