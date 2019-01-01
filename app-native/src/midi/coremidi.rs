use super::{MidiDriver, MidiOutput};
use hero_studio_core::time::{clock, ClockTime};

#[derive(Clone)]
pub struct CoreMidi {
  name: String,
}

impl CoreMidi {
  pub fn new() -> CoreMidi {
    CoreMidi {
      name: "Core MIDI".to_string(),
    }
  }
}

impl MidiDriver for CoreMidi {
  fn get_host_time(&self) -> ClockTime {
    let host_time = unsafe { external::AudioGetCurrentHostTime() };
    let nanos = unsafe { external::AudioConvertHostTimeToNanos(host_time) };
    ClockTime::new(
      ((nanos as u128 * clock::NANOS_PER_SECOND as u128) / clock::UNITS_PER_SECOND as u128) as u64,
    )
  }

  // fn create_virtual_output<T>(&self, name: T) -> dyn MidiOutput where T: Into<String> {

  // }
}

pub struct CoreMidiOutput {
  name: String,
}

impl MidiOutput for CoreMidiOutput {
  fn name(&self) -> String {
    self.name.clone()
  }
}

mod external {
  #[link(name = "CoreAudio", kind = "framework")]
  extern "C" {
    pub fn AudioConvertHostTimeToNanos(inHostTime: u64) -> u64;
    pub fn AudioGetCurrentHostTime() -> u64;
  }
}
