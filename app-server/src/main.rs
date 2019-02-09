use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use log::{debug, info};

use failure;
use failure::{Error, Fail};
// use failure_derive;

use crossbeam_channel::Receiver;

use portaudio;

use hero_studio_core::midi::bus::{BusAddress, MidiBus};

use hero_studio_core::{
  config::Audio as AudioConfig, config::Config, config::ConfigLock, studio::Studio, time::BarsTime,
};

mod midi;
use crate::midi::{Midi, MidiDriver, MidiError, PORT_MIDI_ID /*, CORE_MIDI_ID*/};

mod audio;
use crate::audio::{PortAudioDriver, PortAudioStream};

mod server;
use crate::server::{Message, Server, ALL_PORTS};

// mod reactor;
// use crate::reactor::Reactor;

mod events;
use crate::events::Event;

mod worker;
use crate::worker::Worker;

const APP_NAME: &'static str = "Hero Studio";

const HERO_STUDIO_CONFIG: &'static str = "HERO_STUDIO_CONFIG";
const DEFAULT_HERO_STUDIO_CONFIG: &'static str = "studio.toml";

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

  let config = init_config()?;

  let audio_config = config.audio.clone();

  let (midi_driver, midi_bus) = init_midi(&config)?;

  let config_lock = Arc::new(RwLock::new(config));

  let studio = init_studio(config_lock.clone(), midi_bus)?;

  // let studio_lock = Arc::new(RwLock::new(studio));

  let (audio_driver, stream) = init_audio(audio_config.clone())?;

  let worker = init_worker(studio, audio_config.clone(), &stream)?;

  drop(audio_config);

  // TODO get port from config
  let server = init_server(3001)?;

  // let reactor = init_reactor(&server, audio_rx)?;

  // debug!("Started");
  // audio_driver.sleep(Duration::from_secs(1));

  // debug!("Play");
  // studio_lock
  //   .write()
  //   .map(|mut studio| studio.play(false))
  //   .map_err(|_err| MainError::StudioWriteLock)?;

  stream.wait();

  worker.close();

  server.close();

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

fn init_config() -> Result<Config, Error> {
  let config_path =
    std::env::var(HERO_STUDIO_CONFIG).unwrap_or_else(|_| DEFAULT_HERO_STUDIO_CONFIG.to_string());

  info!("Loading studio configuration from {} ...", config_path);
  let config = Config::from_file(config_path.as_str())?;
  debug!("{:#?}", config);

  Ok(config)
}

fn init_midi(_config: &Config) -> Result<(Box<dyn MidiDriver>, MidiBus), Error> {
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

fn init_studio(config: ConfigLock, midi_bus: MidiBus) -> Result<Studio, Error> {
  info!("Initialising the studio ...");

  let midi_bus = Arc::new(RwLock::new(midi_bus));

  let mut studio = Studio::new(config, midi_bus);

  studio.song_mut().set_loop_end(BarsTime::new(2, 0, 0, 0));

  Ok(studio)
}

fn init_audio(audio_config: AudioConfig) -> Result<(Rc<PortAudioDriver>, PortAudioStream), Error> {
  info!("Initialising audio ...");

  // let (audio_tx, audio_rx) = crossbeam_channel::unbounded::<Event>();

  let driver = PortAudioDriver::new().map(Rc::new)?;
  let mut stream = PortAudioStream::new(driver.clone(), audio_config)?;
  stream.start()?;

  Ok((driver, stream))
}

fn init_server(port: u16) -> Result<Server, Error> {
  info!("Initialising the websocket server ...");

  let server = Server::new(port)?;

  let receiver = server.receiver();
  let sender = server.sender();

  std::thread::spawn(move || {
    for msg in receiver.iter() {
      debug!("Received {:#?}", msg);
    }
  });

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

fn init_worker(
  studio: Studio,
  audio_config: AudioConfig,
  stream: &PortAudioStream,
) -> Result<Worker, Error> {
  let worker = Worker::new();

  let (audio_tx, audio_rx) = stream.channel();

  worker.start(studio, audio_config, audio_tx, audio_rx);

  Ok(worker)
}

// fn init_reactor(server: &Server, audio_rx: Receiver<Event>) -> Result<Reactor, Error> {
//   let reactor = Reactor::new(server, audio_rx);
//   Ok(reactor)
// }
