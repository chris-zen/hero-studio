use std::rc::Rc;

use log::{debug, info};

use failure;
use failure::{Error, Fail};

use crossbeam_channel::{Receiver, Sender};

use hero_studio_core::{config::Config as StudioConfig, studio::Studio, time::BarsTime};

mod config;
use crate::config::Config as AppConfig;

mod midi;
use crate::midi::io::{MidiIo, Protocol as MidiOutputProtocol};

mod audio;
use crate::audio::callback::{AudioCallback, Protocol as AudioProtocol};
use crate::audio::drivers::portaudio::{PortAudioDriver, PortAudioStream};

mod controller;
use crate::controller::{Controller, Protocol as ControllerProtocol};

mod server;
use crate::server::{Message as ServerMessage, Server};

//mod events;

mod realtime;

//const APP_NAME: &'static str = "Hero Studio";

const HERO_STUDIO_CONFIG: &str = "HERO_STUDIO_CONFIG";
const DEFAULT_HERO_STUDIO_CONFIG: &str = "studio.toml";

const HERO_STUDIO_APP_CONFIG: &str = "HERO_STUDIO_APP_CONFIG";
const DEFAULT_HERO_STUDIO_APP_CONFIG: &str = "app.toml";

const HERO_STUDIO_LOG_CONFIG: &str = "HERO_STUDIO_LOG_CONFIG";
const DEFAULT_HERO_STUDIO_LOG_CONFIG: &str = "log4rs.yaml";

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

  let (server_tx, server_rx) = Server::new_channel();
  let (ctrl_tx, ctrl_rx) = Controller::new_channel();
  let (audio_tx, audio_rx) = PortAudioStream::new_channel();
  let (midi_tx, midi_rx) = MidiIo::new_channel();

  let midi_output = MidiIo::new(
    midi_config,
    audio_config,
    midi_tx.clone(),
    midi_rx.clone(),
    ctrl_tx.clone(),
  )?;

  let studio = init_studio(studio_config)?;

  let (_audio_driver, mut stream) = init_audio(studio, audio_rx.clone(), midi_tx.clone())?;

  let controller = Controller::new(
    ctrl_tx.clone(),
    ctrl_rx.clone(),
    audio_tx.clone(),
    midi_tx.clone(),
  )?;

  let server = init_server(
    websocket_port,
    server_tx.clone(),
    server_rx.clone(),
    ctrl_tx.clone(),
  )?;

  drop(server_tx);
  drop(server_rx);
  drop(ctrl_tx);
  drop(ctrl_rx);
  drop(audio_tx);
  drop(audio_rx);
  drop(midi_tx);
  drop(midi_rx);

  stream.wait();

  server.close();

  controller.stop()?;

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
  studio: Studio,
  audio_rx: Receiver<AudioProtocol>,
  midi_output_tx: Sender<MidiOutputProtocol>,
) -> Result<(Rc<PortAudioDriver>, PortAudioStream), Error> {
  info!("Initialising audio ...");

  let audio_config = &studio.config().audio.clone();

  let driver = PortAudioDriver::new().map(Rc::new)?;
  let audio_callback = AudioCallback::new(studio, audio_rx, midi_output_tx);
  let mut stream = PortAudioStream::new(driver.clone(), audio_config, audio_callback)?;
  stream.start()?;

  Ok((driver, stream))
}

fn init_server(
  port: u16,
  server_tx: Sender<ServerMessage>,
  server_rx: Receiver<ServerMessage>,
  ctrl_tx: Sender<ControllerProtocol>,
) -> Result<Server, Error> {
  info!("Initialising the websocket server ...");

  let server = Server::new(port, server_tx, server_rx, ctrl_tx)?;

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
