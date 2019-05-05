#[cfg(target_os = "macos")]
use audio_thread_priority::{
  demote_current_thread_from_real_time, promote_current_thread_to_real_time, RtPriorityHandle,
};

pub type Result<T> = std::result::Result<T, RealTimeAudioPriorityError>;

const ERROR_MSG: &str = "Thread could not be promoted to real time";

#[derive(Debug, Clone)]
pub struct RealTimeAudioPriorityError {}

impl std::fmt::Display for RealTimeAudioPriorityError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", ERROR_MSG)
  }
}

impl std::error::Error for RealTimeAudioPriorityError {
  fn description(&self) -> &str {
    ERROR_MSG
  }

  fn cause(&self) -> Option<&std::error::Error> {
    None
  }
}

pub struct RealTimeAudioPriority {
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  handle: Option<RtPriorityHandle>,
}

impl RealTimeAudioPriority {
  pub fn promote(sample_rate: u32, buffer_size: u32) -> Result<RealTimeAudioPriority> {
    Self::promote_rt(sample_rate, buffer_size)
  }

  #[cfg(any(target_os = "macos", target_os = "windows"))]
  fn promote_rt(sample_rate: u32, buffer_size: u32) -> Result<RealTimeAudioPriority> {
    promote_current_thread_to_real_time(buffer_size, sample_rate)
      .map(|handle| RealTimeAudioPriority {
        handle: Some(handle),
      })
      .map_err(|_err| RealTimeAudioPriorityError {})
  }

  #[cfg(any(target_os = "macos", target_os = "windows"))]
  fn demote_rt(&mut self) {
    self.handle.take().into_iter().for_each(|handle| {
      let _ = demote_current_thread_from_real_time(handle);
    });
  }

  #[cfg(target_os = "linux")]
  fn promote_rt(sample_rate: u32, buffer_size: u32) -> Result<RealTimeAudioPriority> {
    // TODO try something with thread_priority or rtkit
    Ok(RealTimeAudioPriority {})
  }

  #[cfg(target_os = "linux")]
  fn demote_rt(&mut self) {}

  #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
  fn promote_rt(sample_rate: u32, buffer_size: u32) -> Result<RealTimeAudioPriority> {
    Ok(RealTimeAudioPriority {})
  }

  #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
  fn demote_rt(&mut self) {}
}

impl Drop for RealTimeAudioPriority {
  fn drop(&mut self) {
    self.demote_rt();
  }
}
