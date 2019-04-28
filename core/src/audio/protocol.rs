use crate::pool::Pool;
use crate::audio;
use crate::midi;
use crate::time::ClockTime;

pub struct Protocol {
  midi_output: Option<Box<midi::IoVec>>,
  audio_output: Option<Box<audio::Buffer>>,
  audio_input: Option<Box<audio::Buffer>>,
  input_time: Option<ClockTime>,
  output_time: Option<ClockTime>,
}

impl Protocol {
  pub fn new() -> Protocol {
    Protocol {
      midi_output: None,
      audio_output: None,
      audio_input: None,
      input_time: None,
      output_time: None,
    }
  }

  // This function prepares a protocol object with processed audio
  // and MIDI events for the next frame.
  // It will also provide buffers for the returned audio input.
  pub fn prepare(
    &mut self,
    midi_output: Box<midi::IoVec>,
    audio_output: Box<audio::Buffer>,
    audio_input: Box<audio::Buffer>) {
    self.midi_output = Some(midi_output);
    self.audio_output = Some(audio_output);
    self.audio_input = Some(audio_input);
    self.input_time = None;
    self.output_time = None;
  }

  // Once a prepared protocol instance has been processed by the audio thread,
  // it can be returned to the workers to continue sending the MIDI events,
  // and storing the audio input, which will require knowing the times.
  pub fn update_times(&mut self, input_seconds: f64, output_seconds: f64) {
    self.input_time = Some(ClockTime::from_seconds(input_seconds));
    self.output_time = Some(ClockTime::from_seconds(output_seconds));
  }

  // Given the corresponding pools, this function releases
  // all the internal objects.
  pub fn release(&mut self,
                 midi_io_vec_pool: &mut Pool<midi::IoVec>,
                 audio_buffer_pool: &mut Pool<audio::Buffer>) {
    self.midi_output.take().into_iter()
      .for_each(|midi_output| midi_io_vec_pool.release(midi_output));
    self.audio_output.take().into_iter()
      .for_each(|audio_output| audio_buffer_pool.release(audio_output));
    self.audio_input.take().into_iter()
      .for_each(|audio_input| audio_buffer_pool.release(audio_input));
  }

  // Returns the MIDI output vector
  // It is assumed that it will be defined when called
  pub fn midi_output(&mut self) -> &mut midi::IoVec {
    &mut *self.midi_output.as_mut().unwrap()
  }

  // Returns the audio output buffer
  // It is assumed that it will be defined when called
  pub fn audio_output(&self) -> &audio::Buffer {
    &*self.audio_output.as_ref().unwrap()
  }

  // Returns the audio input buffer
  // It is assumed that it will be defined when called
  pub fn audio_input(&mut self) -> &mut audio::Buffer {
    &mut *self.audio_input.as_mut().unwrap()
  }

  // Returns the host time for the audio input buffer
  // It is assumed that it will be defined when called
  pub fn input_time(&self) -> ClockTime {
    self.input_time.unwrap()
  }

  // Returns the host time for the audio output buffer
  // It is assumed that it will be defined when called
  pub fn output_time(&self) -> ClockTime {
    self.output_time.unwrap()
  }
}

pub fn new_protocol_pool(pool_capacity: usize) -> Pool<Protocol> {
  let allocator = Box::new(|| Box::new(Protocol::new()));
  let reset = Box::new(|_item: &mut Protocol| {});
  Pool::new(pool_capacity, allocator, reset)
}