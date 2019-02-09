// use log::{debug};

use std::collections::HashSet;
use std::iter::FromIterator;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use portmidi::{DeviceInfo, MidiEvent, MidiMessage, OutputPort, PortMidi};

use super::{MidiDestination, MidiDriver, MidiDriverId, MidiEndpoint, MidiError, MidiResult};
use hero_studio_core::midi::{
  bus::{BusNode, BusNodeLock, NodeClass, NodeFeature},
  encoder::Encoder,
  messages::Message,
};
use hero_studio_core::time::ClockTime;

pub const ID: MidiDriverId = MidiDriverId("PortMIDI");

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
  fn id(&self) -> MidiDriverId {
    ID
  }

  fn get_host_time(&self) -> ClockTime {
    // TODO !!!
    ClockTime::zero()
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
          .filter(|device| device.is_output())
          .collect::<Vec<DeviceInfo>>()
          .into_iter()
          .enumerate()
          .map(|(index, device)| {
            Box::new(PortMidiDestination {
              name: device.name().clone(),
              default: index == 0,
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
  default: bool,
  context: Rc<PortMidi>,
  device: DeviceInfo,
}

// impl Drop for PortMidiDestination {
//   fn drop(&mut self) {
//     println!("PortMidiDestination::Drop");
//   }
// }

impl MidiEndpoint for PortMidiDestination {
  fn name(&self) -> String {
    self.name.clone()
  }
}

impl MidiDestination for PortMidiDestination {
  fn open(&self) -> MidiResult<BusNodeLock> {
    self
      .context
      .output_port(self.device.clone(), MIDI_BUF_LEN)
      .map_err(|err| MidiError::DestinationOpen {
        cause: format!("Device={:?}, Error={:?}", self.name, err),
      })
      .map(|port| {
        let features = if self.default {
          HashSet::from_iter(std::iter::once(NodeFeature::Default))
        } else {
          HashSet::new()
        };
        Arc::new(RwLock::new(OutputBusNode {
          name: self.name.clone(),
          features,
          port,
        })) as BusNodeLock
      })
  }
}

struct OutputBusNode {
  name: String,
  features: HashSet<NodeFeature>,
  port: OutputPort,
}

// impl Drop for OutputBusNode {
//   fn drop(&mut self) {
//     println!("OutputBusNode::Drop");
//   }
// }

impl BusNode for OutputBusNode {
  fn name(&self) -> &str {
    self.name.as_str()
  }

  fn class(&self) -> &NodeClass {
    &NodeClass::Destination
  }

  fn features(&self) -> &HashSet<NodeFeature> {
    &self.features
  }

  fn send_message(&mut self, time: ClockTime, msg: &Message) {
    // trace!(">>> {:?} {:?}", time, msg);
    let timestamp = (time.to_nanos() / 1000) as u32;
    let data_size = Encoder::data_size(msg);
    let mut data = Vec::with_capacity(data_size);
    unsafe { data.set_len(data_size) };
    let slice = data.as_mut_slice();
    Encoder::encode(msg, slice);
    match msg {
      Message::SysEx { data } => {
        self
          .port
          .write_sysex(timestamp, data.as_slice())
          .unwrap_or(());
      }
      _ => {
        let message = match data_size {
          1 => MidiMessage {
            status: slice[0],
            data1: 0,
            data2: 0,
          },
          2 => MidiMessage {
            status: slice[0],
            data1: slice[1],
            data2: 0,
          },
          _ => MidiMessage {
            status: slice[0],
            data1: slice[1],
            data2: slice[2],
          },
        };
        let event = MidiEvent { message, timestamp };
        self.port.write_event(event).unwrap_or(());
      }
    }
  }
}
