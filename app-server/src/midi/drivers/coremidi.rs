use std::rc::Rc;

use coremidi::{
  Client, Destination, Destinations, InputPort, OutputPort, PacketBuffer, PacketList, Source,
  Sources,
};

use hero_studio_core::midi::buffer::Buffer;
use hero_studio_core::midi::encoder::Encoder;
use hero_studio_core::time::ClockTime;

use super::{
  MidiDestination, MidiDriver, MidiEndpoint, MidiError, MidiInput, MidiOutput, MidiResult,
  MidiSource, MidiSourceCallback,
};
use hero_studio_core::midi::decoder::{DecodedMessage, Decoder};

pub const ID: &str = "CoreMIDI";

const INPUT_BUFFER_CAPACITY: usize = 16 * 1024;

const OUTPUT_MESSAGE_CAPACITY: usize = 8;
const OUTPUT_PACKET_BUFFER_CAPACITY: usize = 16 * 1024;

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

  fn sources(&self) -> Vec<Box<dyn MidiSource>> {
    Sources
      .into_iter()
      .enumerate()
      .map(|(index, source)| {
        let name = source
          .display_name()
          .unwrap_or_else(|| format!("source-{}", index));
        let client = Rc::clone(&self.client);
        let source = Rc::new(source);
        Box::new(CoreMidiSource::new(name, client, source)) as Box<MidiSource>
      })
      .collect()
  }

  fn destinations(&self) -> Vec<Box<dyn MidiDestination>> {
    Destinations
      .into_iter()
      .enumerate()
      .map(|(index, destination)| {
        let name = destination
          .display_name()
          .unwrap_or_else(|| format!("destination-{}", index));
        let client = Rc::clone(&self.client);
        let destination = Rc::new(destination);
        Box::new(CoreMidiDestination::new(name, client, destination)) as Box<MidiDestination>
      })
      .collect()
  }

  // fn create_virtual_output<T>(&self, name: T) -> dyn MidiOutput where T: Into<String> {

  // }
}

pub struct CoreMidiSource {
  name: String,
  client: Rc<Client>,
  source: Rc<Source>,
}

impl CoreMidiSource {
  fn new(name: String, client: Rc<Client>, source: Rc<Source>) -> Self {
    CoreMidiSource {
      name,
      client,
      source,
    }
  }

  fn callback_proxy(packet_list: &PacketList, buffer: &mut Buffer, callback: &MidiSourceCallback) {
    buffer.reset();
    for packet in packet_list.iter() {
      let nanos = unsafe { external::AudioConvertHostTimeToNanos(packet.timestamp()) };
      let timestamp = ClockTime::from_nanos(nanos);
      for event in Decoder::new(packet.data()) {
        if let DecodedMessage::Message(msg) = event {
          buffer.push(timestamp, msg)
        }
      }
    }
    (callback)(&buffer)
  }
}

impl MidiSource for CoreMidiSource {
  fn name(&self) -> &str {
    self.name.as_str()
  }

  fn open(&self, callback: Box<MidiSourceCallback>) -> Result<Box<MidiInput>, MidiError> {
    let mut buffer = Buffer::with_capacity(INPUT_BUFFER_CAPACITY);
    self
      .client
      .input_port(self.name.as_str(), move |packet_list: &PacketList| {
        Self::callback_proxy(packet_list, &mut buffer, &*callback)
      })
      .and_then(|port| port.connect_source(&self.source).map(|_| port))
      .map_err(|status| MidiError::SourceOpen {
        cause: format!("Source={:?}, OSStatus={:?}", self.name, status),
      })
      .map(|port| {
        Box::new(CoreMidiInput::new(
          self.name.clone(),
          self.client.clone(),
          self.source.clone(),
          port,
        )) as Box<MidiInput>
      })
  }
}

struct CoreMidiInput {
  name: String,
  _client: Rc<Client>,
  _source: Rc<Source>,
  _port: InputPort,
}

impl CoreMidiInput {
  fn new(name: String, client: Rc<Client>, source: Rc<Source>, port: InputPort) -> CoreMidiInput {
    CoreMidiInput {
      name,
      _client: client,
      _source: source,
      _port: port,
    }
  }
}

impl MidiEndpoint for CoreMidiInput {
  fn name(&self) -> &str {
    self.name.as_str()
  }
}

impl MidiInput for CoreMidiInput {}

pub struct CoreMidiDestination {
  name: String,
  client: Rc<Client>,
  destination: Rc<Destination>,
}

impl CoreMidiDestination {
  fn new(name: String, client: Rc<Client>, destination: Rc<Destination>) -> Self {
    CoreMidiDestination {
      name,
      client,
      destination,
    }
  }
}

impl MidiDestination for CoreMidiDestination {
  fn name(&self) -> &str {
    self.name.as_str()
  }

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

struct CoreMidiOutput {
  name: String,
  _client: Rc<Client>,
  destination: Rc<Destination>,
  port: OutputPort,
  message_buffer: [u8; OUTPUT_MESSAGE_CAPACITY],
  packet_buffer: PacketBuffer,
}

impl CoreMidiOutput {
  fn new(
    name: String,
    client: Rc<Client>,
    destination: Rc<Destination>,
    port: OutputPort,
  ) -> CoreMidiOutput {
    let message_buffer = [0; OUTPUT_MESSAGE_CAPACITY];
    let packet_buffer = PacketBuffer::with_capacity(OUTPUT_PACKET_BUFFER_CAPACITY);

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
      if packet_buffer_size + data_size >= OUTPUT_PACKET_BUFFER_CAPACITY {
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
    pub fn AudioConvertHostTimeToNanos(inHostTime: u64) -> u64;
  }
}
