use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{Metronome as MetronomeConfig, MetronomeNote, MidiPort};
use crate::midi::bus::{BusAddress, NodeClass, NodeFeature};
use crate::midi::{
  bus::{BusNode, BusQuery, MidiBusLock},
  messages::Message,
};
use crate::song::transport::Transport;
use crate::time::{
  ticks::TICKS_RESOLUTION, BarsTime, ClockTime, SampleRate, Signature, Tempo, TicksTime,
};

use super::transport::Segment;

pub struct Metronome {
  config: MetronomeConfig,

  enabled: bool,

  bar_ticks: TicksTime,
  beat_ticks: TicksTime,

  midi_bus: MidiBusLock,
  bus_addresses: Vec<BusAddress>,
}

impl Metronome {
  pub fn new(config: MetronomeConfig, transport: &Transport, midi_bus: MidiBusLock) -> Metronome {
    let enabled = config.enabled;

    let (bar_ticks, beat_ticks) = Self::bar_and_beat_ticks(*transport.get_signature());

    let bus_addresses = Self::bus_addresses_from_midi_port(&config.port, &midi_bus);

    Metronome {
      config,
      enabled,
      bar_ticks,
      beat_ticks,
      midi_bus,
      bus_addresses,
    }
  }

  pub fn set_enabled(&mut self, enabled: bool) {
    self.enabled = enabled;
  }

  pub fn is_enabled(&self) -> bool {
    self.enabled
  }

  pub fn process_segment(&mut self, segment: &Segment, transport: &Transport) {
    if self.enabled {
      let signature = *transport.get_signature();
      let tempo = *transport.get_tempo();

      let mut next_bar = Self::ceil_ticks(segment.start_ticks, self.bar_ticks);
      let mut next_beat = Self::ceil_ticks(segment.start_ticks, self.beat_ticks);

      while next_beat < segment.end_ticks {
        let bars_time = BarsTime::from_ticks(next_beat, signature);

        let advanced_ticks = next_beat - segment.start_ticks;
        let note_time = segment.play_time + advanced_ticks.to_clock(signature, tempo);

        if next_beat == next_bar {
          // println!("Metronome: |> {:?}", bars_time);
          self.send_note(note_time, &self.config.bar_note, signature, tempo);
          next_bar += self.bar_ticks;
        } else {
          // println!("Metronome: ~> {:?}", bars_time);
          self.send_note(note_time, &self.config.beat_note, signature, tempo);
        }
        next_beat += self.beat_ticks;
      }
    }
  }

  fn send_note(
    &self,
    start_time: ClockTime,
    note: &MetronomeNote,
    signature: Signature,
    tempo: Tempo,
  ) {
    let duration_ticks = TicksTime::new(16 * TICKS_RESOLUTION / note.duration as u64);
    let duration_time = duration_ticks.to_clock(signature, tempo);
    let end_time = start_time + duration_time;

    if let Ok(mut midi_bus) = self.midi_bus.write() {
      for bus_addr in self.bus_addresses.iter() {
        if let Some(bus_node_lock) = midi_bus.get_node_mut(bus_addr) {
          if let Ok(mut bus_node) = bus_node_lock.write() {
            bus_node.send_message(
              start_time,
              &Message::NoteOn {
                channel: note.channel,
                key: note.key,
                velocity: note.velocity,
              },
            );
            bus_node.send_message(
              end_time,
              &Message::NoteOff {
                channel: note.channel,
                key: note.key,
                velocity: note.velocity,
              },
            );
          }
        }
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

  fn bus_addresses_from_midi_port(port: &MidiPort, midi_bus: &MidiBusLock) -> Vec<BusAddress> {
    match port {
      MidiPort::None => Vec::new(),
      MidiPort::SystemDefault => Self::bus_addresses_by_query(
        midi_bus,
        &BusQuery::new()
          .class(NodeClass::Destination)
          .feature(NodeFeature::Default),
      ),
      MidiPort::All => {
        Self::bus_addresses_by_query(midi_bus, &BusQuery::new().class(NodeClass::Destination))
      }
      MidiPort::ByName(name) => {
        Self::bus_addresses_by_query(midi_bus, &BusQuery::new().name(name.as_str()))
      }
    }
  }

  fn bus_addresses_by_query(midi_bus: &MidiBusLock, query: &BusQuery) -> Vec<BusAddress> {
    midi_bus
      .read()
      .map(|bus| bus.addresses_by_query(query))
      .unwrap_or(Vec::new())
  }
}
