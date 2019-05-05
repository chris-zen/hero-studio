pub mod portaudio;

use failure::Fail;

#[derive(Debug, Fail)]
pub enum AudioError {
  #[fail(display = "Driver error: {}", cause)]
  DriverError { cause: String },
}

type AudioResult<T> = Result<T, AudioError>;
