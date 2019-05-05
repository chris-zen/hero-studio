use std::rc::Rc;

use coremidi::{Client, Destination, Destinations, OutputPort, PacketBuffer};

use hero_studio_core::midi::buffer::Buffer;
use hero_studio_core::midi::encoder::Encoder;
use hero_studio_core::time::ClockTime;

use super::{MidiDestination, MidiDriver, MidiEndpoint, MidiError, MidiOutput, MidiResult};

pub const ID: &str = "CoreMIDI";

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
  client: Rc<Client>,
  destination: Rc<Destination>,
}

impl MidiEndpoint for CoreMidiDestination {
  fn name(&self) -> &str {
    self.name.as_str()
  }
}

impl MidiDestination for CoreMidiDestination {
  fn open(&self) -> MidiResult<Box<dyn MidiOutput>> {
    self
      .client
      .output_port(self.name.as_str())
      .map_err(|status| MidiError::DestinationOpen {
        cause: format!("Destination={:?}, OSStatus={:?}", self.name, status),
      })
      .map(|port| {
        Box::new(CoreMidiOutput::new(
          self.name.clone(),
          self.client.clone(),
          self.destination.clone(),
          port,
        )) as Box<MidiOutput>
      })
  }
}

const MESSAGE_CAPACITY: usize = 8;
const PACKET_BUFFER_CAPACITY: usize = 16 * 1024;

struct CoreMidiOutput {
  name: String,
  _client: Rc<Client>,
  destination: Rc<Destination>,
  port: OutputPort,
  message_buffer: [u8; MESSAGE_CAPACITY],
  packet_buffer: PacketBuffer,
}

impl CoreMidiOutput {
  fn new(
    name: String,
    client: Rc<Client>,
    destination: Rc<Destination>,
    port: OutputPort,
  ) -> CoreMidiOutput {
    let message_buffer = [0; MESSAGE_CAPACITY];
    let packet_buffer = PacketBuffer::with_capacity(PACKET_BUFFER_CAPACITY);

    CoreMidiOutput {
      name,
      _client: client,
      destination,
      port,
      message_buffer,
      packet_buffer,
    }
  }
}

impl MidiEndpoint for CoreMidiOutput {
  fn name(&self) -> &str {
    self.name.as_str()
  }
}

impl MidiOutput for CoreMidiOutput {
  fn send(&mut self, base_time: ClockTime, buffer: &Buffer) {
    self.packet_buffer.clear();
    let mut packet_buffer_size = 0;

    for event in buffer.iter() {
      let timestamp = base_time + event.timestamp;
      let data_size = Encoder::data_size(&event.message);
      if packet_buffer_size + data_size >= PACKET_BUFFER_CAPACITY {
        let _ = self.port.send(&self.destination, &self.packet_buffer);
        self.packet_buffer.clear();
        packet_buffer_size = 0;
      }
      let host_time = unsafe { external::AudioConvertNanosToHostTime(timestamp.to_nanos()) };

      Encoder::encode(&event.message, &mut self.message_buffer);

      self
        .packet_buffer
        .push_data(host_time, &self.message_buffer[0..data_size]);
      packet_buffer_size += data_size;
    }
    let _ = self.port.send(&self.destination, &self.packet_buffer);
  }
}

mod external {
  #[link(name = "CoreAudio", kind = "framework")]
  extern "C" {
    pub fn AudioConvertNanosToHostTime(inNanos: u64) -> u64;
  }
}
