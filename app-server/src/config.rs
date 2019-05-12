use failure;
use failure::Error;

use serde_derive::Deserialize;

use std::fs::File;
use std::io::Read;

#[serde(default)]
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
  pub midi: Midi,
  pub websocket: WebSocket,
}

impl Default for Config {
  fn default() -> Config {
    Config {
      midi: Midi::default(),
      websocket: WebSocket::default(),
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

  #[allow(dead_code)]
  pub fn from_str<'a, T>(content: T) -> Result<Config, Error>
  where
    T: Into<&'a str>,
  {
    let config: Config = toml::from_str(content.into())?;
    Ok(config)
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct PoolWithItemCapacity {
  pub pool_capacity: usize,
  pub item_capacity: usize,
}

#[serde(default)]
#[derive(Deserialize, Debug, Clone)]
pub struct Midi {
  pub buffer_pool: PoolWithItemCapacity,
  pub io_vec_pool: PoolWithItemCapacity,
}

impl Default for Midi {
  fn default() -> Midi {
    Midi {
      buffer_pool: PoolWithItemCapacity {
        pool_capacity: 256,
        item_capacity: 1024,
      },
      io_vec_pool: PoolWithItemCapacity {
        pool_capacity: 128,
        item_capacity: 1024,
      },
    }
  }
}

#[serde(default)]
#[derive(Deserialize, Debug, Clone)]
pub struct WebSocket {
  pub port: u16,
}

impl Default for WebSocket {
  fn default() -> WebSocket {
    WebSocket { port: 3001 }
  }
}
