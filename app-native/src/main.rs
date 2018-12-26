use std::sync::{Arc, RwLock};

use failure;
use failure::{Fail, Error, SyncFailure};
use failure_derive;

use portaudio;

use hero_studio_core::{
    studio::Studio,
    config::Config,
    time::ClockTime
};

mod midi;

mod audio;
use crate::audio::{audio_start, audio_close};

const HERO_STUDIO_CONFIG: &'static str = "HERO_STUDIO_CONFIG";
const DEFAULT_HERO_STUDIO_CONFIG: &'static str = "studio.toml";

#[derive(Debug, Fail)]
enum MainError {
  #[fail(display = "Unable to lock studio for write")]
  StudioWriteLock
}

fn main() -> Result<(), Error> {
    let config_path = std::env::var(HERO_STUDIO_CONFIG)
        .unwrap_or(DEFAULT_HERO_STUDIO_CONFIG.to_string());

    let config = Config::from_file(config_path.as_str())?;
    let mut studio = Studio::new(config.clone());
    studio.song_mut()
        .get_transport_mut()
        .set_loop_end_time(ClockTime::from_seconds(4.0));
    let studio_mutex = Arc::new(RwLock::new(studio));

    println!("{:#?}", config);

    let pa_ctx = portaudio::PortAudio::new()?;

    let mut stream = audio_start(&pa_ctx, studio_mutex.clone())?;

    println!("Started");
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("Play");
    studio_mutex.write()
        .map(|mut studio| studio.play(false))
        .map_err(|_err| MainError::StudioWriteLock)?;

    // Loop while the non-blocking stream is active.
    while let Ok(true) = stream.is_active() {
        pa_ctx.sleep(1000);
    }

    audio_close(&mut stream)?;

    Ok(())
}
