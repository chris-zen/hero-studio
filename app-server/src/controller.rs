use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, Sender};
use failure::Fail;
use log::{debug, info};

use crate::audio::callback::Protocol as AudioProtocol;
use crate::midi::io::Protocol as MidiOutputProtocol;
use crate::server::Message as ServerMessage;

#[derive(Debug, Fail)]
pub enum ControllerError {
  #[fail(display = "Failed to create the Controller thread: {}", cause)]
  Start { cause: String },

  #[fail(display = "Failed to join the Controller thread")]
  Stop,
}

pub enum Protocol {
  Stop,

  ServerInput(ServerMessage),

  MidiInitialised,
}

struct ControllerThread {
  audio_tx: Sender<AudioProtocol>,
  midi_tx: Sender<MidiOutputProtocol>,
}

impl ControllerThread {
  fn new(audio_tx: Sender<AudioProtocol>, midi_tx: Sender<MidiOutputProtocol>) -> ControllerThread {
    ControllerThread { audio_tx, midi_tx }
  }

  pub fn handle_messages(&mut self, protocol_rx: Receiver<Protocol>) {
    for msg in protocol_rx.iter() {
      match msg {
        Protocol::Stop => {
          drop(self.audio_tx.send(AudioProtocol::Stop));
          drop(self.midi_tx.send(MidiOutputProtocol::Stop));
          break;
        }

        Protocol::ServerInput(message) => {
          debug!("Received {:#?}", message);
        }

        Protocol::MidiInitialised => {}
      }
    }
  }
}

pub struct Controller {
  handler: JoinHandle<()>,
  protocol_tx: Sender<Protocol>,
}

impl Controller {
  // TODO Use an spsc array when published by crossbeam
  pub const CHANNEL_CAPACITY: usize = 128 * 1024;
  pub fn new_channel() -> (Sender<Protocol>, Receiver<Protocol>) {
    crossbeam_channel::bounded::<Protocol>(Self::CHANNEL_CAPACITY)
  }

  #[allow(clippy::too_many_arguments)]
  pub fn new(
    protocol_tx: Sender<Protocol>,
    protocol_rx: Receiver<Protocol>,
    audio_tx: Sender<AudioProtocol>,
    midi_tx: Sender<MidiOutputProtocol>,
  ) -> Result<Controller, ControllerError> {
    info!("Starting Controller ...");

    thread::Builder::new()
      .name("controller".into())
      .spawn(move || ControllerThread::new(audio_tx, midi_tx).handle_messages(protocol_rx))
      .map_err(|err| ControllerError::Start {
        cause: err.to_string(),
      })
      .map(|handler| Controller {
        handler,
        protocol_tx,
      })
  }

  pub fn stop(self) -> Result<(), ControllerError> {
    info!("Stopping Controller ...");

    self
      .protocol_tx
      .send(Protocol::Stop)
      .map_err(|_| ControllerError::Stop)
      .and_then(|()| self.handler.join().map_err(|_| ControllerError::Stop))
  }
}
