use std::fmt;

use crate::time::{ticks::TICKS_RESOLUTION, Signature, TicksTime};

#[derive(PartialEq)]
pub struct BarsTime {
  bars: u16,
  beats: u16,
  sixteenths: u16,
  ticks: u32,
}

impl BarsTime {
  pub fn new(bars: u16, beats: u16, sixteenths: u16, ticks: u32) -> BarsTime {
    BarsTime {
      bars,
      beats,
      sixteenths,
      ticks,
    }
  }

  pub fn from_bars(bars: u16) -> BarsTime {
    BarsTime::new(bars, 0, 0, 0)
  }

  pub fn from_ticks(ticks_time: TicksTime, signature: Signature) -> BarsTime {
    let num_ticks = u64::from(ticks_time);
    let total_sixteenths = num_ticks / TICKS_RESOLUTION;
    let num_sixteenths_per_beat = 16 / u64::from(signature.get_note_value());
    let total_beats = total_sixteenths / num_sixteenths_per_beat;
    BarsTime {
      bars: (total_beats / u64::from(signature.get_num_beats())) as u16,
      beats: (total_beats % u64::from(signature.get_num_beats())) as u16,
      sixteenths: (total_sixteenths % num_sixteenths_per_beat) as u16,
      ticks: (num_ticks % TICKS_RESOLUTION) as u32,
    }
  }

  pub fn get_bars(&self) -> u16 {
    self.bars
  }

  pub fn get_beats(&self) -> u16 {
    self.beats
  }

  pub fn get_sixteenths(&self) -> u16 {
    self.sixteenths
  }

  pub fn get_ticks(&self) -> u32 {
    self.ticks
  }

  pub fn to_ticks(&self, signature: Signature) -> TicksTime {
    let num_sixteenths_per_beat = 16.0 / f64::from(signature.get_note_value());
    let num_ticks_per_beat = num_sixteenths_per_beat * TICKS_RESOLUTION as f64;
    let num_ticks_per_bar = f64::from(signature.get_num_beats()) * num_ticks_per_beat;
    TicksTime::new(
      u64::from(self.bars) * num_ticks_per_bar as u64
        + u64::from(self.beats) * num_ticks_per_beat as u64
        + u64::from(self.sixteenths) * TICKS_RESOLUTION
        + u64::from(self.ticks),
    )
  }
}

impl fmt::Debug for BarsTime {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{:05}:{:02}:{:02}:{:04}",
      self.bars + 1,
      self.beats + 1,
      self.sixteenths + 1,
      self.ticks
    )
  }
}

#[cfg(test)]
mod test {

  use super::BarsTime;
  use crate::time::{ticks::TicksTime, ticks::TICKS_RESOLUTION, Signature};

  #[test]
  pub fn new() {
    let time = BarsTime::new(10, 1, 2, 100);
    assert_eq!(time.get_bars(), 10);
    assert_eq!(time.get_beats(), 1);
    assert_eq!(time.get_sixteenths(), 2);
    assert_eq!(time.get_ticks(), 100);
  }

  #[test]
  pub fn from_bars() {
    let time = BarsTime::from_bars(2);
    assert_eq!(time.get_bars(), 2);
    assert_eq!(time.get_beats(), 0);
    assert_eq!(time.get_sixteenths(), 0);
    assert_eq!(time.get_ticks(), 0);
  }

  #[test]
  pub fn from_ticks() {
    let ticks = TicksTime::new(
      TICKS_RESOLUTION * 4 * 3 * 10 + // 10 bars
          TICKS_RESOLUTION * 4 * 2 +  // 2 beats
          TICKS_RESOLUTION     +      // 1 sixteens
          30, // 30 ticks
    );

    let time = BarsTime::from_ticks(ticks, Signature::new(3, 4));
    assert_eq!(time.get_bars(), 10);
    assert_eq!(time.get_beats(), 2);
    assert_eq!(time.get_sixteenths(), 1);
    assert_eq!(time.get_ticks(), 30);
  }

  #[test]
  pub fn to_ticks() {
    let signature = Signature::new(3, 4);
    let ticks = TicksTime::new(123_456_789);
    let time = BarsTime::from_ticks(ticks, signature);
    let ticks = time.to_ticks(signature);
    assert_eq!(u64::from(ticks), 123_456_789);
  }
}
