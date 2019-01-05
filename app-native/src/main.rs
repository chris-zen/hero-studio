use std::sync::{Arc, RwLock};

use failure;
use failure::{Error, Fail};
use failure_derive;

use portaudio;

use hero_studio_core::midi::bus::{BusAddress, MidiBus};

use hero_studio_core::{config::Config, studio::Studio, time::BarsTime};

mod midi;
use crate::midi::{Midi, MidiError};

mod audio;
use crate::audio::{audio_close, audio_start};

const APP_NAME: &'static str = "Hero Studio";

const HERO_STUDIO_CONFIG: &'static str = "HERO_STUDIO_CONFIG";
const DEFAULT_HERO_STUDIO_CONFIG: &'static str = "studio.toml";

#[derive(Debug, Fail)]
enum MainError {
  #[fail(display = "Unable to lock studio for write")]
  StudioWriteLock,

  #[fail(display = "Failed to get a MIDI driver: {}", cause)]
  GetMidiDriver { cause: MidiError },
}

fn main() -> Result<(), Error> {
  let config_path =
    std::env::var(HERO_STUDIO_CONFIG).unwrap_or_else(|_| DEFAULT_HERO_STUDIO_CONFIG.to_string());

  let config = Config::from_file(config_path.as_str())?;
  println!("{:#?}", config);

  let mut midi_bus = MidiBus::new();

  let midi = Midi::new();

  // TODO create a driver from the configuration
  let midi_driver_id = *midi.drivers().first().unwrap();

  let midi_driver = midi
    .driver(midi_driver_id, APP_NAME)
    .map_err(|cause| MainError::GetMidiDriver { cause })?;

  for destination in midi_driver.destinations() {
    if let Ok(bus_node) = destination.open() {
      println!("Adding MIDI destination to the bus: {}", destination.name());
      midi_bus.add_node(&BusAddress::new(), bus_node);
    }
  }

  let audio_config = config.audio.clone();
  let config_lock = Arc::new(RwLock::new(config));

  let midi_bus = Arc::new(RwLock::new(midi_bus));
  let mut studio = Studio::new(config_lock, midi_bus);
  studio.song_mut().set_loop_end(BarsTime::new(2, 0, 0, 0));

  let studio_lock = Arc::new(RwLock::new(studio));

  let pa_ctx = portaudio::PortAudio::new()?;

  let mut stream = audio_start(&pa_ctx, audio_config, studio_lock.clone())?;

  println!("Started");
  std::thread::sleep(std::time::Duration::from_secs(1));

  println!("Play");
  studio_lock
    .write()
    .map(|mut studio| studio.play(false))
    .map_err(|_err| MainError::StudioWriteLock)?;

  // Loop while the non-blocking stream is active.
  while let Ok(true) = stream.is_active() {
    pa_ctx.sleep(1000);
  }

  audio_close(&mut stream)?;

  Ok(())
}
