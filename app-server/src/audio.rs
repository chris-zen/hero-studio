use std::time::Duration;
use std::rc::Rc;

use log::{debug, trace};

use failure::{Error, Fail};
// use failure_derive;

use crossbeam_channel::Sender;

use portaudio::{
  DuplexStreamCallbackArgs, DuplexStreamSettings,
  PortAudio, Stream, StreamParameters,
};

use hero_studio_core::config::Audio as AudioConfig;
// use hero_studio_core::studio::{AudioTime, Studio};

// use crate::events::Event;

const CHANNELS: i32 = 2;
const INTERLEAVED: bool = true;

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
    trace!("  default host: {:#?}", portaudio.host_api_info(default_host));

    trace!("All devices:");
    for device in portaudio.devices()? {
      let (idx, info) = device?;
      trace!("[{:?}] ---------------------------------------", idx);
      trace!("{:#?}", &info);
    }

    Ok(PortAudioDriver {
      portaudio
    })
  }

  pub fn sleep(&self, duration: Duration) {
    self.portaudio.sleep(duration.as_millis() as i32);
  }

  pub fn close(self) -> AudioResult<()> {
    Ok(())
  }
}

pub struct PortAudioStream {
  driver: Rc<PortAudioDriver>,
  config: AudioConfig,
  stream: PaStream,
}

impl PortAudioStream {
  pub fn new(driver: Rc<PortAudioDriver>, config: AudioConfig) -> AudioResult<PortAudioStream> {
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

    let callback = move |DuplexStreamCallbackArgs {
                          in_buffer,
                          out_buffer,
                          frames,
                          time,
                          ..
                        }| {

      // TODO ...

      portaudio::Continue
      // portaudio::Complete
    };

    let stream = portaudio.open_non_blocking_stream(settings, callback)?;

    Ok(PortAudioStream {
      driver,
      config,
      stream,
    })
  }

  pub fn start(&mut self) -> AudioResult<()> {
    debug!("Starting the audio stream ...");
    self.stream.start().map_err(|err| err.into())
  }

  pub fn wait(&self) {
    while let Ok(true) = self.stream.is_active() {
      self.driver.sleep(Duration::from_secs(1));
    }
  }

  pub fn stop(&mut self) -> AudioResult<()> {
    debug!("Stopping the audio stream ...");
    self.stream.stop().map_err(|err| err.into())
  }

  pub fn close(mut self) -> AudioResult<()> {
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
