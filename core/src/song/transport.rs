use crate::time::{
  Signature,
  Tempo,
  SampleRate,
  ClockTime,
  TicksTime,
  BarsTime,
  clock,
  drift_correction::ClockDriftCorrection,
  drift_correction::TicksDriftCorrection
};


const DEFAULT_TEMPO: u16 = 120;
const DEFAULT_SIGNATURE_NUM_BEATS: u8 = 4;
const DEFAULT_SIGNATURE_NOTE_VALUE: u8 = 4;

pub struct Transport {
  signature: Signature,
  tempo: Tempo,
  sample_rate: SampleRate,

  playing: bool,

  start_ticks: TicksTime,
  current_ticks: TicksTime,
  next_ticks: TicksTime,
  ticks_drift_correction: TicksDriftCorrection,

  start_time: ClockTime,
  current_time: ClockTime,
  next_time: ClockTime,
  clock_drift_correction: ClockDriftCorrection,

  loop_enabled: bool,
  loop_start: TicksTime,
  loop_end: TicksTime,
}

impl Transport {
  pub fn new(sample_rate: SampleRate) -> Self {
    let signature = Signature::new(DEFAULT_SIGNATURE_NUM_BEATS, DEFAULT_SIGNATURE_NOTE_VALUE);
    let tempo = Tempo::new(DEFAULT_TEMPO);
    let mut transport = Transport {
      signature: signature,
      tempo: tempo,
      sample_rate: sample_rate,

      playing: false,

      start_ticks: TicksTime::zero(),
      current_ticks: TicksTime::zero(),
      next_ticks: TicksTime::zero(),
      ticks_drift_correction: TicksDriftCorrection::new(signature, tempo, sample_rate),

      start_time: ClockTime::zero(),
      current_time: ClockTime::zero(),
      next_time: ClockTime::zero(),
      clock_drift_correction: ClockDriftCorrection::new(sample_rate),

      loop_enabled: true,
      loop_start: TicksTime::zero(),
      loop_end: TicksTime::zero(),
    };
    transport.update_timing_constants();
    transport
  }

  pub fn set_signature(&mut self, signature: Signature) {
    self.signature = signature;
  }

  pub fn get_signature(&self) -> &Signature {
    &self.signature
  }

  pub fn set_tempo(&mut self, tempo: Tempo) {
    self.tempo = tempo;
  }

  pub fn get_tempo(&self) -> &Tempo {
    &self.tempo
  }

  pub fn set_sample_rate(&mut self, sample_rate: SampleRate) {
    self.sample_rate = sample_rate;
    self.update_timing_constants();
  }

  pub fn get_sample_rate(&self) -> &SampleRate {
    &self.sample_rate
  }

  pub fn is_playing(&self) -> bool {
    self.playing
  }

  pub fn play(&mut self, restart: bool) -> bool {
    self.playing = !self.playing;
    if restart {
      self.reset_position();
    }
    self.playing
  }

  pub fn stop(&mut self) {
    if !self.playing {
      self.reset_position();
    }
    self.playing = false;
  }

  fn reset_position(&mut self) {
    self.current_ticks = self.start_ticks;
    self.next_ticks = self.current_ticks;

    self.current_time = self.start_time;
    self.next_time = self.current_time;
  }

  pub fn set_position(&mut self, position: BarsTime) {
    self.current_ticks = position.to_ticks(self.signature);
    self.next_ticks = self.current_ticks;

    self.current_time = self.current_ticks.to_clock(self.signature, self.tempo);
    self.next_time = self.current_time;
  }
  pub fn get_position(&self) -> BarsTime {
    BarsTime::from_ticks(self.current_ticks, self.signature)
  }

  pub fn set_loop_enabled(&mut self, enabled: bool) {
    self.loop_enabled = enabled;
  }

  pub fn is_loop_enabled(&self) -> bool {
    self.loop_enabled
  }

  pub fn set_loop_start(&mut self, position: BarsTime) {
    self.loop_start = position.to_ticks(self.signature);
  }

  pub fn get_loop_start(&self) -> BarsTime {
    BarsTime::from_ticks(self.loop_start, self.signature)
  }

  pub fn set_loop_end(&mut self, position: BarsTime) {
    self.loop_end = position.to_ticks(self.signature);
  }

  pub fn get_loop_end(&self) -> BarsTime {
    BarsTime::from_ticks(self.loop_end, self.signature)
  }

  pub(super) fn segments_iterator(&mut self, samples: u32) -> SegmentsIterator {
    SegmentsIterator::new(samples, self,
                          self.next_ticks, &self.ticks_drift_correction,
                          self.next_time, &self.clock_drift_correction)
  }

  pub(super) fn update_from_segments(&mut self, segments: &SegmentsIterator) {
    self.current_ticks = segments.next_ticks;
    self.next_ticks = segments.next_ticks;
    self.ticks_drift_correction = segments.ticks_drift_correction.clone();

    self.current_time = segments.next_time;
    self.next_time = segments.next_time;
    self.clock_drift_correction = segments.clock_drift_correction.clone();
  }

  ///! Update timing constants that change sporadically (ex. changes on sample rate, tempo, signature, ...)
  fn update_timing_constants(&mut self) {
    self.ticks_drift_correction = TicksDriftCorrection::new(self.signature, self.tempo, self.sample_rate);
    println!("Ticks error per sample = {:?} ticks", self.ticks_drift_correction.get_error_per_sample());

    self.clock_drift_correction = ClockDriftCorrection::new(self.sample_rate);
    println!("Clock error per sample = {:?} clock units", self.clock_drift_correction.get_error_per_sample());
  }

  ///! Determine whether or not not to move the song position to the start of the loop
  fn loop_to_start(&self, prev_ticks: TicksTime, next_ticks: TicksTime) -> bool {
    self.loop_enabled &&
      prev_ticks < self.loop_end &&
      self.loop_end <= next_ticks
  }
}

pub struct Segment {
  pub(super) start_time: ClockTime,
  pub(super) end_time: ClockTime,

  pub(super) start_ticks: TicksTime,
  pub(super) end_ticks: TicksTime,
}

pub struct SegmentsIterator {
  current_time: ClockTime,
  next_time: ClockTime,
  remaining_time: ClockTime,
  clock_drift_correction: ClockDriftCorrection,

  current_ticks: TicksTime,
  next_ticks: TicksTime,
  remaining_ticks: TicksTime,
  ticks_drift_correction: TicksDriftCorrection
}

impl SegmentsIterator {

  fn new(samples: u32, transport: &Transport,
         next_ticks: TicksTime, ticks_drift_correction: &TicksDriftCorrection,
         next_time: ClockTime, clock_drift_correction: &ClockDriftCorrection) -> SegmentsIterator {

    let mut ticks_drift_correction = ticks_drift_correction.clone();
    let mut clock_drift_correction = clock_drift_correction.clone();

    println!("[{:?}] <{:010?}> Err: {:?} Correction {:?} <{:010?}> Err: {:?} Correction {:?}",
      BarsTime::from_ticks(next_ticks, transport.signature),
      u64::from(next_ticks),
      ticks_drift_correction.get_error_accumulated(),
      ticks_drift_correction.get_last_correction(),
      next_time.units(),
      clock_drift_correction.get_error_accumulated(),
      clock_drift_correction.get_last_correction());

    let remaining_ticks = ticks_drift_correction.next(samples);
    let remaining_time = clock_drift_correction.next(samples);

    SegmentsIterator {
      current_ticks: next_ticks,
      next_ticks: next_ticks,
      remaining_ticks: remaining_ticks,
      ticks_drift_correction: ticks_drift_correction,

      current_time: next_time,
      next_time: next_time,
      remaining_time: remaining_time,
      clock_drift_correction: clock_drift_correction,
    }
  }

  pub fn next(&mut self, transport: &Transport) -> Option<Segment> {
    self.current_ticks = self.next_ticks;
    self.current_time = self.next_time;

    if self.remaining_ticks > TicksTime::zero() {
      let end_ticks = self.current_ticks + self.remaining_ticks;
      let end_time = self.current_time + self.remaining_time;

      if transport.loop_to_start(self.current_ticks, end_ticks) {
        let loop_end_time = transport.loop_end.to_clock(transport.signature, transport.tempo);
        let segment = Segment {
          start_ticks: self.current_ticks,
          end_ticks: transport.loop_end,
          start_time: self.current_time,
          end_time: loop_end_time,
        };
        self.next_ticks = transport.loop_start;
        self.remaining_ticks = end_ticks - transport.loop_end;
        self.next_time = transport.loop_start.to_clock(transport.signature, transport.tempo);
        self.remaining_time = end_time - loop_end_time;
        Some(segment)
      }
      else {
        let segment = Segment {
          start_ticks: self.current_ticks,
          end_ticks: end_ticks,
          start_time: self.current_time,
          end_time: end_time
        };
        self.next_ticks = end_ticks;
        self.remaining_ticks = TicksTime::zero();
        self.next_time = end_time;
        self.remaining_time = ClockTime::zero();
        Some(segment)
      }
    }
    else {
      None
    }
  }
}
