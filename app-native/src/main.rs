use std::collections::HashSet;
use std::sync::{Arc, Mutex, RwLock};

use failure;
use failure::{Error, Fail};
use failure_derive;

use portaudio;

use hero_studio_core::midi::bus::{BusAddress, BusNode, MidiBus, NodeClass, NodeFeature};
use hero_studio_core::{config::Config, studio::Studio, time::BarsTime};

mod midi;
use crate::midi::Midi;

mod audio;
use crate::audio::{audio_close, audio_start};

const HERO_STUDIO_CONFIG: &'static str = "HERO_STUDIO_CONFIG";
const DEFAULT_HERO_STUDIO_CONFIG: &'static str = "studio.toml";

#[derive(Debug, Fail)]
enum MainError {
  #[fail(display = "Unable to lock studio for write")]
  StudioWriteLock,
}

pub struct FakeBusNode<'a> {
  name: &'a str,
  class: NodeClass,
  features: HashSet<NodeFeature>,
}

impl<'a> BusNode for FakeBusNode<'a> {
  fn name(&self) -> &str {
    self.name
  }

  fn class(&self) -> NodeClass {
    self.class
  }
}

fn main() -> Result<(), Error> {
  let config_path =
    std::env::var(HERO_STUDIO_CONFIG).unwrap_or(DEFAULT_HERO_STUDIO_CONFIG.to_string());

  let midi = Midi::init();
  let midi_mutex = Arc::new(Mutex::new(midi));

  let mut midi_bus = MidiBus::new();
  let fake_bus_node = FakeBusNode {
    name: "fake",
    class: NodeClass::MidiDestination,
    features: HashSet::from_iter(std::iter::once(NodeFeature::Default)),
  };
  let fake_bus_addr = BusAddress::new();
  midi_bus.add_node(&fake_bus_addr, Arc::new(RwLock::new(fake_bus_node)));
  let midi_bus = Arc::new(RwLock::new(midi_bus));

  let config = Config::from_file(config_path.as_str())?;
  let mut studio = Studio::new(config.clone(), midi_bus);
  studio.song_mut().set_loop_end(BarsTime::new(2, 0, 0, 0));

  let studio_lock = Arc::new(RwLock::new(studio));

  println!("{:#?}", config);

  let pa_ctx = portaudio::PortAudio::new()?;

  let mut stream = audio_start(&pa_ctx, midi_mutex.clone(), studio_lock.clone())?;

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
