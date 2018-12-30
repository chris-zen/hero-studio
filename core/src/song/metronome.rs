use crate::time::{SampleRate, Signature, TicksTime, BarsTime};

use super::transport::Segment;

pub struct Metronome {
  signature: Signature,
  enabled: bool,

  bar_ticks: TicksTime,
  beat_ticks: TicksTime,
}

impl Metronome {
  pub fn new(signature: Signature) -> Metronome {
    let (bar_ticks, beat_ticks) = Self::bar_and_beat_ticks(signature);

    Metronome {
      signature: signature,
      enabled: true,

      bar_ticks: bar_ticks,
      beat_ticks: beat_ticks,
    }
  }

  pub fn set_signature(&mut self, signature: Signature) {
    self.signature = signature;
    let (bar_ticks, beat_ticks) = Self::bar_and_beat_ticks(signature);
    self.bar_ticks = bar_ticks;
    self.beat_ticks = beat_ticks;
  }

  pub fn set_enabled(&mut self, enabled: bool) {
    self.enabled = enabled;
  }

  pub fn is_enabled(&self) -> bool {
    self.enabled
  }

  pub fn process_segment(&mut self, segment: &Segment) {
    if self.enabled {
      let mut next_bar = Self::ceil_ticks(segment.start_ticks, self.bar_ticks);
      let mut next_beat = Self::ceil_ticks(segment.start_ticks, self.beat_ticks);

      while next_beat < segment.end_ticks {
        let bars_time = BarsTime::from_ticks(next_beat, self.signature);

        if next_beat == next_bar {
          println!("Metronome: |> {:?}", bars_time);
          next_bar += self.bar_ticks;
        }
        else {
          println!("Metronome: ~> {:?}", bars_time);
        }
        next_beat += self.beat_ticks;
      }
    }
  }

  fn bar_and_beat_ticks(signature: Signature) -> (TicksTime, TicksTime) {
    let bar_ticks = BarsTime::from_bars(1).to_ticks(signature);
    let beat_ticks = bar_ticks / signature.get_num_beats() as u64;
    (bar_ticks, beat_ticks)
  }

  fn ceil_ticks(start: TicksTime, module: TicksTime) -> TicksTime {
    ((start + module - TicksTime::new(1)) / module) * module
  }
}
