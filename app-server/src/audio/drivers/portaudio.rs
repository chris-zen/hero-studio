use std::rc::Rc;
use std::time::Duration;

use log::{debug, error, info, trace};

use crossbeam_channel::{Receiver, Sender};

use portaudio::{
  stream::callback_flags, stream::callback_flags::CallbackFlags, DuplexStreamCallbackArgs,
  DuplexStreamSettings, PortAudio, Stream, StreamParameters,
};

use hero_studio_core::audio::{AudioInput, AudioOutput};
use hero_studio_core::config::Audio as AudioConfig;
use hero_studio_core::time::ClockTime;

use crate::audio::callback::Protocol;
use crate::audio::callback::{AudioCallback, AudioCallbackResult};
use crate::audio::drivers::{AudioError, AudioResult};

const INTERLEAVED: bool = true;

impl From<portaudio::error::Error> for AudioError {
  fn from(cause: portaudio::error::Error) -> AudioError {
    AudioError::DriverError {
      cause: cause.to_string(),
    }
  }
}

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
}

pub struct PortAudioStream {
  driver: Rc<PortAudioDriver>,
  stream: PaStream,
}

impl PortAudioStream {
  // TODO Use an spsc array when published by crossbeam
  pub const CHANNEL_CAPACITY: usize = 128 * 1024;
  pub fn new_channel() -> (Sender<Protocol>, Receiver<Protocol>) {
    crossbeam_channel::bounded::<Protocol>(Self::CHANNEL_CAPACITY)
  }

  pub fn new(
    driver: Rc<PortAudioDriver>,
    config: &AudioConfig,
    mut audio_io: AudioCallback,
  ) -> AudioResult<PortAudioStream> {
    info!("Creating an audio stream ...");

    let portaudio = &driver.portaudio;

    // TODO get devices to use from config

    let def_output = portaudio.default_output_device()?;
    let output_info = portaudio.device_info(def_output)?;
    debug!("Output device info: {:#?}", &output_info);

    let def_input = portaudio.default_input_device()?;
    let input_info = portaudio.device_info(def_input)?;
    debug!("Input device info: {:#?}", &input_info);

    // Construct the stream parameters
    let latency = input_info.default_low_input_latency;
    let input_params = StreamParameters::<f32>::new(
      def_input,
      input_info.max_input_channels,
      INTERLEAVED,
      latency,
    );

    let latency = output_info.default_low_output_latency;
    let output_params = StreamParameters::<f32>::new(
      def_output,
      output_info.max_output_channels,
      INTERLEAVED,
      latency,
    );

    let sample_rate = f64::from(config.sample_rate);
    portaudio.is_duplex_format_supported(input_params, output_params, sample_rate)?;

    // Construct the duplex stream
    let num_frames = u32::from(config.frames);
    let settings = DuplexStreamSettings::new(input_params, output_params, sample_rate, num_frames);

    let num_input_channels = input_info.max_input_channels as usize;
    let num_output_channels = output_info.max_output_channels as usize;

    let starting = true;
    let callback = move |args| {
      Self::callback(
        args,
        &mut audio_io,
        num_input_channels as usize,
        num_output_channels as usize,
        starting,
      )
    };

    let stream = portaudio.open_non_blocking_stream(settings, callback)?;

    Ok(PortAudioStream { driver, stream })
  }

  fn callback(
    args: DuplexStreamCallbackArgs<f32, f32>,
    audio_io: &mut AudioCallback,
    in_channels: usize,
    out_channels: usize,
    starting: bool,
  ) -> portaudio::stream::CallbackResult {
    let DuplexStreamCallbackArgs {
      in_buffer,
      out_buffer,
      frames,
      time,
      flags,
    } = args;

    Self::detect_and_report_xrun(starting, time.out_buffer_dac, flags);

    let in_time = ClockTime::from_seconds(time.in_buffer_adc);
    let out_time = ClockTime::from_seconds(time.out_buffer_dac);
    let audio_input = AudioInput::new(in_time, in_channels, in_buffer);
    let audio_output = AudioOutput::new(out_time, out_channels, out_buffer);
    match audio_io.process(frames, audio_input, audio_output) {
      Ok(AudioCallbackResult::Continue) => portaudio::Continue,
      Ok(AudioCallbackResult::Stop) => portaudio::Complete,
      Err(_err) => {
        // TODO handle error
        error!("{}", _err.to_string());
        portaudio::Complete
      }
    }
  }

  pub fn start(&mut self) -> AudioResult<()> {
    info!("Starting the audio stream ...");
    self.stream.start().map_err(Into::into)
  }

  pub fn wait(&self) {
    while let Ok(true) = self.stream.is_active() {
      self.driver.sleep(Duration::from_secs(1));
    }
  }

  pub fn stop(&mut self) -> AudioResult<()> {
    info!("Stopping the audio stream ...");
    self.stream.stop().map_err(Into::into)
  }

  pub fn close(mut self) -> AudioResult<()> {
    info!("Closing the audio stream ...");
    self.stream.close().map_err(Into::into)
  }

  fn detect_and_report_xrun(mut starting: bool, output_time: f64, flags: CallbackFlags) {
    if starting && flags != callback_flags::INPUT_UNDERFLOW {
      starting = false;
    }

    if !starting && !flags.is_empty() {
      let nanos = ClockTime::from_seconds(output_time).to_nanos();
      let microseconds = nanos / 1000;
      let seconds = nanos / 1_000_000_000;
      let minutes = seconds / 60;
      let hours = minutes / 60;
      debug!(
        "xrun {}:{:02}:{:02}:{:06} {:?}",
        hours,
        minutes % 60,
        seconds % 60,
        microseconds % 1_000_000,
        flags
      );
      // TODO measure xrun rate and stop if too high
    }
  }
}
