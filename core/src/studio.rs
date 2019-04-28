use std::fmt;
use std::ops::{Deref, DerefMut};

use crate::config::{Config, MidiPort};
use crate::pool::Pool;
use crate::midi;
use crate::midi::bus::{BusAddress, BusQuery, MidiBusLock, NodeClass, NodeFeature};
use crate::audio;
use crate::metronome::Metronome;
use crate::transport::{Segment, Transport};
use crate::song::Song;
use crate::time::{ClockTime, BarsTime};

pub const MAX_AUDIO_BUFFER_SIZE: usize = 2 * 4 * 1024;

fn fill_with_zero(s: &mut [f32]) {
  for d in s {
    *d = 0.0;
  }
}

pub struct Studio {
  config: Config,
  midi_bus: MidiBusLock,
  transport: Transport,
  metronome: Metronome,
  metronome_bus_address: Option<BusAddress>,
  song: Song,
}

unsafe impl Send for Studio {}

impl Studio {
  pub fn new(config: Config, midi_bus: MidiBusLock) -> Studio {

    let song = Song::new("untitled", &config);

    let sample_rate = config.audio.sample_rate;
    let transport = Transport::new(sample_rate);

    let metronome_config = config.metronome.clone();
    let signature = *transport.get_signature();
    let metronome_bus_address =
      Self::bus_address_from_midi_port(&config.metronome.port, &midi_bus);
    let metronome = Metronome::new(metronome_config, signature);

    Studio {
      config,
      midi_bus,
      transport,
      metronome,
      metronome_bus_address,
      song,
    }
  }

  pub fn config(&self) -> &Config {
    &self.config
  }

  pub fn song(&self) -> &Song {
    &self.song
  }

  pub fn song_mut(&mut self) -> &mut Song {
    &mut self.song
  }

  pub fn set_loop_enabled(&mut self, enabled: bool) {
    self.transport.set_loop_enabled(enabled);
  }

  pub fn set_loop_start(&mut self, position: BarsTime) {
    self.transport.set_loop_start(position);
  }

  pub fn set_loop_end(&mut self, position: BarsTime) {
    self.transport.set_loop_end(position)
  }

  pub fn play(&mut self, restart: bool) -> bool {
    self.transport.play(restart);
    self.transport.is_playing()
  }

  pub fn stop(&mut self) {
    self.transport.stop();
  }

  pub fn prepare(
    &mut self,
    frames: usize,
    midi_buffer_pool: &mut Pool<midi::Buffer>,
    midi_output: &mut midi::IoVec,
    _audio_input: &audio::Buffer,
    _audio_output: &mut audio::Buffer,
  ) {

    if self.transport.is_playing() {
      let mut metronome_buffer = midi_buffer_pool.get_or_alloc();

      let base_time = ClockTime::zero();
      let mut segments = self.transport.segments_iterator(base_time, frames as u32);
      while let Some(segment) = segments.next(&self.transport) {
        self.metronome.process_segment(&segment, &mut metronome_buffer);
        self.song.process_segment(&segment);
      }
      self.transport.update_from_segments(&segments);

      if let Some(address) = self.metronome_bus_address {
        midi_output.push(midi::Io {
          address: address,
          buffer: Some(metronome_buffer),
        });
      }

      let out = _audio_output.slice_mut(frames * 2);
      fill_with_zero(out);
      // for i in 0..frames {
      //   let v = (i / 2) as f32 / frames as f32;
      //   let j = i * 2;
      //   _audio_output[j] = v;
      //   _audio_output[j + 1] = v;
      // }
    }
    else {
      let out = _audio_output.slice_mut(frames * 2);
      fill_with_zero(out);
    }
  }

  // FIXME Have a dedicated MIDI thread
  pub fn complete(
    &mut self,
    midi_buffer_pool: &mut Pool<midi::Buffer>,
    midi_output: &mut midi::IoVec,
    output_time: ClockTime,
  ) {
    if let Ok(mut midi_bus) = self.midi_bus.write() {
      for output in midi_output.iter_mut() {
        if let Some(buffer) = output.buffer.take() {
          if let Some(bus_node_lock) = midi_bus.get_node_mut(&output.address) {
            if let Ok(mut bus_node) = bus_node_lock.write() {
              for event in buffer.iter() {
                let timestamp = output_time + event.timestamp;
                bus_node.send_message(
                  timestamp,
                  &event.message,
                );
              }
            }
          }
          midi_buffer_pool.release(buffer);
        }
      }
    }
  }

  // pub fn audio_handler(
  //   &mut self,
  //   audio_time: AudioTime,
  //   frames: usize,
  //   _in_buffer: &[f32],
  //   _out_buffer: &mut [f32],
  // ) {
  //   // retrieve midi events from the armed track's input port
  //   // retrieve midi events from the tracks from time.output
  //   // schedule midi events to the output ports
  //   // process the audio for time.output taking into consideration the midi events

  //   if self.transport.is_playing() {
  //     let mut metronome_buffer = self.midi_buffer_pool.get_or_alloc();

  //     let master_clock = audio_time.output;
  //     let mut segments = self.transport.segments_iterator(master_clock, frames as u32);
  //     while let Some(segment) = segments.next(&self.transport) {

  //       self.metronome.process_segment(&segment, &mut metronome_buffer);
  //       self.song.process_segment(&segment);
  //     }
  //     self.transport.update_from_segments(&segments);


  //   }

  //   // TODO some devices might need to keep track of time even when not playing
  // }

  fn bus_address_from_midi_port(port: &MidiPort, midi_bus: &MidiBusLock) -> Option<BusAddress> {
    match port {
      MidiPort::None => None,
      MidiPort::SystemDefault => Self::bus_address_by_query(
        midi_bus,
        &BusQuery::new()
          .class(NodeClass::Destination)
          .feature(NodeFeature::Default),
      ),
      MidiPort::All => {
        None // FIXME We need an specific address for it, or not supported
      }
      MidiPort::ByName(name) => {
        Self::bus_address_by_query(
          midi_bus,
          &BusQuery::new()
            .name(name.as_str()))
      }
    }
  }

  fn bus_address_by_query(midi_bus: &MidiBusLock, query: &BusQuery) -> Option<BusAddress> {
    midi_bus
      .read()
      .map(|bus| {
        bus
          .addresses_by_query(query)
          .first()
          .map(|address| address.clone())
      })
      .unwrap_or(None)
  }
}

impl fmt::Debug for Studio {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Studio({:?})", self.song.get_name())
  }
}
