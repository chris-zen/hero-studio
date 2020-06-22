// use log::{debug};

use std::rc::Rc;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};

use portmidi::{DeviceInfo, InputPort, MidiEvent, MidiMessage, OutputPort, PortMidi};

use hero_studio_core::time::ClockTime;
use hero_studio_core::midi::buffer::Buffer;
use hero_studio_core::midi::{encoder::Encoder, messages::Message};
use hero_studio_core::midi::decoder::{DecodedMessage, Decoder};

use super::{
  MidiDestination, MidiDriver, MidiEndpoint, MidiError, MidiInput, MidiOutput, MidiResult,
  MidiSource,
};
use crate::midi::drivers::MidiSourceCallback;


pub const ID: &str = "PortMIDI";

const MIDI_BUF_LEN: usize = 8 * 1024;

const POLL_MAX_WAIT_NANOS: u64 = 1_000_000; // 1 ms

const INPUT_BUFFER_CAPACITY: usize = 16 * 1024;

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

  fn sources(&self) -> Vec<Box<MidiSource>> {
    self
      .context
      .devices()
      .into_iter()
      .flat_map(|devices| {
        devices
          .into_iter()
          .filter(DeviceInfo::is_input)
          .map(|device| {
            Box::new(PortMidiSource {
              name: device.name().clone(),
              context: Rc::clone(&self.context),
              device: device.clone(),
            }) as Box<MidiSource>
          })
      })
      .collect()
  }

  fn destinations(&self) -> Vec<Box<dyn MidiDestination>> {
    self
      .context
      .devices()
      .into_iter()
      .flat_map(|devices| {
        devices
          .into_iter()
          .filter(DeviceInfo::is_output)
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
}

pub struct PortMidiSource {
  name: String,
  context: Rc<PortMidi>,
  device: DeviceInfo,
}

impl MidiSource for PortMidiSource {
  fn name(&self) -> &str {
    self.name.as_str()
  }

  fn open(&self, callback: Box<MidiSourceCallback>) -> Result<Box<MidiInput>, MidiError> {
    self
      .context
      .input_port(self.device.clone(), MIDI_BUF_LEN)
      .map_err(|err| MidiError::SourceOpen {
        cause: format!("Device={:?}, Error={:?}", self.name, err),
      })
      .map(|port| {
        Box::new(PortMidiInput::new(
          self.name.clone(),
          self.context.clone(),
          port,
          callback,
        )) as Box<MidiInput>
      })
  }
}

struct PortMidiInput {
  name: String,
  _context: Rc<PortMidi>,
  handler: Option<JoinHandle<()>>,
  done: Arc<AtomicBool>,
}

impl PortMidiInput {
  fn new(
    name: String,
    context: Rc<PortMidi>,
    port: InputPort,
    callback: Box<MidiSourceCallback>,
  ) -> PortMidiInput {
    let done = Arc::new(AtomicBool::new(false));
    let done_clone = Arc::clone(&done);

    let thread_name = format!("portmidi-{}", name);
    let handler = std::thread::Builder::new()
      .name(thread_name)
      .spawn(|| Self::poll(port, callback, done_clone))
      .ok();

    PortMidiInput {
      name,
      _context: context,
      handler,
      done,
    }
  }

  fn poll(port: InputPort, callback: Box<MidiSourceCallback>, done: Arc<AtomicBool>) {
    let mut wait_nanos: u64 = 1;
    let mut buffer = Buffer::with_capacity(INPUT_BUFFER_CAPACITY);
    while !done.load(Ordering::Relaxed) {
      if let Ok(events_available) = port.poll() {
        if events_available {
          buffer.reset();
          if let Ok(Some(events)) = port.read_n(MIDI_BUF_LEN) {
            for event in events.into_iter() {
              let raw_msg = event.message;
              let data = [raw_msg.status, raw_msg.data1, raw_msg.data2];
              if let Some(DecodedMessage::Message(message)) = Decoder::new(&data).next() {
                let timestamp = ClockTime::from_millis(u64::from(event.timestamp));
                buffer.push(timestamp, message)
              }
            }
            (callback)(&buffer);
            wait_nanos = 1;
          }
        }
      }

      std::thread::sleep(Duration::from_nanos(wait_nanos));
      wait_nanos = POLL_MAX_WAIT_NANOS.min(wait_nanos * 2);
    }
  }
}

impl MidiEndpoint for PortMidiInput {
  fn name(&self) -> &str {
    self.name.as_str()
  }
}

impl MidiInput for PortMidiInput {}

impl Drop for PortMidiInput {
  fn drop(&mut self) {
    self.done.store(true, Ordering::Relaxed);
    self.handler.take().into_iter().for_each(|handler| {
      let _ = handler.join();
    })
  }
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

impl MidiDestination for PortMidiDestination {
  fn name(&self) -> &str {
    self.name.as_str()
  }

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
