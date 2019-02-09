use std::thread;

use log::{debug, info, trace, warn};

use crossbeam_channel::{Receiver, Select, Sender};

use audio_thread_priority::{
  demote_current_thread_from_real_time, promote_current_thread_to_real_time, RtPriorityHandle,
};

use hero_studio_core::{config::Audio as AudioConfig, studio::Studio};

use crate::audio::{AudioWork, CHANNELS, MAX_FRAMES};

struct WorkerThread {
  studio: Studio,
  buffer_size: usize,
  audio_buffers: Vec<Box<[f32; MAX_FRAMES]>>,
}

impl WorkerThread {
  fn new(studio: Studio, buffer_size: usize) -> WorkerThread {
    let audio_buffers = vec![
      Box::new([0f32; MAX_FRAMES]),
      Box::new([0f32; MAX_FRAMES]),
      Box::new([0f32; MAX_FRAMES]),
    ];

    WorkerThread {
      studio,
      buffer_size,
      audio_buffers,
    }
  }

  fn prepare_audio_buffer(&mut self) -> Option<Box<[f32; MAX_FRAMES]>> {
    self.audio_buffers.pop().map(|mut buffer| {
      for i in 0..self.buffer_size {
        let v = (i / 2) as f32 / self.buffer_size as f32;
        let j = i * 2;
        buffer[j] = v;
        buffer[j + 1] = v;
      }
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
        let rt_handle = Self::promote_to_real_time(&audio_config);

        let mut wt = WorkerThread::new(studio, buffer_size);

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

        Self::demote_from_real_time(rt_handle);
      })
      .unwrap();
  }

  pub fn close(&self) {
    info!("Closing Worker ...");
  }

  fn promote_to_real_time(audio_config: &AudioConfig) -> Option<RtPriorityHandle> {
    match promote_current_thread_to_real_time(audio_config.frames.into(), audio_config.sample_rate)
    {
      Ok(handle) => {
        debug!("Worker thread has now real-time priority");
        Some(handle)
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

  fn demote_from_real_time(handle: Option<RtPriorityHandle>) {
    handle.into_iter().for_each(
      |handle| match demote_current_thread_from_real_time(handle) {
        Ok(_ok) => debug!("Restored Worker thread to normal priority"),
        Err(err) => warn!(
          "Couldn't bring the Worker thread back to normal priority: {:?}",
          err
        ),
      },
    );
  }
}
