use crate::config::Config;
use crate::song::Song;

pub type Seconds = f64;

#[derive(Debug, Clone, Copy)]
pub struct ProcessingTime {
  pub current: Seconds,
  pub input: Seconds,
  pub output: Seconds
}

impl ProcessingTime {
  pub fn new(current: Seconds, input: Seconds, output: Seconds) -> ProcessingTime {
    ProcessingTime {
      current, input, output
    }
  }
}

pub struct Studio {
  config: Config,
  song: Song
}

unsafe impl Send for Studio {}

impl Studio {
  pub fn new(config: Config) -> Studio {
    let sample_rate = config.audio.sample_rate;
    Studio {
      config,
      song: Song::new("untitled", sample_rate)
    }
  }

  pub fn config(&self) -> &Config {
    &self.config
  }

  pub  fn song(&self) -> &Song {
    &self.song
  }

  pub  fn song_mut(&mut self) -> &mut Song {
    &mut self.song
  }

  pub fn play(&mut self, restart: bool) {
    self.song.play(restart);
  }

  pub fn midi_handler(&mut self) {

  }

  pub fn audio_handler(&mut self,
                       _time: ProcessingTime,
                       frames: usize,
                       _in_buffer: &[f32],
                       _out_buffer: &mut [f32]) {

    // retrieve midi events from the armed track's input port
    // retrieve midi events from the tracks from time.output
    // schedule midi events to the output ports
    // process the audio for time.output taking into consideration the midi events

    self.song.process(frames as u32);
  }
}
