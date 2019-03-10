mod portmidi;
pub use self::portmidi::ID as PORT_MIDI_ID;

#[cfg(all(target_os = "macos"))]
mod coremidi;

#[cfg(all(target_os = "macos"))]
pub use self::coremidi::ID as CORE_MIDI_ID;

use std::collections::HashMap;

use failure::Fail;
// use failure_derive;

use hero_studio_core::midi::bus::BusNodeLock;
use hero_studio_core::time::ClockTime;

#[derive(Debug, Fail)]
pub enum MidiError {
  #[fail(display = "Failed to initialise the MIDI driver: {}", cause)]
  Init { cause: String },

  #[fail(display = "Driver not found: {}", id)]
  DriverNotFound { id: String },

  #[fail(display = "Failed to open a destination: {}", cause)]
  DestinationOpen { cause: String },

  // #[fail(display = "Failed to open a source: {}", cause)]
  // SourceOpen { cause: String },
}

pub type MidiResult<T> = Result<T, MidiError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MidiDriverId(&'static str);

impl From<MidiDriverId> for String {
  fn from(item: MidiDriverId) -> Self {
    item.0.to_string()
  }
}

impl From<MidiDriverId> for &'static str {
  fn from(item: MidiDriverId) -> Self {
    item.0
  }
}

type MidiDriverFactory = Box<Fn(String) -> MidiResult<Box<dyn MidiDriver>>>;

pub struct Midi {
  drivers: HashMap<MidiDriverId, MidiDriverFactory>,
}

impl Midi {
  pub fn new() -> Midi {
    let mut drivers: HashMap<MidiDriverId, MidiDriverFactory> = HashMap::new();

    {
      let portmidi_factory = Box::new(|_app_name: String| {
        portmidi::PortMidiDriver::new().map(|driver| Box::new(driver) as Box<MidiDriver>)
      });
      drivers.insert(portmidi::ID, portmidi_factory);
    }

    if cfg!(all(target_os = "macos")) {
      let driver_factory = Box::new(|app_name: String| {
        coremidi::CoreMidi::new(app_name).map(|driver| Box::new(driver) as Box<MidiDriver>)
      });
      drivers.insert(coremidi::ID, driver_factory);
    }

    Midi { drivers }
  }

  pub fn drivers(&self) -> Vec<MidiDriverId> {
    self.drivers.keys().map(|id| *id).collect()
  }

  pub fn driver<T>(&self, id: MidiDriverId, app_name: T) -> MidiResult<Box<dyn MidiDriver>>
  where
    T: Into<String>,
  {
    self
      .drivers
      .get(&id)
      .map(|driver_factory| driver_factory(app_name.into()))
      .unwrap_or(Err(MidiError::DriverNotFound {
        id: String::from(id),
      }))
  }
}

pub trait MidiDriver {
  fn id(&self) -> MidiDriverId;

  fn get_host_time(&self) -> ClockTime;

  // fn sources(&self) -> Vec<Box<dyn MidiSource>>;
  fn destinations(&self) -> Vec<Box<dyn MidiDestination>>;

  // fn create_virtual_output<T>(&self, name: T) -> dyn MidiOutput where T: Into<String>;
}

pub trait MidiEndpoint {
  fn name(&self) -> String;
}

pub trait MidiDestination: MidiEndpoint {
  fn open(&self) -> MidiResult<BusNodeLock>;
}
