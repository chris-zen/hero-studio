use crate::time::{
  drift_correction::ClockDriftCorrection, drift_correction::TicksDriftCorrection, BarsTime,
  ClockTime, SampleRate, Signature, Tempo, TicksTime,
};

const DEFAULT_TEMPO: u16 = 120;
const DEFAULT_SIGNATURE_NUM_BEATS: u8 = 4;
const DEFAULT_SIGNATURE_NOTE_VALUE: u8 = 4;

pub struct Transport {
  sample_rate: SampleRate,
  signature: Signature,
  tempo: Tempo,

  playing: bool,

  next_play_duration: TicksTime,

  start_position: TicksTime,
  current_position: TicksTime,
  next_position: TicksTime,
  time_drift_correction: TicksDriftCorrection,

  loop_enabled: bool,
  loop_start: TicksTime,
  loop_end: TicksTime,
}

impl Transport {
  pub fn new(sample_rate: SampleRate) -> Self {
    let signature = Signature::new(DEFAULT_SIGNATURE_NUM_BEATS, DEFAULT_SIGNATURE_NOTE_VALUE);
    let tempo = Tempo::new(DEFAULT_TEMPO);
    let mut transport = Transport {
      sample_rate,
      signature,
      tempo,

      playing: false,

      next_play_duration: TicksTime::zero(),

      start_position: TicksTime::zero(),
      current_position: TicksTime::zero(),
      next_position: TicksTime::zero(),
      time_drift_correction: TicksDriftCorrection::new(signature, tempo, sample_rate),

      loop_enabled: true,
      loop_start: TicksTime::zero(),
      loop_end: TicksTime::zero(),
    };
    transport.update_timing_constants();
    transport
  }

  pub fn set_sample_rate(&mut self, sample_rate: SampleRate) {
    self.sample_rate = sample_rate;
    self.update_timing_constants();
  }

  pub fn get_sample_rate(&self) -> &SampleRate {
    &self.sample_rate
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
    self.next_play_duration = TicksTime::zero();
    self.current_position = self.start_position;
    self.next_position = self.current_position;
  }

  pub fn set_position(&mut self, position: BarsTime) {
    self.current_position = position.to_ticks(self.signature);
    self.next_position = self.current_position;
  }

  pub fn get_position(&self) -> BarsTime {
    BarsTime::from_ticks(self.current_position, self.signature)
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

  pub(super) fn segments_iterator(
    &self,
    master_clock: ClockTime,
    samples: u32,
  ) -> SegmentsIterator {
    SegmentsIterator::new(
      samples,
      self,
      master_clock,
      self.next_play_duration,
      self.next_position,
      &self.time_drift_correction,
    )
  }

  pub(super) fn update_from_segments(&mut self, segments: &SegmentsIterator) {
    self.next_play_duration = segments.next_play_duration;
    self.current_position = segments.next_position;
    self.next_position = segments.next_position;
    self.time_drift_correction = segments.time_drift_correction.clone();
    // println!("))))))))> {:#?}", self.time_drift_correction);
  }

  ///! Update timing constants that change sporadically (ex. changes on sample rate, tempo, signature, ...)
  fn update_timing_constants(&mut self) {
    self.time_drift_correction =
      TicksDriftCorrection::new(self.signature, self.tempo, self.sample_rate);
    println!(
      "Ticks error per sample = {:?} ticks",
      self.time_drift_correction.get_error_per_sample()
    );
  }

  ///! Determine whether or not not to move the song position to the start of the loop
  fn crossing_loop_end(&self, prev_ticks: TicksTime, next_position: TicksTime) -> bool {
    self.loop_enabled && prev_ticks < self.loop_end && self.loop_end <= next_position
  }
}

pub struct SegmentsIterator {
  master_clock: ClockTime,
  next_master_clock: ClockTime,

  play_duration: TicksTime,
  next_play_duration: TicksTime,

  current_position: TicksTime,
  next_position: TicksTime,

  remaining_duration: TicksTime,
  time_drift_correction: TicksDriftCorrection,
}

impl SegmentsIterator {
  fn new(
    samples: u32,
    _transport: &Transport,
    next_master_clock: ClockTime,
    next_play_duration: TicksTime,
    next_position: TicksTime,
    time_drift_correction: &TicksDriftCorrection,
  ) -> SegmentsIterator {
    let mut time_drift_correction = time_drift_correction.clone();

    // println!(
    //   "[{:?}] <{:010?}> Err: {:?} Correction {:?} ({:010?}) A {:010?}",
    //   BarsTime::from_ticks(next_position, _transport.signature),
    //   u64::from(next_position),
    //   time_drift_correction.get_error_accumulated(),
    //   time_drift_correction.get_last_correction(),
    //   u64::from(next_play_duration),
    //   next_master_clock.units()
    // );

    let remaining_duration = time_drift_correction.next(samples);

    SegmentsIterator {
      master_clock: next_master_clock,
      next_master_clock,
      play_duration: next_play_duration,
      next_play_duration,
      current_position: next_position,
      next_position,
      remaining_duration,
      time_drift_correction,
    }
  }

  pub fn next(&mut self, transport: &Transport) -> Option<Segment> {
    self.master_clock = self.next_master_clock;
    self.play_duration = self.next_play_duration;
    self.current_position = self.next_position;

    if self.remaining_duration > TicksTime::zero() {
      let end_position = self.current_position + self.remaining_duration;

      if transport.crossing_loop_end(self.current_position, end_position) {
        self.remaining_duration = end_position - transport.loop_end;
        let end_position = transport.loop_end;
        self.next_position = transport.loop_start;
        let segment_duration = end_position - self.current_position;
        self.next_play_duration = self.play_duration + segment_duration;
        let segment = Segment::new(
          transport.sample_rate,
          transport.signature,
          transport.tempo,
          self.master_clock,
          self.current_position,
          end_position,
          segment_duration,
          self.play_duration,
        );
        self.next_master_clock = self.master_clock + segment.clock_duration;
        Some(segment)
      } else {
        self.next_position = end_position;
        let segment_duration = self.remaining_duration;
        self.remaining_duration = TicksTime::zero();
        self.next_play_duration = self.play_duration + segment_duration;
        let segment = Segment::new(
          transport.sample_rate,
          transport.signature,
          transport.tempo,
          self.master_clock,
          self.current_position,
          end_position,
          segment_duration,
          self.play_duration,
        );
        self.next_master_clock = self.master_clock + segment.clock_duration;
        Some(segment)
      }
    } else {
      None
    }
  }
}

pub struct Segment {
  pub(super) sample_rate: SampleRate,
  pub(super) signature: Signature,
  pub(super) tempo: Tempo,

  pub(super) master_clock: ClockTime,

  pub(super) start_position: TicksTime,
  pub(super) end_position: TicksTime,
  pub(super) duration: TicksTime,

  pub(super) clock_start_position: ClockTime,
  pub(super) clock_end_position: ClockTime,
  pub(super) clock_duration: ClockTime,

  pub(super) play_duration: TicksTime,
  pub(super) clock_play_duration: ClockTime,
}

impl Segment {
  pub fn new(
    sample_rate: SampleRate,
    signature: Signature,
    tempo: Tempo,
    master_clock: ClockTime,
    start_position: TicksTime,
    end_position: TicksTime,
    duration: TicksTime,
    play_duration: TicksTime,
  ) -> Segment {
    Segment {
      sample_rate,
      signature,
      tempo,
      master_clock,
      start_position,
      end_position,
      duration,
      play_duration,
      clock_start_position: start_position.to_clock(signature, tempo),
      clock_end_position: end_position.to_clock(signature, tempo),
      clock_duration: duration.to_clock(signature, tempo),
      clock_play_duration: play_duration.to_clock(signature, tempo),
    }
  }
}
