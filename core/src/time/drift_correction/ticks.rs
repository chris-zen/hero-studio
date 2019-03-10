use crate::time::{BarsTime, ClockTime, SampleRate, Signature, Tempo, TicksTime};

const SECONDS_PER_MINUTE: f64 = 60.0;

#[derive(Debug, Clone)]
pub struct TicksDriftCorrection {
  ticks_per_sample: f64,
  error_per_sample: f64,
  error_accumulated: f64,
  last_correction: f64,
}

impl TicksDriftCorrection {
  pub fn new(signature: Signature, tempo: Tempo, sample_rate: SampleRate) -> TicksDriftCorrection {
    let ticks_per_beat = f64::from(BarsTime::new(0, 1, 0, 0).to_ticks(signature));
    let ticks_per_sample =
      ticks_per_beat * f64::from(tempo) / (SECONDS_PER_MINUTE * sample_rate as f64);
    let error_per_second = f64::from(ClockTime::from_seconds(1.0).to_ticks(signature, tempo))
      - ticks_per_sample * sample_rate as f64;
    let error_per_sample = error_per_second / sample_rate as f64;
    // println!("ticks_per_sample={:?}, ticks_per_sec={:?}, ticks_per_sec_we={:?}, error_per_sec={:?}, error_per_sample={:?}",
    //   ticks_per_sample,
    //   f64::from(ClockTime::from_seconds(1.0).to_ticks(signature, tempo)),
    //   ticks_per_sample * sample_rate as f64,
    //   error_per_second, error_per_sample);

    TicksDriftCorrection {
      ticks_per_sample,
      error_per_sample,
      error_accumulated: 0.0,
      last_correction: 0.0,
    }
  }

  pub fn get_ticks_per_sample(&self) -> f64 {
    self.ticks_per_sample
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

  pub fn next(&mut self, samples: u32) -> TicksTime {
    let samples_ticks = self.ticks_per_sample * samples as f64;
    let samples_error =
      samples_ticks - samples_ticks.round() + self.error_per_sample * samples as f64;
    let total_error = self.error_accumulated + samples_error;
    if total_error.abs() >= 1.0 {
      self.last_correction = total_error.round();
      self.error_accumulated = total_error - self.last_correction;
      TicksTime::new((samples_ticks + self.last_correction) as u64)
    } else {
      self.last_correction = 0.0;
      self.error_accumulated = total_error;
      TicksTime::new(samples_ticks as u64)
    }
  }
}

#[cfg(test)]
mod test {

  use super::TicksDriftCorrection;
  use super::{Signature, Tempo, TicksTime};

  #[test]
  pub fn ticks_drift_correction_new() {
    let correction = TicksDriftCorrection::new(Signature::new(6, 13), Tempo::new(130), 44100);
    assert_eq!(correction.ticks_per_sample, 4.190461073318216);
    assert_eq!(
      correction.error_per_sample,
      -0.000007558578987370399
    );
    assert_eq!(correction.error_accumulated, 0.0);
    assert_eq!(correction.last_correction, 0.0);
  }

  #[test]
  pub fn ticks_drift_correction_next() {
    let mut correction = TicksDriftCorrection::new(Signature::new(4, 4), Tempo::new(60), 44100);
    let ticks = correction.next(100);
    assert_eq!(ticks, TicksTime::new(628));
    let ticks = correction.next(100);
    assert_eq!(ticks, TicksTime::new(628));
    let ticks = correction.next(100);
    assert_eq!(ticks, TicksTime::new(627));
    let ticks = correction.next(100);
    assert_eq!(ticks, TicksTime::new(628));
    let ticks = correction.next(100);
    assert_eq!(ticks, TicksTime::new(627));
  }
}
