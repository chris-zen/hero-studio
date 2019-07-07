use std::collections::HashSet;
use std::thread;
use std::thread::JoinHandle;

use failure::Fail;

use log::{debug, error, info};

use crossbeam_channel::{Receiver, Sender};
use hero_studio_core::time::ClockTime;

use hero_studio_core::config::{Audio as AudioConfig, Midi as MidiConfig};
use hero_studio_core::midi::buffer::{Buffer, Endpoint, EventIo};

use crate::controller::Protocol as StudioProtocol;
use crate::midi::drivers::{MidiDriver, MidiDrivers, MidiOutput as MidiOutputPort, MidiInput as MidiInputPort};
use crate::midi::endpoints::{EndpointId, Endpoints};
use crate::realtime::RealTimeAudioPriority;


#[derive(Debug, Fail)]
pub enum MidiIoError {
  #[fail(display = "Failed to create the MIDI output thread: {}", cause)]
  Start { cause: String },

  #[fail(display = "Failed to join the MIDI output thread")]
  Stop,
}

#[derive(Debug)]
pub enum Protocol {
  Stop,

  EventOut(EventIo),

  EventIn(EventIo),
}

pub struct MidiIoThread {
  _driver: Box<dyn MidiDriver>,
  endpoints_out: Endpoints<MidiOutputPort>,
  _endpoints_in: Endpoints<MidiInputPort>,
  buffer: Option<Buffer>,
  _rta_priority: Option<RealTimeAudioPriority>,
}

impl MidiIoThread {
  pub fn new(
    config: &MidiConfig,
    audio_config: &AudioConfig,
    midi_in_tx: Sender<Protocol>,
    studio_tx: Sender<StudioProtocol>,
  ) -> MidiIoThread {
    let (driver, endpoints_out, endpoints_in) = Self::init_endpoints(config, midi_in_tx);

    drop(studio_tx.send(StudioProtocol::MidiInitialised));

    let _rta_priority =
      RealTimeAudioPriority::promote(audio_config.sample_rate, audio_config.frames.into()).ok();

    MidiIoThread {
      _driver: driver,
      endpoints_out,
      _endpoints_in: endpoints_in,
      buffer: Some(Buffer::with_capacity(1)),
      _rta_priority,
    }
  }

  pub fn handle_messages(&mut self, protocol_rx: Receiver<Protocol>) {
    info!("Handling MIDI output messages ...");

    for message in protocol_rx.iter() {
      match message {
        Protocol::EventOut(event) => {
          self.send_event(event);
        }

        Protocol::Stop => {
          info!("MIDI output thread stopped ...");
          break;
        }

        _ => unreachable!()
      }
    }
  }

  // FIXME Temporal solution until we can use crossbeam spsc array and read on chunks rather than individual events
  fn send_event(&mut self, event: EventIo) {
    let mut buffer = self.buffer.take().unwrap();

    buffer.reset().push(event.timestamp, event.message);

    match event.endpoint {
      Endpoint::None => {}

      Endpoint::Default => {
        if let Some(endpoint) = self.endpoints_out.get_mut(0) {
          endpoint.send(ClockTime::zero(), &buffer)
        }
      }

      Endpoint::All => self
        .endpoints_out
        .iter_mut()
        .for_each(|endpoint| endpoint.send(ClockTime::zero(), &buffer)),

      Endpoint::Id(id) => {
        if let Some(endpoint) = self.endpoints_out.get_mut(id) {
          endpoint.send(ClockTime::zero(), &buffer)
        }
      }
    }

    self.buffer = Some(buffer);
  }

  // TODO This logic should go into another thread that will scan ports regularly and report back to this one
  fn update_endpoints_out(
    _config: &MidiConfig,
    driver: &MidiDriver,
    endpoints_out: &mut Endpoints<MidiOutputPort>,
  ) {
    let mut unvisited: HashSet<EndpointId> = endpoints_out.ids().cloned().collect();

    // TODO send the updates to the studio worker

    debug!("Updating output endpoints:");
    for destination in driver.destinations() {
      let name = destination.name();
      if let Some(id) = endpoints_out.get_id_from_name(&name) {
        unvisited.remove(&id);
        debug!("(=) {} [{}]", name, id);
      } else if let Ok(endpoint) = destination.open() {
        let id = endpoints_out.add(name, endpoint);
        debug!("(+) {} [{}]", name, id);
      } else {
        error!("Error opening MIDI output port: {}", name);
      }
    }
    endpoints_out.remove(unvisited, |name, id| debug!("(-) {} [{}]", name, id));
  }

  fn update_endpoints_in(
    _config: &MidiConfig,
    driver: &MidiDriver,
    midi_in_tx: Sender<Protocol>,
    endpoints_in: &mut Endpoints<MidiInputPort>,
  ) {
    let mut unvisited: HashSet<EndpointId> = endpoints_in.ids().cloned().collect();

    // TODO send the updates to the studio worker

    debug!("Updating input endpoints:");

    for source in driver.sources() {
      let name = source.name();
      let tx = midi_in_tx.clone();

      if let Some(id) = endpoints_in.get_id_from_name(&name) {
        unvisited.remove(&id);
        debug!("(=) {} [{}]", name, id);
      } else {
        let id = endpoints_in.next_id();

        let callback = Box::new(move |buffer: &Buffer| {
          for event in buffer.iter() {
            let endpoint = Endpoint::Id(id);
            let event_io = EventIo::new(event.timestamp, endpoint, event.message.clone());
            drop(tx.send(Protocol::EventIn(event_io)));
          }
        });

        if let Ok(endpoint) = source.open(callback) {
          endpoints_in.add(name, endpoint);
          debug!("(+) {} [{}]", name, id);
        } else {
          error!("Error opening MIDI input port: {}", name);
        }
      }
    }
    endpoints_in.remove(unvisited, |name, id| debug!("(-) {} [{}]", name, id));
  }

  fn init_endpoints(config: &MidiConfig, midi_in_tx: Sender<Protocol>) -> (Box<dyn MidiDriver>, Endpoints<MidiOutputPort>, Endpoints<MidiInputPort>) {
    info!("Initialising MIDI IO ...");

    let drivers = MidiDrivers::new();
    let app_name = "hero-studio"; // TODO from app_config ?
    let driver = drivers
      .driver(config.driver_id.clone(), app_name)
      .or_else(|_| drivers.default(app_name))
      .unwrap(); // FIXME maybe we need a thread supervisor ?

    debug!("MIDI Driver: {}", driver.id());

    let mut endpoints_out = Endpoints::new();
    Self::update_endpoints_out(config, driver.as_ref(), &mut endpoints_out);

    let mut endpoints_in = Endpoints::new();
    Self::update_endpoints_in(config, driver.as_ref(), midi_in_tx, &mut endpoints_in);

    (driver, endpoints_out, endpoints_in)
  }
}

pub struct MidiIo {
  handler: JoinHandle<()>,
  midi_out_tx: Sender<Protocol>,
}

impl MidiIo {
  // TODO Use an spsc array when published by crossbeam
  pub const CHANNEL_CAPACITY: usize = 128 * 1024;
  pub fn new_channel() -> (Sender<Protocol>, Receiver<Protocol>) {
    crossbeam_channel::bounded::<Protocol>(Self::CHANNEL_CAPACITY)
  }

  pub fn new(
    config: &MidiConfig,
    audio_config: &AudioConfig,
    midi_out_tx: Sender<Protocol>,
    midi_out_rx: Receiver<Protocol>,
    midi_in_tx: Sender<Protocol>,
    studio_tx: Sender<StudioProtocol>,
  ) -> Result<MidiIo, MidiIoError> {
    info!("Spawning MIDI IO thread ...");

    let cloned_config = config.clone();
    let cloned_audio_config = audio_config.clone();

    thread::Builder::new()
      .name("midi-io".into())
      .spawn(move || {
        MidiIoThread::new(&cloned_config, &cloned_audio_config, midi_in_tx, studio_tx)
          .handle_messages(midi_out_rx)
      })
      .map_err(|err| MidiIoError::Start {
        cause: err.to_string(),
      })
      .map(|handler| MidiIo {
        handler,
        midi_out_tx,
      })
  }

  pub fn stop(self) -> Result<(), MidiIoError> {
    info!("Stopping MIDI IO thread ...");

    self
      .midi_out_tx
      .send(Protocol::Stop)
      .map_err(|_| MidiIoError::Stop)
      .and_then(|()| self.handler.join().map_err(|_| MidiIoError::Stop))
  }
}
