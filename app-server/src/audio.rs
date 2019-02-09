use std::rc::Rc;
use std::time::Duration;

use log::{debug, info, trace};

use failure::{Error, Fail};
// use failure_derive;

use crossbeam_channel::{Receiver, Sender};

use portaudio::{
  DuplexStreamCallbackArgs, DuplexStreamSettings, PortAudio, Stream, StreamParameters,
};

use hero_studio_core::config::Audio as AudioConfig;
use hero_studio_core::midi::messages::Message;
use hero_studio_core::studio::AudioTime;

// use crate::events::Event;

const INTERLEAVED: bool = true;
pub const CHANNELS: i32 = 2;
pub const MAX_FRAMES: usize = 4 * 1024;

#[derive(Debug, Fail)]
pub enum AudioError {
  #[fail(display = "PortAudio error: {}", cause)]
  PortAudioError { cause: portaudio::error::Error },
}

impl From<portaudio::error::Error> for AudioError {
  fn from(cause: portaudio::error::Error) -> AudioError {
    AudioError::PortAudioError { cause }
  }
}

type AudioResult<T> = Result<T, AudioError>;

type PaStream = Stream<portaudio::NonBlocking, portaudio::Duplex<f32, f32>>;

pub struct PortAudioDriver {
  portaudio: PortAudio,
}

impl PortAudioDriver {
  pub fn new() -> AudioResult<PortAudioDriver> {
    let portaudio = PortAudio::new()?;

    trace!("PortAudio:");
    trace!("  version: {}", portaudio.version());
    trace!("  version text: {:?}", portaudio.version_text());
    trace!("  host count: {}", portaudio.host_api_count()?);

    let default_host = portaudio.default_host_api()?;
    trace!(
      "  default host: {:#?}",
      portaudio.host_api_info(default_host)
    );

    trace!("All devices:");
    for device in portaudio.devices()? {
      let (idx, info) = device?;
      trace!("[{:?}] ---------------------------------------", idx);
      trace!("{:#?}", &info);
    }

    Ok(PortAudioDriver { portaudio })
  }

  pub fn sleep(&self, duration: Duration) {
    self.portaudio.sleep(duration.as_millis() as i32);
  }

  pub fn close(self) -> AudioResult<()> {
    Ok(())
  }
}

pub struct AudioWork {
  pub audio: Box<[f32; MAX_FRAMES]>,
  pub time: Option<AudioTime>,
}

impl AudioWork {
  pub fn new(audio: Box<[f32; MAX_FRAMES]>) -> AudioWork {
    AudioWork { audio, time: None }
  }

  pub fn with_time(self, time: AudioTime) -> AudioWork {
    AudioWork {
      time: Some(time),
      ..self
    }
  }
}

pub struct PortAudioStream {
  driver: Rc<PortAudioDriver>,
  config: AudioConfig,
  stream: PaStream,
  work_tx: Sender<AudioWork>,
  work_rx: Receiver<AudioWork>,
  completed_tx: Sender<AudioWork>,
  completed_rx: Receiver<AudioWork>,
}

impl PortAudioStream {
  pub fn new(driver: Rc<PortAudioDriver>, config: AudioConfig) -> AudioResult<PortAudioStream> {
    info!("Creating an audio stream ...");

    let portaudio = &driver.portaudio;

    // TODO get devices to use from config

    let def_output = portaudio.default_output_device()?;
    let output_info = portaudio.device_info(def_output)?;
    trace!("Default output device info: {:#?}", &output_info);

    let def_input = portaudio.default_input_device()?;
    let input_info = portaudio.device_info(def_input)?;
    trace!("Default input device info: {:#?}", &input_info);

    // Construct the stream parameters
    let latency = input_info.default_low_input_latency;
    let input_params = StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

    let latency = output_info.default_low_output_latency;
    let output_params = StreamParameters::<f32>::new(def_output, CHANNELS, INTERLEAVED, latency);

    let sample_rate = config.sample_rate as f64;
    portaudio.is_duplex_format_supported(input_params, output_params, sample_rate)?;

    // Construct the duplex stream
    let num_frames = config.frames as u32;
    let settings = DuplexStreamSettings::new(input_params, output_params, sample_rate, num_frames);

    let (work_tx, work_rx) = crossbeam_channel::unbounded::<AudioWork>();
    let (completed_tx, completed_rx) = crossbeam_channel::unbounded::<AudioWork>();

    let cloned_work_rx = work_rx.clone();
    let cloned_completed_tx = completed_tx.clone();

    let callback = move |args| Self::callback(args, &cloned_work_rx, &cloned_completed_tx);

    let stream = portaudio.open_non_blocking_stream(settings, callback)?;

    Ok(PortAudioStream {
      driver,
      config,
      stream,
      work_tx,
      work_rx,
      completed_tx,
      completed_rx,
    })
  }

  fn callback(
    args: DuplexStreamCallbackArgs<f32, f32>,
    work_rx: &Receiver<AudioWork>,
    completed_tx: &Sender<AudioWork>,
  ) -> portaudio::stream::CallbackResult {
    let DuplexStreamCallbackArgs {
      // in_buffer,
      mut out_buffer,
      frames,
      time,
      ..
    } = args;

    let audio_time = AudioTime::new(time.current, time.in_buffer_adc, time.out_buffer_dac);

    match work_rx.try_recv() {
      Ok(work) => {
        let buffer_size = frames as usize * CHANNELS as usize;
        out_buffer.copy_from_slice(&work.audio[0..buffer_size]);

        drop(completed_tx.send(work.with_time(audio_time)));
      }
      Err(_err) => {
        Self::zero_fill(&mut out_buffer);
        debug!("xrun: {:?} {}", audio_time.output, _err);
        // TODO send xrun event
      }
    }

    portaudio::Continue
    // portaudio::Complete
  }

  fn zero_fill(s: &mut [f32]) {
    for d in s {
      *d = 0.0;
    }
  }

  pub fn channel(&self) -> (Sender<AudioWork>, Receiver<AudioWork>) {
    (self.work_tx.clone(), self.completed_rx.clone())
  }

  pub fn start(&mut self) -> AudioResult<()> {
    info!("Starting the audio stream ...");
    self.stream.start().map_err(|err| err.into())
  }

  pub fn wait(&self) {
    while let Ok(true) = self.stream.is_active() {
      self.driver.sleep(Duration::from_secs(1));
    }
  }

  pub fn stop(&mut self) -> AudioResult<()> {
    info!("Stopping the audio stream ...");
    self.stream.stop().map_err(|err| err.into())
  }

  pub fn close(mut self) -> AudioResult<()> {
    info!("Closing the audio stream ...");
    self.stream.close().map_err(|err| err.into())
  }
}

//   // A callback to pass to the non-blocking stream.
//   let callback = move |DuplexStreamCallbackArgs {
//                          in_buffer,
//                          out_buffer,
//                          frames,
//                          time,
//                          ..
//                        }| {
//     studio_lock
//       .write()
//       .map(|mut studio| {
//         let audio_time = AudioTime::new(time.current, time.in_buffer_adc, time.out_buffer_dac);
//         // TODO strategy to handle errors
//         studio.audio_handler(audio_time, frames, in_buffer, out_buffer);
//         // TODO send update event
//         portaudio::Continue
//       })
//       .unwrap_or(portaudio::Complete)
//   };
