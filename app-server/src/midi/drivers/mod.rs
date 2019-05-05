mod portmidi;
pub use self::portmidi::ID as PORT_MIDI_ID;

#[cfg(target_os = "macos")]
mod coremidi;

#[cfg(target_os = "macos")]
pub use self::coremidi::ID as CORE_MIDI_ID;

#[cfg(not(target_os = "macos"))]
const DEFAULT_ID: &'static str = PORT_MIDI_ID;

#[cfg(target_os = "macos")]
const DEFAULT_ID: &str = CORE_MIDI_ID;

use std::collections::HashMap;

use failure::Fail;

use hero_studio_core::midi::buffer::Buffer;
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

type MidiDriverFactory = Box<Fn(String) -> MidiResult<Box<dyn MidiDriver>>>;

pub struct MidiDrivers {
  drivers: HashMap<String, MidiDriverFactory>,
}

impl MidiDrivers {
  pub fn new() -> MidiDrivers {
    let mut drivers: HashMap<String, MidiDriverFactory> = HashMap::new();

    Self::add_platform_drivers(&mut drivers);

    Self::add_common_drivers(&mut drivers);

    MidiDrivers { drivers }
  }

  #[cfg(target_os = "macos")]
  fn add_platform_drivers(drivers: &mut HashMap<String, MidiDriverFactory>) {
    let coremidi_factory = Box::new(|app_name: String| {
      coremidi::CoreMidi::new(app_name).map(|driver| Box::new(driver) as Box<MidiDriver>)
    });
    drivers.insert(coremidi::ID.to_string(), coremidi_factory);
  }

  #[cfg(not(target_os = "macos"))]
  fn add_platform_drivers(drivers: &mut HashMap<String, MidiDriverFactory>) {}

  fn add_common_drivers(drivers: &mut HashMap<String, MidiDriverFactory>) {
    let portmidi_factory = Box::new(|_app_name: String| {
      portmidi::PortMidiDriver::new().map(|driver| Box::new(driver) as Box<MidiDriver>)
    });
    drivers.insert(portmidi::ID.to_string(), portmidi_factory);
  }

  #[allow(dead_code)]
  pub fn drivers(&self) -> Vec<&String> {
    self.drivers.keys().collect()
  }

  pub fn driver<A, B>(&self, id: A, app_name: B) -> MidiResult<Box<dyn MidiDriver>>
  where
    A: Into<String>,
    B: Into<String>,
  {
    let id = id.into();
    self
      .drivers
      .get(&id)
      .map(|driver_factory| driver_factory(app_name.into()))
      .unwrap_or_else(|| Err(MidiError::DriverNotFound { id }))
  }

  pub fn default<T>(&self, app_name: T) -> MidiResult<Box<dyn MidiDriver>>
  where
    T: Into<String>,
  {
    self.driver(DEFAULT_ID, app_name.into())
  }
}

pub trait MidiDriver {
  fn id(&self) -> &str;

  // fn sources(&self) -> Vec<Box<dyn MidiSource>>;
  fn destinations(&self) -> Vec<Box<dyn MidiDestination>>;

  // fn create_virtual_output<T>(&self, name: T) -> dyn MidiOutput where T: Into<String>;
}

pub trait MidiEndpoint {
  fn name(&self) -> &str;
}

pub trait MidiDestination: MidiEndpoint {
  fn open(&self) -> MidiResult<Box<dyn MidiOutput>>;
}

pub trait MidiOutput: MidiEndpoint {
  fn send(&mut self, base_time: ClockTime, buffer: &Buffer);
}
