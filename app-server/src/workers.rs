use std::thread;

use log::{debug, info, warn};

use crossbeam_channel::{Receiver, Select, Sender};

use hero_studio_core::audio;
use hero_studio_core::audio::{new_protocol_pool, Protocol};
use hero_studio_core::config::Audio as AudioConfig;
use hero_studio_core::midi;
use hero_studio_core::pool::Pool;
use hero_studio_core::studio::Studio;

use crate::config::Config as AppConfig;
use crate::realtime_thread::RealTimeAudioPriority;


struct MasterThread {
  studio: Studio,
  num_samples: usize,
  audio_buffer_pool: Pool<audio::Buffer>,
  midi_buffer_pool: Pool<midi::Buffer>,
  midi_io_vec_pool: Pool<midi::IoVec>,
  protocol_pool: Pool<Protocol>,
  _rta_priority: Option<RealTimeAudioPriority>,
}

impl MasterThread {
  fn new(studio: Studio, app_config: AppConfig) -> MasterThread {
    let config = studio.config();
    let audio_config = config.audio.clone();
    let num_samples = audio_config.frames as usize;

    let app_audio_config = &app_config.audio;
    let app_midi_config = &app_config.midi;

    let audio_buffer_pool = audio::new_buffer_pool(
      app_audio_config.buffer_pool.pool_capacity,
      app_audio_config.buffer_pool.item_capacity,
    );

    let midi_buffer_pool = midi::new_buffer_pool(
      app_midi_config.buffer_pool.pool_capacity,
      app_midi_config.buffer_pool.item_capacity,
    );

    let midi_io_vec_pool = midi::new_io_vec_pool(
      app_midi_config.io_vec_pool.pool_capacity,
      app_midi_config.io_vec_pool.item_capacity,
    );

    let protocol_pool = new_protocol_pool(
      app_audio_config.protocol_pool.pool_capacity,
    );

    let _rta_priority = Self::promote_to_real_time(&audio_config);

    MasterThread {
      studio,
      num_samples,
      audio_buffer_pool,
      midi_buffer_pool,
      midi_io_vec_pool,
      protocol_pool,
      _rta_priority,
    }
  }

  fn protocol_received(&mut self, mut protocol: Box<Protocol>) {
    {
      let output_time = protocol.output_time();
      let mut midi_output = protocol.midi_output();

      self
        .studio
        .complete(&mut self.midi_buffer_pool, &mut midi_output, output_time);
    }

    {
      // TODO do something with the input: audio_input, input_time

      let _input_time = protocol.input_time();
      let _audio_input = protocol.audio_input();
    }

    protocol.release(&mut self.midi_io_vec_pool, &mut self.audio_buffer_pool);
    self.protocol_pool.release(protocol);
  }

  fn prepare_protocol(&mut self) -> Option<Box<Protocol>> {
    let mut audio_output = self.audio_buffer_pool.get()?;
    let audio_input = self.audio_buffer_pool.get()?;
    let mut midi_output = self.midi_io_vec_pool.get()?;

    self.studio.prepare(
      self.num_samples,
      &mut self.midi_buffer_pool,
      &mut midi_output,
      &audio_input,
      &mut audio_output,
    );

    let mut protocol = self.protocol_pool.get_or_alloc();
    protocol.prepare(midi_output, audio_output, audio_input);
    Some(protocol)
  }

  fn send_next_audio_buffer(&mut self, audio_tx: &Sender<Box<Protocol>>) {
    match self.prepare_protocol() {
      Some(protocol) => {
        drop(audio_tx.send(protocol));
      }
      None => {
        // TODO notify out of buffers
        debug!("Out of buffers");
      }
    };
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

pub struct Workers {}

impl Workers {
  pub fn new() -> Workers {
    Workers {}
  }

  pub fn start(
    &self,
    studio: Studio,
    app_config: AppConfig,
    audio_tx: Sender<Box<Protocol>>,
    audio_rx: Receiver<Box<Protocol>>,
  ) {
    info!("Starting Workers ...");

    thread::Builder::new()
      .name("worker-master".into())
      .spawn(move || {
        let mut master = MasterThread::new(studio, app_config);

        master.send_next_audio_buffer(&audio_tx);
        master.send_next_audio_buffer(&audio_tx);

        let mut sel = Select::new();
        let audio_index = sel.recv(&audio_rx);

        loop {
          match sel.ready() {
            index if index == audio_index => {
              audio_rx.try_recv().into_iter().for_each(|protocol| {
                master.protocol_received(protocol);
                master.send_next_audio_buffer(&audio_tx);
              });
            }
            _ => unreachable!(),
          };
        }
      })
      .unwrap();
  }

  pub fn close(&self) {
    info!("Closing Workers ...");
  }
}
