use std::thread;
use std::thread::JoinHandle;

use failure::Fail;
use log::{debug, info, warn};

use crossbeam_channel::{Receiver, Sender};

use hero_studio_core::audio;
use hero_studio_core::config::Audio as AudioConfig;
use hero_studio_core::midi;
use hero_studio_core::pool::Pool;
use hero_studio_core::studio::Studio;
use hero_studio_core::time::ClockTime;

use crate::audio::io::Protocol as AudioProtocol;
use crate::config::Config as AppConfig;
use crate::midi::output::Protocol as MidiOutputProtocol;
use crate::realtime_thread::RealTimeAudioPriority;

#[derive(Debug, Fail)]
pub enum StudioWorkersError {
  #[fail(display = "Failed to create the Studio workers threads: {}", cause)]
  Start { cause: String },

  #[fail(display = "Failed to join the Studio workers threads")]
  Stop,
}

pub enum Protocol {
  Stop,

  AudioInput {
    time: ClockTime,                    // timestamp of the captured audio input
    input: Box<audio::Buffer>,          // captured audio input buffer
    release_output: Box<audio::Buffer>, // audio output buffer to be released
  },

  MidiOutputInitialised,
  MidiReleased(Box<midi::BufferIoVec>),
}

struct MasterThread {
  studio: Studio,
  audio_tx: Sender<AudioProtocol>,
  midi_out_tx: Sender<MidiOutputProtocol>,
  num_samples: usize,
  num_input_channels: usize,
  num_output_channels: usize,
  audio_buffer_pool: Pool<audio::Buffer>,
  midi_buffer_pool: Pool<midi::Buffer>,
  midi_io_vec_pool: Pool<midi::BufferIoVec>,
  _rta_priority: Option<RealTimeAudioPriority>,
}

impl MasterThread {
  fn new(
    studio: Studio,
    app_config: AppConfig,
    audio_tx: Sender<AudioProtocol>,
    midi_out_tx: Sender<MidiOutputProtocol>,
    num_input_channels: usize,
    num_output_channels: usize,
  ) -> MasterThread {
    let config = studio.config();
    let audio_config = config.audio.clone();
    let num_samples = audio_config.frames as usize;

    let app_audio_config = &app_config.audio;
    let app_midi_config = &app_config.midi;

    let max_num_channels = num_input_channels.max(num_output_channels);
    let audio_buffer_pool = audio::new_buffer_pool(
      app_audio_config.buffer_pool.pool_capacity,
      app_audio_config.buffer_pool.item_capacity * max_num_channels,
    );

    let midi_buffer_pool = midi::new_buffer_pool(
      app_midi_config.buffer_pool.pool_capacity,
      app_midi_config.buffer_pool.item_capacity,
    );

    let midi_io_vec_pool = midi::new_buffer_io_vec_pool(
      app_midi_config.io_vec_pool.pool_capacity,
      app_midi_config.io_vec_pool.item_capacity,
    );

    let _rta_priority = Self::promote_to_real_time(&audio_config);

    MasterThread {
      studio,
      audio_tx,
      midi_out_tx,
      num_samples,
      num_input_channels,
      num_output_channels,
      audio_buffer_pool,
      midi_buffer_pool,
      midi_io_vec_pool,
      _rta_priority,
    }
  }

  pub fn handle_messages(&mut self, protocol_rx: Receiver<Protocol>) {
    for msg in protocol_rx.iter() {
      match msg {
        Protocol::Stop => {
          drop(self.audio_tx.send(AudioProtocol::Stop));
          drop(self.midi_out_tx.send(MidiOutputProtocol::Stop));
          break;
        }

        Protocol::AudioInput {
          time,
          input,
          release_output,
        } => {
          let process_result = self.process(time, input, release_output);
          process_result.into_iter().for_each(|msg| {
            drop(self.audio_tx.send(msg));
          });
        }

        Protocol::MidiOutputInitialised => {
          drop(self.send_initial_audio());
          //          drop(self.send_initial_audio());
        }

        Protocol::MidiReleased(io_vec) => {
          self.midi_io_vec_pool.release(io_vec);
        }
      }
    }
  }

  fn send_initial_audio(&mut self) -> Option<()> {
    let audio_input = self.audio_buffer_pool.get()?;
    let audio_output = self.audio_buffer_pool.get()?;
    self
      .process(ClockTime::zero(), audio_input, audio_output)
      .into_iter()
      .for_each(|msg| {
        drop(self.audio_tx.send(msg));
      });
    Some(())
  }

  fn process(
    &mut self,
    input_time: ClockTime,
    audio_input: Box<audio::Buffer>,
    mut audio_output: Box<audio::Buffer>,
  ) -> Option<AudioProtocol> {
    let mut midi_output = self.midi_io_vec_pool.get()?;

    self.studio.process(
      self.num_samples,
      input_time,
      self.num_input_channels,
      &audio_input,
      self.num_output_channels,
      &mut audio_output,
      &mut self.midi_buffer_pool,
      &mut midi_output,
    );

    Some(AudioProtocol::Next {
      audio_input,
      midi_output,
      audio_output,
    })
  }

  fn promote_to_real_time(audio_config: &AudioConfig) -> Option<RealTimeAudioPriority> {
    match RealTimeAudioPriority::promote(audio_config.sample_rate, audio_config.frames.into()) {
      Ok(_rta_priority) => {
        debug!("Worker thread has now real-time priority");
        Some(_rta_priority)
      }
      Err(err) => {
        warn!(
          "Couldn't promote the Worker thread into real time: {:?}",
          err
        );
        None
      }
    }
  }
}

pub struct StudioWorkers {
  handler: JoinHandle<()>,
  protocol_tx: Sender<Protocol>,
}

impl StudioWorkers {
  pub fn new_channel() -> (Sender<Protocol>, Receiver<Protocol>) {
    // FIXME use bounded channels !!!
    crossbeam_channel::unbounded::<Protocol>()
  }

  pub fn new(
    studio: Studio,
    app_config: AppConfig,
    protocol_tx: Sender<Protocol>,
    protocol_rx: Receiver<Protocol>,
    audio_tx: Sender<AudioProtocol>,
    midi_out_tx: Sender<MidiOutputProtocol>,
    num_input_channels: usize,
    num_output_channels: usize,
  ) -> Result<StudioWorkers, StudioWorkersError> {
    info!("Starting Workers ...");

    thread::Builder::new()
      .name("studio-master".into())
      .spawn(move || {
        MasterThread::new(
          studio,
          app_config,
          audio_tx,
          midi_out_tx,
          num_input_channels,
          num_output_channels,
        )
        .handle_messages(protocol_rx)
      })
      .map_err(|err| StudioWorkersError::Start {
        cause: err.to_string(),
      })
      .map(|handler| StudioWorkers {
        handler,
        protocol_tx,
      })
  }

  pub fn stop(self) -> Result<(), StudioWorkersError> {
    info!("Stopping Workers ...");

    self
      .protocol_tx
      .send(Protocol::Stop)
      .map_err(|_| StudioWorkersError::Stop)
      .and_then(|()| self.handler.join().map_err(|_| StudioWorkersError::Stop))
  }
}
