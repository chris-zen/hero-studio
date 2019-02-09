use std::thread;

use log::{debug, info, trace, warn};

use crossbeam_channel::{Receiver, Select, Sender};

use hero_studio_core::{config::Audio as AudioConfig, studio::Studio};

use crate::audio::{AudioWork, CHANNELS, MAX_FRAMES};
use crate::realtime_thread::RealTimeAudioPriority;

struct WorkerThread {
  studio: Studio,
  audio_config: AudioConfig,
  audio_buffers: Vec<Box<[f32; MAX_FRAMES]>>,
  rta_priority: Option<RealTimeAudioPriority>,
}

impl WorkerThread {
  fn new(studio: Studio, audio_config: AudioConfig) -> WorkerThread {
    let audio_buffers = vec![
      Box::new([0f32; MAX_FRAMES]),
      Box::new([0f32; MAX_FRAMES]),
      Box::new([0f32; MAX_FRAMES]),
    ];

    let rta_priority = Self::promote_to_real_time(&audio_config);

    WorkerThread {
      studio,
      audio_config,
      audio_buffers,
      rta_priority,
    }
  }

  fn prepare_audio_buffer(&mut self) -> Option<Box<[f32; MAX_FRAMES]>> {
    self.audio_buffers.pop().map(|mut buffer| {
      // for i in 0..self.buffer_size {
      //   let v = (i / 2) as f32 / self.buffer_size as f32;
      //   let j = i * 2;
      //   buffer[j] = v;
      //   buffer[j + 1] = v;
      // }


      buffer
    })
  }

  fn audio_work_completed(&mut self, audio_work: AudioWork) {
    let AudioWork { audio, time } = audio_work;
    self.audio_buffers.push(audio);
    // TODO use the time to send the MIDI messages
  }

  fn send_next_audio_buffer(&mut self, audio_tx: &Sender<AudioWork>) {
    match self.prepare_audio_buffer() {
      Some(buffer) => drop(audio_tx.send(AudioWork::new(buffer))),
      None => {
        // TODO notify out of buffers
        debug!("Out of buffers");
      }
    };
  }

  fn promote_to_real_time(audio_config: &AudioConfig) -> Option<RealTimeAudioPriority> {
    match RealTimeAudioPriority::promote(audio_config.sample_rate, audio_config.frames.into()) {
      Ok(rta_priority) => {
        debug!("Worker thread has now real-time priority");
        Some(rta_priority)
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

pub struct Worker {}

impl Worker {
  pub fn new() -> Worker {
    Worker {}
  }

  pub fn start(
    &self,
    studio: Studio,
    audio_config: AudioConfig,
    audio_tx: Sender<AudioWork>,
    audio_rx: Receiver<AudioWork>,
  ) {
    info!("Starting Worker ...");

    let buffer_size = audio_config.frames as usize;

    thread::Builder::new()
      .name("worker".into())
      .spawn(move || {
        let mut wt = WorkerThread::new(studio, audio_config);

        wt.send_next_audio_buffer(&audio_tx);
        wt.send_next_audio_buffer(&audio_tx);

        let mut sel = Select::new();
        let audio_index = sel.recv(&audio_rx);

        loop {
          match sel.ready() {
            index if index == audio_index => {
              audio_rx.try_recv().into_iter().for_each(|audio_work| {
                wt.audio_work_completed(audio_work);
                wt.send_next_audio_buffer(&audio_tx);
              });
            }
            _ => unreachable!(),
          };
        }
      })
      .unwrap();
  }

  pub fn close(&self) {
    info!("Closing Worker ...");
  }
}
