use std::rc::Rc;

use log::{debug, info};

use failure;
use failure::{Error, Fail};

use crossbeam_channel::{Receiver, Sender};

use hero_studio_core::{
  config::Audio as AudioConfig, config::Config as StudioConfig, studio::Studio, time::BarsTime,
};

mod config;
use crate::config::Config as AppConfig;

mod midi;
use crate::midi::output::{MidiOutput, Protocol as MidiOutputProtocol};

mod audio;
use crate::audio::drivers::portaudio::{PortAudioDriver, PortAudioStream};
use crate::audio::io::{AudioIo, Protocol as AudioProtocol};

mod studio_workers;
use crate::studio_workers::{Protocol as StudioProtocol, StudioWorkers};

mod server;
use crate::server::Server;

//mod events;

mod realtime_thread;

//const APP_NAME: &'static str = "Hero Studio";

const HERO_STUDIO_CONFIG: &'static str = "HERO_STUDIO_CONFIG";
const DEFAULT_HERO_STUDIO_CONFIG: &'static str = "studio.toml";

const HERO_STUDIO_APP_CONFIG: &'static str = "HERO_STUDIO_APP_CONFIG";
const DEFAULT_HERO_STUDIO_APP_CONFIG: &'static str = "app.toml";

const HERO_STUDIO_LOG_CONFIG: &'static str = "HERO_STUDIO_LOG_CONFIG";
const DEFAULT_HERO_STUDIO_LOG_CONFIG: &'static str = "log4rs.yaml";

#[derive(Debug, Fail)]
enum MainError {
  #[fail(display = "Failed to init logging: {}", cause)]
  LoggingInit { cause: String },
}

fn main() -> Result<(), Error> {
  init_logging()?;

  let app_config = init_app_config()?;
  let websocket_port = app_config.websocket.port;

  let studio_config = init_studio_config()?;
  let midi_config = &studio_config.midi;
  let audio_config = &studio_config.audio;

  let (studio_tx, studio_rx) = StudioWorkers::new_channel();
  let (audio_tx, audio_rx) = PortAudioStream::new_channel();
  let (midi_out_tx, midi_out_rx) = MidiOutput::new_channel();

  let midi_output = MidiOutput::new(
    midi_config,
    audio_config,
    midi_out_tx.clone(),
    midi_out_rx.clone(),
    studio_tx.clone(),
  )?;

  //  info!("Waiting for MIDI output to initialise ...");
  //  drop(studio_rx.recv());

  let (_audio_driver, mut stream) = init_audio(
    audio_config,
    audio_rx.clone(),
    studio_tx.clone(),
    midi_out_tx.clone(),
  )?;

  let studio = init_studio(studio_config)?;
  let studio_workers = StudioWorkers::new(
    studio,
    app_config,
    studio_tx.clone(),
    studio_rx.clone(),
    audio_tx.clone(),
    midi_out_tx.clone(),
    stream.num_input_channels(),
    stream.num_output_channels(),
  )?;

  let server = init_server(websocket_port)?;

  drop(studio_tx);
  drop(studio_rx);
  drop(audio_tx);
  drop(audio_rx);
  drop(midi_out_tx);
  drop(midi_out_rx);

  stream.wait();

  server.close();

  studio_workers.stop()?;

  stream.stop()?;
  stream.close()?;

  midi_output.stop()?;

  Ok(())
}

fn init_logging() -> Result<(), Error> {
  let log_config_path = std::env::var(HERO_STUDIO_LOG_CONFIG)
    .unwrap_or_else(|_| DEFAULT_HERO_STUDIO_LOG_CONFIG.to_string());

  log4rs::init_file(log_config_path.as_str(), Default::default()).map_err(|err| {
    MainError::LoggingInit {
      cause: err.to_string(),
    }
  })?;

  Ok(())
}

fn init_app_config() -> Result<AppConfig, Error> {
  let config_path = std::env::var(HERO_STUDIO_APP_CONFIG)
    .unwrap_or_else(|_| DEFAULT_HERO_STUDIO_APP_CONFIG.to_string());

  info!("Loading app configuration from {} ...", config_path);
  let config = AppConfig::from_file(config_path.as_str())?;
  debug!("{:#?}", config);

  Ok(config)
}

fn init_studio_config() -> Result<StudioConfig, Error> {
  let config_path =
    std::env::var(HERO_STUDIO_CONFIG).unwrap_or_else(|_| DEFAULT_HERO_STUDIO_CONFIG.to_string());

  info!("Loading studio configuration from {} ...", config_path);
  let config = StudioConfig::from_file(config_path.as_str())?;
  debug!("{:#?}", config);

  Ok(config)
}

fn init_studio(config: StudioConfig) -> Result<Studio, Error> {
  info!("Initialising the studio ...");

  let mut studio = Studio::new(config);

  studio.set_loop_end(BarsTime::new(2, 0, 0, 0));
  studio.play(true);

  Ok(studio)
}

fn init_audio(
  audio_config: &AudioConfig,
  audio_rx: Receiver<AudioProtocol>,
  studio_tx: Sender<StudioProtocol>,
  midi_output_tx: Sender<MidiOutputProtocol>,
) -> Result<(Rc<PortAudioDriver>, PortAudioStream), Error> {
  info!("Initialising audio ...");

  let driver = PortAudioDriver::new().map(Rc::new)?;
  let audio_io = AudioIo::new(audio_rx, studio_tx, midi_output_tx);
  let mut stream = PortAudioStream::new(driver.clone(), audio_config, audio_io)?;
  stream.start()?;

  Ok((driver, stream))
}

fn init_server(port: u16) -> Result<Server, Error> {
  info!("Initialising the websocket server ...");

  let server = Server::new(port)?;

  let receiver = server.receiver();

  std::thread::spawn(move || {
    for msg in receiver.iter() {
      debug!("Received {:#?}", msg);
    }
  });

  // let sender = server.sender();
  // std::thread::spawn(move || {
  //   let mut count = 0;
  //   loop {
  //     let data = format!("{:?}", count);
  //     drop(sender.send(Message::Outgoing { port: ALL_PORTS, data: data.into_bytes() } ));
  //     std::thread::sleep_ms(1);
  //     count += 1;
  //   }
  // });

  Ok(server)
}
