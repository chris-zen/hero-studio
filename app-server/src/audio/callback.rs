use crossbeam_channel::{Receiver, Sender};

use failure::Fail;

use hero_studio_core::audio::{AudioInput, AudioOutput};
use hero_studio_core::midi::buffer::EventIo;
use hero_studio_core::midi::io::{MidiInput, MidiOutput};
use hero_studio_core::studio::Studio;

use crate::midi::io::Protocol as MidiIoProtocol;


#[derive(Debug, Fail)]
pub enum CallbackError {
  //  #[fail(display = "Failed to try receiving an event: {}", cause)]
  //  TryRecvError { cause: String },
}

impl std::fmt::Display for CallbackError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "CallbackError")
  }
}

pub enum Protocol {
  Stop,
}

struct ReceiverMidiInput {}

impl ReceiverMidiInput {
  fn new() -> Self {
    ReceiverMidiInput {}
  }
}

impl MidiInput for ReceiverMidiInput {}

struct SenderMidiOutput {
  tx: Sender<MidiIoProtocol>,
}

impl SenderMidiOutput {
  fn new(tx: Sender<MidiIoProtocol>) -> Self {
    SenderMidiOutput { tx }
  }
}

impl MidiOutput for SenderMidiOutput {
  fn push(&mut self, event: EventIo) {
    let msg = MidiIoProtocol::Event(event);
    drop(self.tx.send(msg))
  }
}

pub enum AudioCallbackResult {
  Continue,
  Stop,
}

pub struct AudioCallback {
  studio: Studio,
  protocol_rx: Receiver<Protocol>,
  midi_input: ReceiverMidiInput,
  midi_output: SenderMidiOutput,
}

impl AudioCallback {
  pub fn new(
    studio: Studio,
    protocol_rx: Receiver<Protocol>,
    midi_out_tx: Sender<MidiIoProtocol>,
  ) -> AudioCallback {
    AudioCallback {
      studio,
      protocol_rx,
      midi_input: ReceiverMidiInput::new(),
      midi_output: SenderMidiOutput::new(midi_out_tx),
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub fn process(
    &mut self,
    frames: usize,
    audio_input: AudioInput,
    mut audio_output: AudioOutput,
  ) -> Result<AudioCallbackResult, CallbackError> {
    let result = self.handle_messages()?;

    self.studio.process(
      frames,
      &audio_input,
      &mut audio_output,
      &self.midi_input,
      &mut self.midi_output,
    );

    Ok(result)
  }

  fn handle_messages(&mut self) -> Result<AudioCallbackResult, CallbackError> {
    match self.protocol_rx.try_recv() {
      Ok(msg) => self.handle_message(msg),
      Err(_) => Ok(AudioCallbackResult::Continue),
    }
  }

  fn handle_message(&mut self, msg: Protocol) -> Result<AudioCallbackResult, CallbackError> {
    match msg {
      Protocol::Stop => Ok(AudioCallbackResult::Stop),
    }
  }
}
