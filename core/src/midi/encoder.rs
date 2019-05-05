use super::messages::Message;
use super::types::{U14, U3, U4, U7};

#[inline]
fn u3(d: &U3) -> u8 {
  d & 0x07
}

#[inline]
fn u4(d: &U4) -> u8 {
  d & 0x0f
}

#[inline]
fn u7(d: &U7) -> u8 {
  d & 0x7f
}

#[inline]
fn u14_msb(d: &U14) -> u8 {
  ((d >> 7) & 0x7fu16) as u8
}

#[inline]
fn u14_lsb(d: &U14) -> u8 {
  (d & 0x7f) as u8
}

#[inline]
fn status_and_channel(status: U4, channel: &U4) -> u8 {
  (status << 4) | (channel & 0x0f)
}

pub struct Encoder;

impl Encoder {
  pub fn data_size(msg: &Message) -> usize {
    match msg {
      Message::NoteOff { .. } => 3,
      Message::NoteOn { .. } => 3,
      Message::PolyphonicKeyPressure { .. } => 3,
      Message::ControlChange { .. } => 3,
      Message::ProgramChange { .. } => 2,
      Message::ChannelPressure { .. } => 2,
      Message::PitchBend { .. } => 3,
      Message::AllSoundOff { .. } => 3,
      Message::ResetAllControllers { .. } => 3,
      Message::LocalControlOff { .. } => 3,
      Message::LocalControlOn { .. } => 3,
      Message::AllNotesOff { .. } => 3,
      Message::OmniModeOff { .. } => 3,
      Message::OmniModeOn { .. } => 3,
      Message::MonoModeOn { .. } => 3,
      Message::PolyModeOn { .. } => 3,
      Message::MTCQuarterFrame { .. } => 2,
      Message::SongPositionPointer { .. } => 3,
      Message::SongSelect { .. } => 2,
      Message::TuneRequest => 1,
      Message::TimingClock => 1,
      Message::Start => 1,
      Message::Continue => 1,
      Message::Stop => 1,
      Message::ActiveSensing => 1,
      Message::SystemReset => 1,
    }
  }

  pub fn encode(msg: &Message, out: &mut [u8]) {
    match msg {
      Message::NoteOff {
        channel,
        key,
        velocity,
      } => out[..3].copy_from_slice(&[status_and_channel(0b1000, channel), u7(key), u7(velocity)]),
      Message::NoteOn {
        channel,
        key,
        velocity,
      } => out[..3].copy_from_slice(&[status_and_channel(0b1001, channel), u7(key), u7(velocity)]),
      Message::PolyphonicKeyPressure {
        channel,
        key,
        value,
      } => out[..3].copy_from_slice(&[status_and_channel(0b1010, channel), u7(key), u7(value)]),
      Message::ControlChange {
        channel,
        controller,
        value,
      } => out[..3].copy_from_slice(&[
        status_and_channel(0b1011, channel),
        u7(controller),
        u7(value),
      ]),
      Message::ProgramChange { channel, value } => {
        out[..2].copy_from_slice(&[status_and_channel(0b1100, channel), u7(value)])
      }
      Message::ChannelPressure { channel, value } => {
        out[..2].copy_from_slice(&[status_and_channel(0b1101, channel), u7(value)])
      }
      Message::PitchBend { channel, value } => out[..3].copy_from_slice(&[
        status_and_channel(0b1110, channel),
        u14_lsb(value),
        u14_msb(value),
      ]),
      Message::AllSoundOff { channel } => {
        out[..3].copy_from_slice(&[status_and_channel(0b1011, channel), 120, 0])
      }
      Message::ResetAllControllers { channel } => {
        out[..3].copy_from_slice(&[status_and_channel(0b1011, channel), 121, 0])
      }
      Message::LocalControlOff { channel } => {
        out[..3].copy_from_slice(&[status_and_channel(0b1011, channel), 122, 0])
      }
      Message::LocalControlOn { channel } => {
        out[..3].copy_from_slice(&[status_and_channel(0b1011, channel), 122, 127])
      }
      Message::AllNotesOff { channel } => {
        out[..3].copy_from_slice(&[status_and_channel(0b1011, channel), 123, 0])
      }
      Message::OmniModeOff { channel } => {
        out[..3].copy_from_slice(&[status_and_channel(0b1011, channel), 124, 0])
      }
      Message::OmniModeOn { channel } => {
        out[..3].copy_from_slice(&[status_and_channel(0b1011, channel), 125, 0])
      }
      Message::MonoModeOn {
        channel,
        num_channels,
      } => out[..3].copy_from_slice(&[status_and_channel(0b1011, channel), 126, u7(num_channels)]),
      Message::PolyModeOn { channel } => {
        out[..3].copy_from_slice(&[status_and_channel(0b1011, channel), 127, 0])
      }
      Message::MTCQuarterFrame { msg_type, value } => {
        out[..2].copy_from_slice(&[0b1111_0001, (u3(msg_type) << 4) | u4(value)])
      }
      Message::SongPositionPointer { beats } => {
        out[..3].copy_from_slice(&[0b1111_0010, u14_lsb(beats), u14_msb(beats)])
      }
      Message::SongSelect { song } => out[..2].copy_from_slice(&[0b1111_0011, u7(song)]),
      Message::TuneRequest => out[0] = 0b1111_0110,
      Message::TimingClock => out[0] = 0b1111_1000,
      Message::Start => out[0] = 0b1111_1010,
      Message::Continue => out[0] = 0b1111_1011,
      Message::Stop => out[0] = 0b1111_1100,
      Message::ActiveSensing => out[0] = 0b1111_1110,
      Message::SystemReset => out[0] = 0b1111_1111,
    }
  }

  pub fn sysex_data_size(data: &[U7]) -> usize {
    data.len() + 2
  }

  pub fn sysex_encode(data: &[U7], out: &mut [u8]) {
    out[0] = 0b1111_0000;
    out[1..=data.len()].copy_from_slice(&data);
    out[data.len() + 1] = 0b1111_0111
  }
}

#[cfg(test)]
mod test {

  use super::*;

  #[test]
  pub fn test_u3() {
    assert_eq!(u3(&0xff), 0x07);
  }

  #[test]
  pub fn test_u4() {
    assert_eq!(u4(&0xff), 0x0f);
  }

  #[test]
  pub fn test_u7() {
    assert_eq!(u7(&0xff), 0x7f);
  }

  #[test]
  pub fn test_u14_lsb() {
    assert_eq!(u14_lsb(&0b10_1010_1010_1010), 0b010_1010);
  }

  #[test]
  pub fn test_u14_msb() {
    assert_eq!(u14_msb(&0b10_1010_1010_1010), 0b101_0101);
  }

  #[test]
  pub fn test_status_and_channel() {
    assert_eq!(status_and_channel(0b1010_1010, &0b0101_0101), 0b1010_0101);
  }

  #[test]
  pub fn note_off() {
    assert_encoding(
      &Message::NoteOff {
        channel: 1,
        key: 65,
        velocity: 120,
      },
      vec![0b1000_0001, 65, 120],
    )
  }

  #[test]
  pub fn note_on() {
    assert_encoding(
      &Message::NoteOn {
        channel: 1,
        key: 65,
        velocity: 120,
      },
      vec![0b1001_0001, 65, 120],
    )
  }

  #[test]
  pub fn polyphonic_key_pressure() {
    assert_encoding(
      &Message::PolyphonicKeyPressure {
        channel: 1,
        key: 65,
        value: 120,
      },
      vec![0b1010_0001, 65, 120],
    )
  }

  #[test]
  pub fn control_change() {
    assert_encoding(
      &Message::ControlChange {
        channel: 1,
        controller: 65,
        value: 120,
      },
      vec![0b1011_0001, 65, 120],
    )
  }

  #[test]
  pub fn program_change() {
    assert_encoding(
      &Message::ProgramChange {
        channel: 1,
        value: 120,
      },
      vec![0b1100_0001, 120],
    )
  }

  #[test]
  pub fn channel_pressure() {
    assert_encoding(
      &Message::ChannelPressure {
        channel: 1,
        value: 120,
      },
      vec![0b1101_0001, 120],
    )
  }

  #[test]
  pub fn pitch_bend() {
    assert_encoding(
      &Message::PitchBend {
        channel: 1,
        value: 0b10_1010_1010_1010,
      },
      vec![0b1110_0001, 0b010_1010, 0b101_0101],
    )
  }

  #[test]
  pub fn all_sound_off() {
    assert_encoding(
      &Message::AllSoundOff { channel: 1 },
      vec![0b1011_0001, 120, 0],
    )
  }

  #[test]
  pub fn reset_all_controllers() {
    assert_encoding(
      &Message::ResetAllControllers { channel: 1 },
      vec![0b1011_0001, 121, 0],
    )
  }

  #[test]
  pub fn local_control_off() {
    assert_encoding(
      &Message::LocalControlOff { channel: 1 },
      vec![0b1011_0001, 122, 0],
    )
  }

  #[test]
  pub fn local_control_on() {
    assert_encoding(
      &Message::LocalControlOn { channel: 1 },
      vec![0b1011_0001, 122, 127],
    )
  }

  #[test]
  pub fn all_notes_off() {
    assert_encoding(
      &Message::AllNotesOff { channel: 1 },
      vec![0b1011_0001, 123, 0],
    )
  }

  #[test]
  pub fn omni_mode_off() {
    assert_encoding(
      &Message::OmniModeOff { channel: 1 },
      vec![0b1011_0001, 124, 0],
    )
  }

  #[test]
  pub fn mono_mode_on() {
    assert_encoding(
      &Message::MonoModeOn {
        channel: 1,
        num_channels: 5,
      },
      vec![0b1011_0001, 126, 5],
    )
  }

  #[test]
  pub fn poly_mode_on() {
    assert_encoding(
      &Message::PolyModeOn { channel: 1 },
      vec![0b1011_0001, 127, 0],
    )
  }

  #[test]
  pub fn sysex() {
    assert_sysex_encoding(
      vec![1, 2, 3, 4, 5],
      vec![0b1111_0000, 1, 2, 3, 4, 5, 0b1111_0111],
    )
  }

  #[test]
  pub fn mtc_quarter_frame() {
    assert_encoding(
      &Message::MTCQuarterFrame {
        msg_type: 2,
        value: 5,
      },
      vec![0b1111_0001, 0b010_0101],
    )
  }

  #[test]
  pub fn song_position_pointer() {
    assert_encoding(
      &Message::SongPositionPointer {
        beats: 0b10_1010_1010_1010,
      },
      vec![0b1111_0010, 0b010_1010, 0b101_0101],
    )
  }

  #[test]
  pub fn song_select() {
    assert_encoding(&Message::SongSelect { song: 54 }, vec![0b1111_0011, 54])
  }

  #[test]
  pub fn tune_request() {
    assert_encoding(&Message::TuneRequest, vec![0b1111_0110])
  }

  #[test]
  pub fn timing_clock() {
    assert_encoding(&Message::TimingClock, vec![0b1111_1000])
  }

  #[test]
  pub fn start() {
    assert_encoding(&Message::Start, vec![0b1111_1010])
  }

  #[test]
  pub fn test_continue() {
    assert_encoding(&Message::Continue, vec![0b1111_1011])
  }

  #[test]
  pub fn stop() {
    assert_encoding(&Message::Stop, vec![0b1111_1100])
  }

  #[test]
  pub fn active_sensing() {
    assert_encoding(&Message::ActiveSensing, vec![0b1111_1110])
  }

  #[test]
  pub fn system_reset() {
    assert_encoding(&Message::SystemReset, vec![0b1111_1111])
  }

  fn assert_encoding(msg: &Message, expected: Vec<u8>) {
    let data_len = Encoder::data_size(msg);
    let mut data = Vec::<u8>::with_capacity(data_len);
    unsafe { data.set_len(data_len) };
    Encoder::encode(msg, data.as_mut_slice());
    assert_eq!(data, expected);
  }

  fn assert_sysex_encoding(msg: Vec<U7>, expected: Vec<u8>) {
    let data_len = Encoder::sysex_data_size(&msg);
    let mut data = Vec::<u8>::with_capacity(data_len);
    unsafe { data.set_len(data_len) };
    Encoder::sysex_encode(&msg, data.as_mut_slice());
    assert_eq!(data, expected);
  }
}
