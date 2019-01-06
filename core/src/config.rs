use failure;
use failure::Error;

use serde;
use serde_derive::Deserialize;

use std::fs::File;
use std::io::Read;
use std::sync::{Arc, RwLock};

pub type ConfigLock = Arc<RwLock<Config>>;

#[serde(default)]
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
  pub audio: Audio,
  pub midi: Midi,
  pub metronome: Metronome,
}

impl Default for Config {
  fn default() -> Config {
    Config {
      audio: Audio::default(),
      midi: Midi::default(),
      metronome: Metronome::default(),
    }
  }
}

impl Config {
  pub fn from_file<'a, T>(path: T) -> Result<Config, Error>
  where
    T: Into<&'a str>,
  {
    let mut content = String::new();
    let path_str = path.into();
    let mut file = File::open(path_str)?;
    file.read_to_string(&mut content)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
  }

  pub fn from_str<'a, T>(content: T) -> Result<Config, Error>
  where
    T: Into<&'a str>,
  {
    let config: Config = toml::from_str(content.into())?;
    Ok(config)
  }
}

#[serde(default)]
#[derive(Deserialize, Debug, Clone)]
pub struct Audio {
  pub input_port: AudioPort,
  pub output_port: AudioPort,
  pub sample_rate: u32,
  pub frames: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub enum AudioPort {
  #[serde(rename = "none")]
  None,
  #[serde(rename = "default")]
  SystemDefault,
  #[serde(rename = "name")]
  ByName(String),
}

impl Default for Audio {
  fn default() -> Audio {
    Audio {
      input_port: AudioPort::SystemDefault,
      output_port: AudioPort::SystemDefault,
      sample_rate: 44100,
      frames: 512,
    }
  }
}

#[serde(default)]
#[derive(Deserialize, Debug, Clone)]
pub struct Midi {
  pub default_input: MidiPort,
  pub default_output: MidiPort,

  pub virtual_ports: Vec<MidiVirtualPort>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum MidiPort {
  #[serde(rename = "none")]
  None,
  #[serde(rename = "all")]
  All,
  #[serde(rename = "default")]
  SystemDefault,
  #[serde(rename = "name")]
  ByName(String),
}

#[derive(Deserialize, Debug, Clone)]
pub struct MidiVirtualPort {
  pub name: String,
  #[serde(default)]
  pub sync_delay_ms: u32,
}

impl Default for Midi {
  fn default() -> Midi {
    Midi {
      default_input: MidiPort::All,
      default_output: MidiPort::SystemDefault,
      virtual_ports: Vec::new(),
    }
  }
}

#[serde(default)]
#[derive(Deserialize, Debug, Clone)]
pub struct Metronome {
  pub enabled: bool,
  pub port: MidiPort,
  pub bar_note: MetronomeNote,
  pub beat_note: MetronomeNote,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MetronomeNote {
  #[serde(default = "default_metronome_note_channel")]
  pub channel: u8,
  pub key: u8,
  #[serde(default = "default_metronome_note_velocity")]
  pub velocity: u8,
  #[serde(default = "default_metronome_note_duration")]
  pub duration: u8,
}

fn default_metronome_note_channel() -> u8 {
  0
}

fn default_metronome_note_velocity() -> u8 {
  127
}

fn default_metronome_note_duration() -> u8 {
  16 // 1/16 (a sixteenth)
}

impl Default for Metronome {
  fn default() -> Metronome {
    Metronome {
      enabled: true,
      port: MidiPort::SystemDefault,
      bar_note: MetronomeNote {
        channel: 0,
        key: 84,
        velocity: 127,
        duration: 16,
      },
      beat_note: MetronomeNote {
        channel: 0,
        key: 77,
        velocity: 120,
        duration: 16,
      },
    }
  }
}
