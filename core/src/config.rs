use failure;
use failure::Error;

use serde;
use serde_derive::Deserialize;

use std::fs::File;
use std::io::Read;

#[derive(Deserialize, Debug, Clone)]
pub enum AudioPort {
  #[serde(rename = "none")]
  None,
  #[serde(rename = "default")]
  SystemDefault,
  #[serde(rename = "name")]
  ByName(String),
}

#[serde(default)]
#[derive(Deserialize, Debug, Clone)]
pub struct Audio {
  pub input_port: AudioPort,
  pub output_port: AudioPort,
  pub sample_rate: u32,
  pub frames: u16,
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
pub struct Config {
  pub audio: Audio,
}

impl Default for Config {
  fn default() -> Config {
    Config {
      audio: Audio::default(),
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
