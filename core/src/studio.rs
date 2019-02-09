use std::fmt;

use crate::config::Config;
use crate::midi::bus::MidiBusLock;
use crate::song::Song;
use crate::time::ClockTime;

pub type Seconds = f64;

#[derive(Debug, Clone, Copy)]
pub struct AudioTime {
  pub current: ClockTime,
  pub input: ClockTime,
  pub output: ClockTime,
}

impl AudioTime {
  pub fn new(current: Seconds, input: Seconds, output: Seconds) -> AudioTime {
    AudioTime {
      current: ClockTime::from_seconds(current),
      input: ClockTime::from_seconds(input),
      output: ClockTime::from_seconds(output),
    }
  }
}

pub struct Studio {
  config: Config,
  midi_bus: MidiBusLock,
  song: Song,
}

unsafe impl Send for Studio {}

impl Studio {
  pub fn new(config: Config, midi_bus: MidiBusLock) -> Studio {
    let song = Song::new("untitled", &config, midi_bus.clone());

    Studio {
      config,
      midi_bus,
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

  pub fn play(&mut self, restart: bool) {
    self.song.play(restart);
  }

  pub fn audio_handler(
    &mut self,
    audio_time: AudioTime,
    frames: usize,
    _in_buffer: &[f32],
    _out_buffer: &mut [f32],
  ) {
    // retrieve midi events from the armed track's input port
    // retrieve midi events from the tracks from time.output
    // schedule midi events to the output ports
    // process the audio for time.output taking into consideration the midi events

    self.song.process(audio_time, frames as u32);
  }
}

impl fmt::Debug for Studio {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Studio({:?})", self.song.get_name())
  }
}
