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

  clock_drift_correction: ClockDriftCorrection,

  playing: bool,
  start_time: ClockTime,
  current_time: ClockTime,
  next_time: ClockTime,

  loop_enabled: bool,
  loop_start_time: ClockTime,
  loop_end_time: ClockTime,
}

impl Transport {
  pub fn new(sample_rate: SampleRate) -> Self {
    let mut transport = Transport {
      signature: Signature::new(DEFAULT_SIGNATURE_NUM_BEATS, DEFAULT_SIGNATURE_NOTE_VALUE),
      tempo: Tempo::new(DEFAULT_TEMPO),
      sample_rate: sample_rate,

      clock_drift_correction: ClockDriftCorrection::new(sample_rate),

      playing: false,

      start_time: ClockTime::zero(),
      current_time: ClockTime::zero(),
      next_time: ClockTime::zero(),

      loop_enabled: true,
      loop_start_time: ClockTime::zero(),
      loop_end_time: ClockTime::zero(),
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
      self.current_time = self.start_time;
      self.next_time = self.current_time;
    }
    self.playing
  }

  pub fn stop(&mut self) {
    self.playing = false;
  }

  pub fn set_loop_enabled(&mut self, enabled: bool) {
    self.loop_enabled = enabled;
  }

  pub fn is_loop_enabled(&self) -> bool {
    self.loop_enabled
  }

  pub fn set_loop_start_time(&mut self, time: ClockTime) {
    self.loop_start_time = time;
  }

  pub fn get_loop_start_time(&self) -> ClockTime {
    self.loop_start_time
  }

  pub fn set_loop_end_time(&mut self, time: ClockTime) {
    self.loop_end_time = time;
  }

  pub fn get_loop_end_time(&self) -> ClockTime {
    self.loop_end_time
  }

  pub(super) fn segments_iterator(&mut self, samples: u32) -> SegmentsIterator {
    SegmentsIterator::new(self.next_time, samples, &self.clock_drift_correction)
  }

  pub(super) fn update_from_segments(&mut self, segments: &SegmentsIterator) {
    self.current_time = segments.next_time;
    self.next_time = segments.next_time;
    self.clock_drift_correction = segments.clock_drift_correction.clone();
  }

  ///! Update timing constants that change sporadically (ex. changes on sample rate, tempo, signature, ...)
  fn update_timing_constants(&mut self) {
    self.clock_drift_correction = ClockDriftCorrection::new(self.sample_rate);
    println!("Clock error per sample = {:?} clock units", self.clock_drift_correction.get_error_per_sample());
  }

  ///! Determine whether or not not to move the song position to the start of the loop
  fn loop_to_start(&self, prev_time: ClockTime, next_time: ClockTime) -> bool {
    self.loop_enabled &&
      prev_time < self.loop_end_time &&
      self.loop_end_time <= next_time
  }
}

pub struct Segment {
  pub(super) start_time: ClockTime,
  pub(super) end_time: ClockTime,
}

pub struct SegmentsIterator {
  current_time: ClockTime,
  next_time: ClockTime,
  remaining_time: ClockTime,
  clock_drift_correction: ClockDriftCorrection
}

impl SegmentsIterator {

  fn new(next_time: ClockTime, samples: u32, clock_drift_correction: &ClockDriftCorrection) -> SegmentsIterator {
    let mut clock_drift_correction = clock_drift_correction.clone();
    let remaining_time = clock_drift_correction.next(samples);

    println!("[{:013?}] Err: {:?} Correction {:?}",
        next_time.units(),
        clock_drift_correction.get_error_accumulated(),
        clock_drift_correction.get_last_correction());

    SegmentsIterator {
      current_time: next_time,
      next_time: next_time,
      remaining_time: remaining_time,
      clock_drift_correction: clock_drift_correction,
    }
  }

  pub fn next(&mut self, transport: &Transport) -> Option<Segment> {
    self.current_time = self.next_time;
    if self.remaining_time > ClockTime::zero() {
      let end_time = self.current_time + self.remaining_time;
      if transport.loop_to_start(self.current_time, end_time) {
        let segment = Segment {
          start_time: self.current_time,
          end_time: transport.loop_end_time
        };
        self.next_time = transport.loop_start_time;
        self.remaining_time = end_time - transport.loop_end_time;
        Some(segment)
      }
      else {
        let segment = Segment {
          start_time: self.current_time,
          end_time: end_time
        };
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
