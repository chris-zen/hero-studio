pub mod clock;
pub mod ticks;
pub mod bars;
pub mod signature;
pub mod tempo;

pub use self::clock::ClockTime;
pub use self::ticks::TicksTime;
pub use self::bars::BarsTime;
pub use self::signature::Signature;
pub use self::tempo::Tempo;

pub type SampleRate = u32;

pub struct TimeDriftCorrection {
  time_per_sample: ClockTime,
  error_per_sample: f64,
  error_accumulated: f64,
  last_correction: f64,
}

impl TimeDriftCorrection {

  pub fn new(sample_rate: SampleRate) -> TimeDriftCorrection {
    TimeDriftCorrection {
      time_per_sample: ClockTime::from_samples(1, sample_rate),
      error_per_sample: TimeDriftCorrection::error_per_sample(sample_rate),
      error_accumulated: 0.0,
      last_correction: 0.0
    }
  }

  pub fn error_per_sample(sample_rate: SampleRate) -> f64 {
    let time_per_sample = ClockTime::from_samples(1, sample_rate);
    let error_per_second = ClockTime::from_seconds(1.0) - (time_per_sample * sample_rate);
    error_per_second.units() as f64 / sample_rate as f64
  }

  pub fn get_time_per_sample(&self) -> ClockTime {
    self.time_per_sample
  }

  pub fn get_error_per_sample(&self) -> f64 {
    self.error_per_sample
  }

  pub fn get_error_accumulated(&self) -> f64 {
    self.error_accumulated
  }

  pub fn get_last_correction(&self) -> f64 {
    self.last_correction
  }

  pub fn update(&mut self, samples: u32) -> ClockTime {
    let samples_time = self.time_per_sample * samples;
    let total_error = self.error_accumulated + self.error_per_sample * samples as f64;
    if total_error >= 1.0 {
      self.last_correction = total_error.round();
      self.error_accumulated = total_error - self.last_correction;
      samples_time + ClockTime::new(self.last_correction as clock::UnitType)
    }
    else {
      self.last_correction = 0.0;
      self.error_accumulated = total_error;
      samples_time
    }
  }
}

#[cfg(test)]
mod test {

  use super::ClockTime;
  use super::TimeDriftCorrection;

  #[test]
  pub fn time_drift_correction_new() {
    let correction = TimeDriftCorrection::new(44100);
    assert_eq!(correction.time_per_sample, ClockTime::new(22675736));
    assert_eq!(correction.error_per_sample, 0.9614512471655329);
    assert_eq!(correction.error_accumulated, 0.0);
    assert_eq!(correction.last_correction, 0.0);
  }

  #[test]
  pub fn time_drift_correction_error_per_sample() {
    let error = TimeDriftCorrection::error_per_sample(44100);
    assert_eq!(error, 0.9614512471655329);
  }

  #[test]
  pub fn time_drift_correction_update() {
    let mut correction = TimeDriftCorrection::new(44100);
    let samples_time = correction.update(100);
    assert_eq!(samples_time, ClockTime::new(2267573696));
    let samples_time = correction.update(100);
    assert_eq!(samples_time, ClockTime::new(2267573696));
    let samples_time = correction.update(100);
    assert_eq!(samples_time, ClockTime::new(2267573696));
    let samples_time = correction.update(100);
    assert_eq!(samples_time, ClockTime::new(2267573697));
  }
}
