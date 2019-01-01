#[cfg(all(target_os = "macos"))]
mod coremidi;

use std::sync::{Arc, Mutex};

use hero_studio_core::time::ClockTime;

pub trait MidiDriver {
  // fn inputs(&self) -> Iterator<Item=dyn MidiInput>;
  // fn outputs(&self) -> Iterator<Item=dyn MidiOutput>;

  fn get_host_time(&self) -> ClockTime;

  // fn create_virtual_output<T>(&self, name: T) -> dyn MidiOutput where T: Into<String>;
}

pub trait MidiInput {}

pub trait MidiOutput {
  fn name(&self) -> String;

  // fn send(&self);
}

pub struct Midi {
  drivers: Vec<Arc<Mutex<dyn MidiDriver>>>,
}

impl Midi {
  pub fn init() -> Midi {
    Midi {
      drivers: vec![#[cfg(all(target_os = "macos"))]
      Arc::new(Mutex::new(coremidi::CoreMidi::new()))],
    }
  }

  pub fn current(&self) -> Arc<Mutex<dyn MidiDriver>> {
    Arc::clone(&self.drivers[0])
  }
}
