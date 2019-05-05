use std::collections::HashSet;
use std::thread;
use std::thread::JoinHandle;

use failure::Fail;

use log::{debug, error, info, warn};

use crossbeam_channel::{Receiver, Sender};
use hero_studio_core::midi;
use hero_studio_core::time::ClockTime;

use hero_studio_core::config::{Audio as AudioConfig, Midi as MidiConfig};
use hero_studio_core::midi::buffer::Endpoint;

use crate::midi::drivers::{MidiDriver, MidiDrivers, MidiOutput as MidiOutputPort};
use crate::midi::endpoints::{EndpointId, Endpoints};
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
    buffer_io_vec: Box<midi::BufferIoVec>,
  },
}

pub struct MidiOutputThread {
  _driver: Box<dyn MidiDriver>,
  endpoints: Endpoints<MidiOutputPort>,
  studio_tx: Sender<StudioProtocol>,
  _rta_priority: Option<RealTimeAudioPriority>,
}

impl MidiOutputThread {
  pub fn new(
    config: &MidiConfig,
    audio_config: &AudioConfig,
    studio_tx: Sender<StudioProtocol>,
  ) -> MidiOutputThread {
    let (driver, endpoints) = Self::init_endpoints(config);

    drop(studio_tx.send(StudioProtocol::MidiOutputInitialised));

    let _rta_priority = Self::promote_to_real_time(audio_config);

    MidiOutputThread {
      _driver: driver,
      endpoints,
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

        Protocol::Output {
          time,
          buffer_io_vec,
        } => {
          self.send_buffer_io_vec(time, &buffer_io_vec);
          drop(
            self
              .studio_tx
              .send(StudioProtocol::MidiReleased(buffer_io_vec)),
          );
        }
      }
    }
  }

  fn send_buffer_io_vec(&mut self, base_time: ClockTime, buffer_io_vec: &[midi::BufferIo]) {
    for buffer_io in buffer_io_vec.iter() {
      if let Some(buffer) = &buffer_io.buffer {
        match buffer_io.endpoint {
          Endpoint::None => {}
          Endpoint::Default => {
            if let Some(endpoint) = self.endpoints.get_mut(0) {
              endpoint.send(base_time, buffer)
            }
          }
          Endpoint::All => self
            .endpoints
            .iter_mut()
            .for_each(|endpoint| endpoint.send(base_time, buffer)),
          Endpoint::Id(id) => {
            if let Some(endpoint) = self.endpoints.get_mut(id) {
              endpoint.send(base_time, buffer)
            }
          }
        }
      }
    }
  }

  fn update_endpoints(
    _config: &MidiConfig,
    driver: &MidiDriver,
    endpoints: &mut Endpoints<MidiOutputPort>,
  ) {
    let mut unvisited: HashSet<EndpointId> = endpoints.ids().cloned().collect();

    // TODO send the updates to the studio worker

    debug!("Updating endpoints:");
    for destination in driver.destinations() {
      let name = destination.name();
      if let Some(id) = endpoints.get_id_from_name(&name) {
        unvisited.remove(&id);
        debug!("(=) {} [{}]", name, id);
      } else if let Ok(endpoint) = destination.open() {
        let id = endpoints.add(name, endpoint);
        debug!("(+) {} [{}]", name, id);
      } else {
        error!("Error opening MIDI output port: {}", name);
      }
    }
    endpoints.remove(unvisited, |name, id| debug!("(-) {} [{}]", name, id));
  }

  fn init_endpoints(config: &MidiConfig) -> (Box<dyn MidiDriver>, Endpoints<MidiOutputPort>) {
    info!("Initialising MIDI output ...");

    let drivers = MidiDrivers::new();
    let app_name = "hero-studio"; // TODO from app_config ?
    let driver = drivers
      .driver(config.driver_id.clone(), app_name)
      .or_else(|_| drivers.default(app_name))
      .unwrap(); // FIXME maybe we need a thread supervisor ?

    debug!("MIDI Driver: {}", driver.id());

    let mut endpoints = Endpoints::new();

    Self::update_endpoints(config, driver.as_ref(), &mut endpoints);

    (driver, endpoints)
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
