use std::rc::Rc;
use std::sync::{Arc, RwLock};

use log::{debug, info};

use failure;
use failure::{Error, Fail};

use hero_studio_core::midi::bus::{BusAddress, MidiBus};

use hero_studio_core::{
  config::Audio as AudioConfig, config::Config as StudioConfig, studio::Studio,
  time::BarsTime,
};

mod config;
use crate::config::Config as AppConfig;

mod midi;
use crate::midi::{Midi, MidiDriver, MidiError, PORT_MIDI_ID /*, CORE_MIDI_ID*/};

mod audio;
use crate::audio::{PortAudioDriver, PortAudioStream};

mod server;
use crate::server::Server;

mod events;

mod realtime_thread;

mod workers;
use crate::workers::Workers;

const APP_NAME: &'static str = "Hero Studio";

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

  // #[fail(display = "Unable to lock studio for write")]
  // StudioWriteLock,
  #[fail(display = "Failed to get a MIDI driver: {}", cause)]
  GetMidiDriver { cause: MidiError },
}

fn main() -> Result<(), Error> {
  init_logging()?;

  let app_config = init_app_config()?;
  let websocket_port = app_config.websocket.port;

  let studio_config = init_studio_config()?;

  let (_midi_driver, midi_bus) = init_midi(&studio_config)?;

  let studio = init_studio(studio_config, midi_bus)?;

  let (_audio_driver, mut stream) = init_audio(&studio.config().audio)?;

  let worker = init_workers(studio, app_config, &stream)?;

  let server = init_server(websocket_port)?;

  stream.wait();

  worker.close();

  server.close();

  stream.stop()?;
  stream.close()?;

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

fn init_midi(_config: &StudioConfig) -> Result<(Box<dyn MidiDriver>, MidiBus), Error> {
  info!("Initialising MIDI ...");

  let midi = Midi::new();
  let mut midi_bus = MidiBus::new();

  // TODO create a driver from the configuration
  // let midi_driver_id = *midi.drivers().first().unwrap();
  let midi_driver_id = PORT_MIDI_ID;
  // let midi_driver_id = CORE_MIDI_ID;

  let midi_driver = midi
    .driver(midi_driver_id, APP_NAME)
    .map_err(|cause| MainError::GetMidiDriver { cause })?;

  debug!("MIDI Driver: {:?}", midi_driver.id());

  debug!("Destinations:");
  for destination in midi_driver.destinations() {
    debug!("=> {:?}", destination.name());
    if let Ok(bus_node) = destination.open() {
      debug!(
        "   Adding MIDI destination to the bus: {}",
        destination.name()
      );
      midi_bus.add_node(&BusAddress::new(), bus_node);
    }
  }

  Ok((midi_driver, midi_bus))
}

fn init_studio(config: StudioConfig, midi_bus: MidiBus) -> Result<Studio, Error> {
  info!("Initialising the studio ...");

  let midi_bus = Arc::new(RwLock::new(midi_bus));

  let mut studio = Studio::new(config, midi_bus);

  studio.set_loop_end(BarsTime::new(2, 0, 0, 0));
  studio.play(true);

  Ok(studio)
}

fn init_audio(audio_config: &AudioConfig) -> Result<(Rc<PortAudioDriver>, PortAudioStream), Error> {
  info!("Initialising audio ...");

  let driver = PortAudioDriver::new().map(Rc::new)?;
  let mut stream = PortAudioStream::new(driver.clone(), audio_config)?;
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

fn init_workers(
  studio: Studio,
  app_config: AppConfig,
  stream: &PortAudioStream,
) -> Result<Workers, Error> {
  let workers = Workers::new();

  let (audio_tx, audio_rx) = stream.channel();

  workers.start(studio, app_config, audio_tx, audio_rx);

  Ok(workers)
}

// fn init_reactor(server: &Server, audio_rx: Receiver<Event>) -> Result<Reactor, Error> {
//   let reactor = Reactor::new(server, audio_rx);
//   Ok(reactor)
// }
