use std::thread;
use std::thread::JoinHandle;

use failure::Fail;
use log::{debug, info, warn};

use crossbeam_channel::{Receiver, Sender};

use hero_studio_core::config::{Audio as AudioConfig, Midi as MidiConfig};
use hero_studio_core::midi;
use hero_studio_core::midi::bus::{BusAddress, MidiBus};
use hero_studio_core::time::ClockTime;

use crate::midi::drivers::{MidiDriver, MidiDrivers};
use crate::realtime_thread::RealTimeAudioPriority;
use crate::studio_workers::Protocol as StudioProtocol;

#[derive(Debug, Fail)]
pub enum MidiIoError {
  #[fail(display = "Failed to create the MIDI output thread: {}", cause)]
  Start { cause: String },

  #[fail(display = "Failed to join the MIDI output thread")]
  Stop,
}

pub enum Protocol {
  Stop,

  Output {
    time: ClockTime,
    io_vec: Box<midi::IoVec>,
  },
}

pub struct MidiOutputThread {
  driver: Box<dyn MidiDriver>,
  midi_bus: MidiBus,
  studio_tx: Sender<StudioProtocol>,
  _rta_priority: Option<RealTimeAudioPriority>,
}

impl MidiOutputThread {
  pub fn new(
    config: &MidiConfig,
    audio_config: &AudioConfig,
    studio_tx: Sender<StudioProtocol>,
  ) -> MidiOutputThread {
    let (driver, midi_bus) = Self::init_destinations(config);

    drop(studio_tx.send(StudioProtocol::MidiOutputInitialised));

    let _rta_priority = None; //Self::promote_to_real_time(audio_config);

    // TODO send the list of destinations to the workers

    MidiOutputThread {
      driver,
      midi_bus,
      studio_tx,
      _rta_priority,
    }
  }

  pub fn handle_messages(&mut self, protocol_rx: Receiver<Protocol>) {
    info!("Handling MIDI output messages ...");

    for message in protocol_rx.iter() {
      match message {
        Protocol::Stop => {
          info!("MIDI output thread stopped ...");
          break;
        }

        Protocol::Output { io_vec, time } => {
          self.send_output(&io_vec, time);
          drop(self.studio_tx.send(StudioProtocol::MidiReleased(io_vec)));
        }
      }
    }
  }

  fn send_output(&mut self, midi_output: &midi::IoVec, base_time: ClockTime) {
    for output in midi_output.iter() {
      if let Some(bus_node_lock) = self.midi_bus.get_node_mut(&output.address) {
        if let Some(boxed_buffer) = &output.buffer {
          if let Ok(mut bus_node) = bus_node_lock.write() {
            for event in boxed_buffer.iter() {
              let timestamp = base_time + event.timestamp;
              bus_node.send_message(timestamp, &event.message);
            }
          }
        }
      }
    }
  }

  pub fn init_destinations(config: &MidiConfig) -> (Box<dyn MidiDriver>, MidiBus) {
    info!("Initialising MIDI output ...");

    let drivers = MidiDrivers::new();
    let app_name = "hero-studio"; // TODO from app_config ?
    let driver = drivers
      .driver(config.driver_id.clone(), app_name)
      .or_else(|_| drivers.default(app_name))
      .unwrap(); // FIXME mybe we need a thread supervisor ?

    debug!("MIDI Driver: {:?}", driver.id());

    let mut midi_bus = MidiBus::new();
    debug!("Destinations:");
    for destination in driver.destinations() {
      debug!("=> {:?}", destination.name());
      if let Ok(bus_node) = destination.open() {
        debug!(
          "   Adding MIDI destination to the bus: {}",
          destination.name()
        );
        midi_bus.add_node(&BusAddress::new(), bus_node);
      }
    }

    (driver, midi_bus)
  }

  fn promote_to_real_time(audio_config: &AudioConfig) -> Option<RealTimeAudioPriority> {
    match RealTimeAudioPriority::promote(audio_config.sample_rate, audio_config.frames.into()) {
      Ok(_rta_priority) => {
        debug!("Midi Output thread has now real-time priority");
        Some(_rta_priority)
      }
      Err(err) => {
        warn!(
          "Couldn't promote the Midi Output thread into real time: {:?}",
          err
        );
        None
      }
    }
  }
}

pub struct MidiOutput {
  handler: JoinHandle<()>,
  protocol_tx: Sender<Protocol>,
}

impl MidiOutput {
  pub fn new_channel() -> (Sender<Protocol>, Receiver<Protocol>) {
    // FIXME use bounded channels !!!
    crossbeam_channel::unbounded::<Protocol>()
  }

  pub fn new(
    config: &MidiConfig,
    audio_config: &AudioConfig,
    protocol_tx: Sender<Protocol>,
    protocol_rx: Receiver<Protocol>,
    studio_tx: Sender<StudioProtocol>,
  ) -> Result<MidiOutput, MidiIoError> {
    info!("Spawning MIDI output thread ...");

    let cloned_config = config.clone();
    let cloned_audio_config = audio_config.clone();

    thread::Builder::new()
      .name("midi-output".into())
      .spawn(move || {
        MidiOutputThread::new(&cloned_config, &cloned_audio_config, studio_tx)
          .handle_messages(protocol_rx)
      })
      .map_err(|err| MidiIoError::Start {
        cause: err.to_string(),
      })
      .map(|handler| MidiOutput {
        handler,
        protocol_tx,
      })
  }

  pub fn stop(self) -> Result<(), MidiIoError> {
    info!("Stopping MIDI output thread ...");

    self
      .protocol_tx
      .send(Protocol::Stop)
      .map_err(|_| MidiIoError::Stop)
      .and_then(|()| self.handler.join().map_err(|_| MidiIoError::Stop))
  }
}
