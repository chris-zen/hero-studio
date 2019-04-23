use crossbeam_channel::{Receiver, Sender};

use log::debug;

use hero_studio_core::audio;
use hero_studio_core::midi;
use hero_studio_core::time::ClockTime;

use crate::midi::output::Protocol as MidiOutputProtocol;
use crate::studio_workers::Protocol as StudioProtocol;

pub enum Protocol {
  Stop,

  Next {
    audio_input: Box<audio::Buffer>, // pre-allocated buffer for the captured audio input
    midi_output: Box<midi::IoVec>,   // midi to send into the MIDI output thread
    audio_output: Box<audio::Buffer>, // audio to send into the output stream
  },
}

pub enum AudioIoResult {
  Continue,
  Stop,
}

pub struct AudioIo {
  studio_ready: bool,
  protocol_rx: Receiver<Protocol>,
  studio_tx: Sender<StudioProtocol>,
  midi_out_tx: Sender<MidiOutputProtocol>,
}

impl AudioIo {
  pub fn new(
    protocol_rx: Receiver<Protocol>,
    studio_tx: Sender<StudioProtocol>,
    midi_out_tx: Sender<MidiOutputProtocol>,
  ) -> AudioIo {
    AudioIo {
      studio_ready: false,
      protocol_rx,
      studio_tx,
      midi_out_tx,
    }
  }

  pub fn process(
    &mut self,
    frames: usize,
    input_time: ClockTime,
    input_channels: usize,
    input: &[f32],
    output_time: ClockTime,
    output_channels: usize,
    output: &mut [f32],
  ) -> AudioIoResult {
    match self.protocol_rx.try_recv() {
      Ok(msg) => match msg {
        Protocol::Stop => AudioIoResult::Stop,

        Protocol::Next {
          midi_output,
          audio_output,
          mut audio_input,
        } => {
          self.studio_ready = true;

          let in_buffer_size = frames * input_channels;
          audio_input[0..in_buffer_size].copy_from_slice(&input);

          let out_buffer_size = frames * output_channels;
          output.copy_from_slice(&audio_output[0..out_buffer_size]);

          drop(self.studio_tx.send(StudioProtocol::AudioInput {
            time: input_time,
            input: audio_input,
            release_output: audio_output,
          }));

          drop(self.midi_out_tx.send(MidiOutputProtocol::Output {
            time: output_time,
            io_vec: midi_output,
          }));

          AudioIoResult::Continue
        }
      },

      Err(err) => {
        Self::fill_with_zero(output);
        if self.studio_ready {
          debug!("xrun: {:?} {}", output_time, err);
          // TODO send xrun event
        }

        AudioIoResult::Continue
        // TODO return Stop if too high xrun ratio
      }
    }
  }

  fn fill_with_zero(s: &mut [f32]) {
    for d in s {
      *d = 0.0;
    }
  }
}
