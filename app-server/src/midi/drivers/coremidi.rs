use std::collections::HashSet;
use std::iter::FromIterator;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use coremidi::{Client, Destination, Destinations, OutputPort, PacketBuffer};

use super::{MidiDestination, MidiDriver, MidiEndpoint, MidiError, MidiResult};
use hero_studio_core::midi::{
  bus::{BusNode, BusNodeLock, NodeClass, NodeFeature},
  encoder::Encoder,
  messages::Message,
};
use hero_studio_core::time::ClockTime;

pub const ID: &'static str = "CoreMIDI";

pub struct CoreMidi {
  client: Rc<Client>,
}

impl CoreMidi {
  pub fn new<T>(app_name: T) -> MidiResult<CoreMidi>
  where
    T: Into<String>,
  {
    Client::new(app_name.into().as_str())
      .map_err(|status| MidiError::Init {
        cause: format!("OSStatus={:?}", status),
      })
      .map(|client| CoreMidi {
        client: Rc::new(client),
      })
  }
}

impl MidiDriver for CoreMidi {
  fn id(&self) -> &str {
    ID
  }

  // fn sources(&self) -> Iterator<Item=dyn MidiEndpoint> {
  //   unimplemented!();
  // }

  fn destinations(&self) -> Vec<Box<dyn MidiDestination>> {
    Destinations
      .into_iter()
      .enumerate()
      .map(|(index, destination)| {
        let name = destination
          .display_name()
          .unwrap_or_else(|| format!("destination-{}", index));
        Box::new(CoreMidiDestination {
          name,
          default: index == 0,
          client: Rc::clone(&self.client),
          destination: Rc::new(destination),
        }) as Box<MidiDestination>
      })
      .collect()
  }

  // fn create_virtual_output<T>(&self, name: T) -> dyn MidiOutput where T: Into<String> {

  // }
}

pub struct CoreMidiDestination {
  name: String,
  default: bool,
  client: Rc<Client>,
  destination: Rc<Destination>,
}

impl MidiEndpoint for CoreMidiDestination {
  fn name(&self) -> String {
    self.name.clone()
  }
}

impl MidiDestination for CoreMidiDestination {
  fn open(&self) -> MidiResult<BusNodeLock> {
    self
      .client
      .output_port(self.name.as_str())
      .map_err(|status| MidiError::DestinationOpen {
        cause: format!("Destination={:?}, OSStatus={:?}", self.name, status),
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
          destination: self.destination.clone(),
          port,
        })) as Arc<RwLock<BusNode>>
      })
  }
}

struct OutputBusNode {
  name: String,
  features: HashSet<NodeFeature>,
  destination: Rc<Destination>,
  // FIXME add the Rc<Client>
  port: OutputPort,
}

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
    // println!(">>> {:?} {:?}", time, msg);
    let timestamp = unsafe { external::AudioConvertNanosToHostTime(time.to_nanos()) };
    let data_size = Encoder::data_size(msg);
    let mut data = Vec::with_capacity(data_size);
    unsafe { data.set_len(data_size) };
    let slice = data.as_mut_slice();
    Encoder::encode(msg, slice);
    let buffer = PacketBuffer::new(timestamp, slice);
    self.port.send(&self.destination, &buffer).unwrap_or(())
  }
}

mod external {
  #[link(name = "CoreAudio", kind = "framework")]
  extern "C" {
    pub fn AudioConvertNanosToHostTime(inNanos: u64) -> u64;
  }
}
