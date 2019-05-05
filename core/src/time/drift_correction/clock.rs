use crate::time::{clock, ClockTime, SampleRate};

#[derive(Clone)]
pub struct ClockDriftCorrection {
  time_per_sample: ClockTime,
  error_per_sample: f64,
  error_accumulated: f64,
  last_correction: f64,
}

impl ClockDriftCorrection {
  pub fn new(sample_rate: SampleRate) -> ClockDriftCorrection {
    ClockDriftCorrection {
      time_per_sample: ClockTime::from_samples(1, sample_rate),
      error_per_sample: ClockDriftCorrection::error_per_sample(sample_rate),
      error_accumulated: 0.0,
      last_correction: 0.0,
    }
  }

  pub fn error_per_sample(sample_rate: SampleRate) -> f64 {
    let time_per_sample = ClockTime::from_samples(1, sample_rate);
    let error_per_second = ClockTime::from_seconds(1.0) - (time_per_sample * sample_rate);
    error_per_second.units() as f64 / f64::from(sample_rate)
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

  pub fn next(&mut self, samples: u32) -> ClockTime {
    let samples_time = self.time_per_sample * samples;
    let total_error = self.error_accumulated + self.error_per_sample * f64::from(samples);
    if total_error.abs() >= 1.0 {
      self.last_correction = total_error.round();
      self.error_accumulated = total_error - self.last_correction;
      samples_time + ClockTime::new(self.last_correction as clock::UnitType)
    } else {
      self.last_correction = 0.0;
      self.error_accumulated = total_error;
      samples_time
    }
  }
}

#[cfg(test)]
mod test {

  use super::ClockDriftCorrection;
  use super::ClockTime;

  #[test]
  pub fn clock_drift_correction_new() {
    let correction = ClockDriftCorrection::new(44100);
    assert_eq!(correction.time_per_sample, ClockTime::new(22675));
    assert_eq!(correction.error_per_sample, 0.736_961_451_247_165_5);
    assert_eq!(correction.error_accumulated, 0.0);
    assert_eq!(correction.last_correction, 0.0);
  }

  #[test]
  pub fn clock_drift_correction_error_per_sample() {
    let error = ClockDriftCorrection::error_per_sample(44100);
    assert_eq!(error, 0.736_961_451_247_165_5);
  }

  #[test]
  pub fn clock_drift_correction_next() {
    let mut correction = ClockDriftCorrection::new(44100);
    let samples_time = correction.next(100);
    assert_eq!(samples_time, ClockTime::new(2_267_574));
    let samples_time = correction.next(100);
    assert_eq!(samples_time, ClockTime::new(2_267_573));
    let samples_time = correction.next(100);
    assert_eq!(samples_time, ClockTime::new(2_267_574));
    let samples_time = correction.next(100);
    assert_eq!(samples_time, ClockTime::new(2_267_574));
    let samples_time = correction.next(100);
    assert_eq!(samples_time, ClockTime::new(2_267_573));
  }
}
