use std::{
  cmp::{min, Ordering},
  ops::{Add, Sub}
};

pub const TICKS_RESOLUTION: u64 = 240;

#[derive(Debug, Eq, Copy, Clone)]
pub struct TicksTime(u64);

impl TicksTime {
  pub fn new(ticks: u64) -> TicksTime {
    TicksTime(ticks)
  }

  pub fn get_ticks(&self) -> u64 {
    self.0
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

impl Sub for TicksTime {
  type Output = TicksTime;
  fn sub(self, rhs: TicksTime) -> Self {
    TicksTime::new(self.0 - min(self.0, rhs.0))
  }
}

#[cfg(test)]
mod test {

  use std::cmp::Ordering;
  use super::TicksTime;


  #[test]
  pub fn new() {
    let ticks_time = TicksTime::new(1234);
    assert_eq!(ticks_time.get_ticks(), 1234);
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
}
