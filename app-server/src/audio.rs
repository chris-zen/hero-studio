use failure::Error;
// use failure_derive;

use std::sync::{Arc, RwLock};

use portaudio::{
  DuplexStreamCallbackArgs, DuplexStreamSettings, PortAudio, Stream, StreamParameters,
};

use hero_studio_core::config::Audio as AudioConfig;
use hero_studio_core::studio::{AudioTime, Studio};

const CHANNELS: i32 = 2;
const INTERLEAVED: bool = true;

// #[derive(Debug, Fail)]
// enum AudioError {
//   #[fail(display = "Unable to access studio configuration")]
//   StudioConfig,
// }

type PortAudioStream = Stream<portaudio::NonBlocking, portaudio::Duplex<f32, f32>>;

pub fn audio_start(
  pa: &PortAudio,
  audio_config: AudioConfig,
  studio_lock: Arc<RwLock<Studio>>,
) -> Result<PortAudioStream, Error> {
  println!("PortAudio:");
  println!("version: {}", pa.version());
  println!("version text: {:?}", pa.version_text());
  println!("host count: {}", pa.host_api_count()?);

  let default_host = pa.default_host_api()?;
  println!("default host: {:#?}", pa.host_api_info(default_host));

  println!("All devices:");
  for device in pa.devices()? {
    let (idx, info) = device?;
    println!("--------------------------------------- {:?}", idx);
    println!("{:#?}", &info);
  }

  // TODO get device from config
  let def_input = pa.default_input_device()?;
  let input_info = pa.device_info(def_input)?;
  println!("Default input device info: {:#?}", &input_info);

  // Construct the input stream parameters.
  let latency = input_info.default_low_input_latency;
  let input_params = StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

  let def_output = pa.default_output_device()?;
  let output_info = pa.device_info(def_output)?;
  println!("Default output device info: {:#?}", &output_info);

  // Construct the output stream parameters.
  let latency = output_info.default_low_output_latency;
  let output_params = StreamParameters::<f32>::new(def_output, CHANNELS, INTERLEAVED, latency);

  // Check that the stream format is supported.
  let sample_rate = audio_config.sample_rate as f64;
  pa.is_duplex_format_supported(input_params, output_params, sample_rate)?;

  // Construct the settings with which we'll open our duplex stream.
  let num_frames = audio_config.frames as u32;
  let settings = DuplexStreamSettings::new(input_params, output_params, sample_rate, num_frames);

  // A callback to pass to the non-blocking stream.
  let callback = move |DuplexStreamCallbackArgs {
                         in_buffer,
                         out_buffer,
                         frames,
                         time,
                         ..
                       }| {
    studio_lock
      .write()
      .map(|mut studio| {
        let audio_time = AudioTime::new(time.current, time.in_buffer_adc, time.out_buffer_dac);
        // TODO strategy to handle errors
        studio.audio_handler(audio_time, frames, in_buffer, out_buffer);
        portaudio::Continue
      })
      .unwrap_or(portaudio::Complete)
  };

  // Construct a stream with input and output sample types of f32.
  let mut stream = pa.open_non_blocking_stream(settings, callback)?;

  stream.start()?;

  Ok(stream)
}

pub fn audio_close(stream: &mut PortAudioStream) -> Result<(), portaudio::Error> {
  println!("Stopping and closing the stream ...");
  stream.stop()?;
  stream.close()
}
