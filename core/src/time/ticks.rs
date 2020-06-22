use std::{
  cmp::{min, Ordering},
  ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use crate::time::{clock, ClockTime, Signature, Tempo};

pub const TICKS_RESOLUTION: u64 = 508_032_000; // 2^10 * 3^4 * 5^3 * 7^2

#[derive(Debug, Eq, Copy, Clone)]
pub struct TicksTime(u64);

impl TicksTime {
  pub fn new(ticks: u64) -> TicksTime {
    TicksTime(ticks)
  }

  pub fn zero() -> TicksTime {
    TicksTime(0)
  }

  pub fn per_minute(signature: Signature, tempo: Tempo) -> TicksTime {
    let ticks_per_beat = TICKS_RESOLUTION * 16 / u64::from(signature.get_note_value());
    TicksTime::new(ticks_per_beat * u64::from(tempo))
  }

  pub fn to_clock(&self, signature: Signature, tempo: Tempo) -> ClockTime {
    let ticks_per_minute = TicksTime::per_minute(signature, tempo).0;
    let clock_units =
      u128::from(self.0) * u128::from(clock::UNITS_PER_MINUTE) / u128::from(ticks_per_minute);
    ClockTime::new(clock_units as u64)
  }
}

impl Ord for TicksTime {
  fn cmp(&self, other: &TicksTime) -> Ordering {
    self.0.cmp(&other.0)
  }
}

impl PartialOrd for TicksTime {
  fn partial_cmp(&self, other: &TicksTime) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for TicksTime {
  fn eq(&self, other: &TicksTime) -> bool {
    self.0 == other.0
  }
}

impl Add for TicksTime {
  type Output = TicksTime;
  fn add(self, rhs: TicksTime) -> Self {
    TicksTime::new(self.0 + rhs.0)
  }
}

impl AddAssign for TicksTime {
  fn add_assign(&mut self, rhs: TicksTime) {
    *self = *self + rhs;
  }
}

impl Sub for TicksTime {
  type Output = TicksTime;
  fn sub(self, rhs: TicksTime) -> Self {
    TicksTime::new(self.0 - min(self.0, rhs.0))
  }
}

impl SubAssign for TicksTime {
  fn sub_assign(&mut self, rhs: TicksTime) {
    *self = *self - rhs;
  }
}

impl Mul for TicksTime {
  type Output = TicksTime;
  fn mul(self, rhs: TicksTime) -> Self {
    TicksTime::new(self.0 * rhs.0)
  }
}

impl Div for TicksTime {
  type Output = TicksTime;
  fn div(self, rhs: TicksTime) -> Self {
    TicksTime::new(self.0 / rhs.0)
  }
}

impl Div<u64> for TicksTime {
  type Output = TicksTime;
  fn div(self, rhs: u64) -> Self {
    TicksTime::new(self.0 / rhs)
  }
}

impl From<TicksTime> for f64 {
  fn from(item: TicksTime) -> Self {
    item.0 as f64
  }
}

impl From<TicksTime> for u64 {
  fn from(item: TicksTime) -> Self {
    item.0 as u64
  }
}

#[cfg(test)]
mod test {

  use super::{clock, Signature, Tempo, TicksTime};
  use std::cmp::Ordering;

  #[test]
  pub fn new() {
    let ticks_time = TicksTime::new(1234);
    assert_eq!(ticks_time.0, 1234);
  }

  #[test]
  pub fn zero() {
    let ticks_time = TicksTime::zero();
    assert_eq!(ticks_time.0, 0);
  }

  #[test]
  pub fn per_minute() {
    let signature = Signature::new(4, 4);
    let tempo = Tempo::new(120);
    let ticks = TicksTime::per_minute(signature, tempo);
    assert_eq!(ticks.0, 243_855_360_000);
  }

  #[test]
  pub fn to_clock() {
    let signature = Signature::new(4, 4);
    let tempo = Tempo::new(120);
    let ticks = TicksTime::per_minute(signature, tempo);
    let time = ticks.to_clock(signature, tempo);
    assert_eq!(time.units() / clock::UNITS_PER_MINUTE, 1);
  }

  #[test]
  pub fn ord_cmp() {
    let time1 = TicksTime::new(1234);
    let time2 = TicksTime::new(1235);
    assert_eq!(time1.cmp(&time2), Ordering::Less);
    assert_eq!(time2.cmp(&time1), Ordering::Greater);
    assert_eq!(time1.cmp(&time1), Ordering::Equal);
  }

  #[test]
  pub fn partial_ord_cmp() {
    let time1 = TicksTime::new(1234);
    let time2 = TicksTime::new(4321);
    assert_eq!(time1.partial_cmp(&time2), Some(Ordering::Less));
    assert_eq!(time2.partial_cmp(&time1), Some(Ordering::Greater));
    assert_eq!(time1.partial_cmp(&time1), Some(Ordering::Equal));
  }

  #[test]
  pub fn partial_eq() {
    let time1 = TicksTime::new(1234);
    let time2 = TicksTime::new(1234);
    let time3 = TicksTime::new(4321);
    assert!(time1.eq(&time2));
    assert!(!time2.eq(&time3));
  }

  #[test]
  pub fn add() {
    let time1 = TicksTime::new(100);
    let time2 = TicksTime::new(50);
    let result = time1 + time2;
    assert_eq!(result, TicksTime(150));
  }

  #[test]
  pub fn sub() {
    let time1 = TicksTime::new(100);
    let time2 = TicksTime::new(30);
    let result = time1 - time2;
    assert_eq!(result, TicksTime(70));
  }

  #[test]
  pub fn mul() {
    let time1 = TicksTime::new(100);
    let time2 = TicksTime::new(5);
    let result = time1 * time2;
    assert_eq!(result, TicksTime(500));
  }

  #[test]
  pub fn div() {
    let time1 = TicksTime::new(100);
    let time2 = TicksTime::new(5);
    let result = time1 / time2;
    assert_eq!(result, TicksTime(20));
  }

  #[test]
  pub fn div_u64() {
    let time1 = TicksTime::new(100);
    let result = time1 / 5;
    assert_eq!(result, TicksTime(20));
  }

  #[test]
  pub fn f64_from() {
    let time1 = TicksTime::new(1234);
    assert_eq!(f64::from(time1), 1234.0);
  }

  #[test]
  pub fn u64_from() {
    let time1 = TicksTime::new(1234);
    assert_eq!(u64::from(time1), 1234);
  }
}
