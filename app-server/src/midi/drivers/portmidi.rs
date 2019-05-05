// use log::{debug};

use std::rc::Rc;

use portmidi::{DeviceInfo, MidiEvent, MidiMessage, OutputPort, PortMidi};

use hero_studio_core::midi::buffer::Buffer;
use hero_studio_core::midi::{encoder::Encoder, messages::Message};
use hero_studio_core::time::ClockTime;

use super::{MidiDestination, MidiDriver, MidiEndpoint, MidiError, MidiOutput, MidiResult};

pub const ID: &str = "PortMIDI";

const MIDI_BUF_LEN: usize = 8 * 1024;

pub struct PortMidiDriver {
  context: Rc<PortMidi>,
}

// impl Drop for PortMidiDriver {
//   fn drop(&mut self) {
//     println!("PortMidiDriver::Drop");
//   }
// }

impl PortMidiDriver {
  pub fn new() -> MidiResult<PortMidiDriver> {
    PortMidi::new()
      .map_err(|err| MidiError::Init {
        cause: format!("{:?}", err),
      })
      .map(|context| PortMidiDriver {
        context: Rc::new(context),
      })
  }
}

impl MidiDriver for PortMidiDriver {
  fn id(&self) -> &str {
    ID
  }

  // fn sources(&self) -> Iterator<Item=dyn MidiEndpoint> {
  //   unimplemented!();
  // }

  fn destinations(&self) -> Vec<Box<dyn MidiDestination>> {
    self
      .context
      .devices()
      .into_iter()
      .flat_map(|devices| {
        devices
          .into_iter()
          .filter(DeviceInfo::is_output)
          .collect::<Vec<DeviceInfo>>()
          .into_iter()
          .map(|device| {
            Box::new(PortMidiDestination {
              name: device.name().clone(),
              context: Rc::clone(&self.context),
              device: device.clone(),
            }) as Box<MidiDestination>
          })
      })
      .collect()
  }

  // fn create_virtual_output<T>(&self, name: T) -> dyn MidiOutput where T: Into<String> {

  // }
}

pub struct PortMidiDestination {
  name: String,
  context: Rc<PortMidi>,
  device: DeviceInfo,
}

// impl Drop for PortMidiDestination {
//   fn drop(&mut self) {
//     println!("PortMidiDestination::Drop");
//   }
// }

impl MidiEndpoint for PortMidiDestination {
  fn name(&self) -> &str {
    self.name.as_str()
  }
}

impl MidiDestination for PortMidiDestination {
  fn open(&self) -> MidiResult<Box<dyn MidiOutput>> {
    self
      .context
      .output_port(self.device.clone(), MIDI_BUF_LEN)
      .map_err(|err| MidiError::DestinationOpen {
        cause: format!("Device={:?}, Error={:?}", self.name, err),
      })
      .map(|port| {
        Box::new(PortMidiOutput::new(
          self.name.clone(),
          self.context.clone(),
          port,
        )) as Box<MidiOutput>
      })
  }
}

const MESSAGE_BUFFER_CAPACITY: usize = 8;

struct PortMidiOutput {
  name: String,
  _context: Rc<PortMidi>,
  port: OutputPort,
  message_buffer: [u8; MESSAGE_BUFFER_CAPACITY],
}

// impl Drop for OutputBusNode {
//   fn drop(&mut self) {
//     println!("OutputBusNode::Drop");
//   }
// }

impl PortMidiOutput {
  fn new(name: String, context: Rc<PortMidi>, port: OutputPort) -> PortMidiOutput {
    PortMidiOutput {
      name,
      _context: context,
      port,
      message_buffer: [0; MESSAGE_BUFFER_CAPACITY],
    }
  }

  fn send_message(&mut self, time: ClockTime, msg: &Message) {
    // trace!(">>> {:?} {:?}", time, msg);
    let timestamp = (time.to_nanos() / 1000) as u32;
    let data_size = Encoder::data_size(msg);

    Encoder::encode(msg, &mut self.message_buffer);

    let message = match data_size {
      1 => MidiMessage {
        status: self.message_buffer[0],
        data1: 0,
        data2: 0,
      },
      2 => MidiMessage {
        status: self.message_buffer[0],
        data1: self.message_buffer[1],
        data2: 0,
      },
      3 => MidiMessage {
        status: self.message_buffer[0],
        data1: self.message_buffer[1],
        data2: self.message_buffer[2],
      },
      _ => unreachable!(),
    };

    let event = MidiEvent { message, timestamp };
    let _ = self.port.write_event(event);
  }

  // fn send_sysex_message(&mut self, time: ClockTime, msg: &[U7]) {
  //   // trace!(">>> {:?} {:?}", time, msg);
  //   let timestamp = (time.to_nanos() / 1000) as u32;
  //   let data_size = Encoder::sysex_data_size(msg);
  //   let mut data = Vec::with_capacity(data_size);
  //   unsafe { data.set_len(data_size) };
  //   let slice = data.as_mut_slice();
  //   Encoder::sysex_encode(msg, slice);
  //   self
  //     .port
  //     .write_sysex(timestamp, data.as_slice())
  //     .unwrap_or(());
  // }
}

impl MidiEndpoint for PortMidiOutput {
  fn name(&self) -> &str {
    self.name.as_str()
  }
}

impl MidiOutput for PortMidiOutput {
  fn send(&mut self, base_time: ClockTime, buffer: &Buffer) {
    for event in buffer.iter() {
      self.send_message(base_time + event.timestamp, &event.message)
    }
  }
}
