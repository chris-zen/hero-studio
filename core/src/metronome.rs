use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{Metronome as MetronomeConfig, MetronomeNote, MidiPort};
use crate::midi;
use crate::midi::buffer::Endpoint;
use crate::time::{
  ticks::TICKS_RESOLUTION, BarsTime, ClockTime, SampleRate, Signature, Tempo, TicksTime,
};
use crate::transport::{Segment, Transport};

pub struct Metronome {
  config: MetronomeConfig,
  enabled: bool,
  endpoint: Endpoint,
  bar_duration: TicksTime,
  beat_duration: TicksTime,
}

impl Metronome {
  pub fn new(config: MetronomeConfig, signature: Signature) -> Metronome {
    let enabled = config.enabled;
    let endpoint = Self::endpoint_from_midi_port(&config.port);
    let (bar_duration, beat_duration) = Self::bar_and_beat_duration(signature);

    Metronome {
      config,
      enabled,
      endpoint,
      bar_duration,
      beat_duration,
    }
  }

  pub fn set_enabled(&mut self, enabled: bool) {
    self.enabled = enabled;
  }

  pub fn is_enabled(&self) -> bool {
    self.enabled
  }

  pub fn endpoint(&self) -> Endpoint {
    self.endpoint
  }

  pub fn process_segment(&mut self, segment: &Segment, buffer: &mut midi::Buffer) {
    if self.enabled {
      let signature = segment.signature;
      let tempo = segment.tempo;
      let mut next_bar_position = Self::ceil_ticks(segment.start_position, self.bar_duration);
      let mut next_beat_position = Self::ceil_ticks(segment.start_position, self.beat_duration);

      while next_beat_position < segment.end_position {
        let advanced_ticks = next_beat_position - segment.start_position;
        let note_time = segment.master_clock + advanced_ticks.to_clock(signature, tempo);

        // let bars_time = BarsTime::from_ticks(next_beat_position, signature);
        if next_beat_position == next_bar_position {
          // println!("Metronome: |> {:?}", bars_time);
          let note = &self.config.bar_note;
          Self::push_note(buffer, note_time, note, signature, tempo);
          next_bar_position += self.bar_duration;
        } else {
          // println!("Metronome: ~> {:?}", bars_time);
          let note = &self.config.beat_note;
          Self::push_note(buffer, note_time, note, signature, tempo);
        }
        next_beat_position += self.beat_duration;
      }
    }
  }

  fn push_note(
    buffer: &mut midi::Buffer,
    start_time: ClockTime,
    note: &MetronomeNote,
    signature: Signature,
    tempo: Tempo,
  ) {
    // TODO duration_ticks only needs to be calculated once per note
    let duration_ticks = TicksTime::new(16 * TICKS_RESOLUTION / note.duration as u64);
    let duration_time = duration_ticks.to_clock(signature, tempo);
    let end_time = start_time + duration_time;

    buffer.push(
      start_time,
      midi::Message::NoteOn {
        channel: note.channel,
        key: note.key,
        velocity: note.velocity,
      },
    );
    buffer.push(
      end_time,
      midi::Message::NoteOff {
        channel: note.channel,
        key: note.key,
        velocity: note.velocity,
      },
    );
  }

  fn bar_and_beat_duration(signature: Signature) -> (TicksTime, TicksTime) {
    let bar_duration = BarsTime::from_bars(1).to_ticks(signature);
    let beat_duration = bar_duration / signature.get_num_beats() as u64;
    (bar_duration, beat_duration)
  }

  fn ceil_ticks(start: TicksTime, module: TicksTime) -> TicksTime {
    ((start + module - TicksTime::new(1)) / module) * module
  }

  fn endpoint_from_midi_port(port: &MidiPort) -> Endpoint {
    // TODO Select the endpoint from the configuration when update events are received
    match port {
      MidiPort::None => Endpoint::None,
      MidiPort::SystemDefault => Endpoint::Default,
      MidiPort::All => Endpoint::All,
      MidiPort::ByName(_name) => Endpoint::None, // TODO
    }
  }
}
