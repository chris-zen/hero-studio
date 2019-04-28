use crate::midi::messages::Message;
use crate::midi::types::{U14, U4, U7};

pub struct Decoder<'a> {
  pos: usize,
  start: usize,
  sysex_data: Vec<U7>,
  sysex_decoding: bool,
  data: &'a [u8],
}

impl<'a> Decoder<'a> {
  pub fn new(data: &'a [u8]) -> Decoder<'a> {
    Decoder {
      pos: 0,
      start: 0,
      sysex_data: Vec::new(),
      sysex_decoding: false,
      data: data,
    }
  }

  fn unknown(&self, end: usize) -> Message {
    Message::Unknown(self.data[self.start..end].to_vec())
  }

  fn next_data(&mut self) -> Result<U7, usize> {
    if self.pos < self.data.len() {
      let d1 = self.data[self.pos];
      if d1 & 0b10000000 == 0 {
        self.pos += 1;
        Ok(d1)
      } else {
        Err(self.pos)
      }
    } else {
      Err(self.pos)
    }
  }

  fn next_data2(&mut self) -> Result<(U7, U7), usize> {
    self
      .next_data()
      .and_then(|d1| self.next_data().and_then(|d2| Ok((d1, d2))))
  }

  fn decode_note(&mut self, channel: U4, is_on: bool) -> Message {
    match self.next_data2() {
      Ok((key, velocity)) => match is_on {
        true => Message::NoteOn {
          channel: channel,
          key: key,
          velocity: velocity,
        },
        false => Message::NoteOff {
          channel: channel,
          key: key,
          velocity: velocity,
        },
      },
      Err(end) => self.unknown(end),
    }
  }

  fn decode_polyphonic_key_pressure(&mut self, channel: U4) -> Message {
    match self.next_data2() {
      Ok((key, pressure)) => Message::PolyphonicKeyPressure {
        channel: channel,
        key: key,
        value: pressure,
      },
      Err(end) => self.unknown(end),
    }
  }

  fn decode_control_change(&mut self, channel: U4) -> Message {
    match self.next_data2() {
      Ok((controller, value)) => match controller {
        120 => match value {
          0 => Message::AllSoundOff { channel: channel },
          _ => self.unknown(self.pos),
        },
        121 => Message::ResetAllControllers { channel: channel },
        122 => match value {
          0 => Message::LocalControlOff { channel: channel },
          127 => Message::LocalControlOn { channel: channel },
          _ => self.unknown(self.pos),
        },
        123 => match value {
          0 => Message::AllNotesOff { channel: channel },
          _ => self.unknown(self.pos),
        },
        124 => match value {
          0 => Message::OmniModeOff { channel: channel },
          _ => self.unknown(self.pos),
        },
        125 => match value {
          0 => Message::OmniModeOn { channel: channel },
          _ => self.unknown(self.pos),
        },
        126 => Message::MonoModeOn {
          channel: channel,
          num_channels: value,
        },
        127 => match value {
          0 => Message::PolyModeOn { channel: channel },
          _ => self.unknown(self.pos),
        },
        _ => Message::ControlChange {
          channel: channel,
          controller: controller,
          value: value,
        },
      },
      Err(end) => self.unknown(end),
    }
  }

  fn decode_program_change(&mut self, channel: U4) -> Message {
    match self.next_data() {
      Ok(program) => Message::ProgramChange {
        channel: channel,
        value: program,
      },
      Err(end) => self.unknown(end),
    }
  }

  fn decode_channel_pressure(&mut self, channel: U4) -> Message {
    match self.next_data() {
      Ok(pressure) => Message::ChannelPressure {
        channel: channel,
        value: pressure,
      },
      Err(end) => self.unknown(end),
    }
  }

  fn decode_pitch_bend(&mut self, channel: U4) -> Message {
    match self.next_data2() {
      Ok((lsb, msb)) => Message::PitchBend {
        channel: channel,
        value: ((msb as U14) << 7) | (lsb as U14),
      },
      Err(end) => self.unknown(end),
    }
  }

  fn decode_mtc_quarter_frame(&mut self) -> Message {
    match self.next_data() {
      Ok(data) => Message::MTCQuarterFrame {
        msg_type: (data >> 4) & 0x07,
        value: data & 0x0f,
      },
      Err(end) => self.unknown(end),
    }
  }

  fn decode_song_position_pointer(&mut self) -> Message {
    match self.next_data2() {
      Ok((lsb, msb)) => Message::SongPositionPointer {
        beats: ((msb as U14) << 7) | (lsb as U14),
      },
      Err(end) => self.unknown(end),
    }
  }

  fn decode_song_select(&mut self) -> Message {
    match self.next_data() {
      Ok(song) => Message::SongSelect { song: song },
      Err(end) => self.unknown(end),
    }
  }

  fn decode_sysex_start(&mut self) -> Option<Message> {
    if !self.sysex_decoding {
      self.sysex_decoding = true;
      self.decode_sysex_data()
    } else {
      Some(self.unknown(self.pos))
    }
  }

  fn decode_sysex_end(&mut self) -> Option<Message> {
    if self.sysex_decoding {
      self.sysex_decoding = false;
      let data = self.sysex_data.to_owned();
      self.sysex_data = Vec::new();
      Some(Message::SysEx { data: data })
    } else {
      Some(self.unknown(self.pos))
    }
  }

  fn decode_sysex_data(&mut self) -> Option<Message> {
    let start = self.pos;
    let mut pos = start;
    while pos < self.data.len() && (self.data[pos] & 0x80) == 0 {
      pos += 1
    }
    self.pos = pos;
    let data = &self.data[start..pos];
    self.sysex_data.extend(data);
    if pos < self.data.len() {
      let status = self.data[pos];
      self.start = self.pos;
      self.pos += 1;
      self.decode(status)
    } else {
      self.sysex_decoding = false;
      let mut data = self.sysex_data.to_owned();
      self.sysex_data = Vec::new();
      data.insert(0, 0b11110000);
      Some(Message::Unknown(data))
    }
  }

  fn decode(&mut self, status: U7) -> Option<Message> {
    match (status >> 4) & 0x0f {
      0b1000 => Some(self.decode_note(status & 0x0f, false)),
      0b1001 => Some(self.decode_note(status & 0x0f, true)),
      0b1010 => Some(self.decode_polyphonic_key_pressure(status & 0x0f)),
      0b1011 => Some(self.decode_control_change(status & 0x0f)),
      0b1100 => Some(self.decode_program_change(status & 0x0f)),
      0b1101 => Some(self.decode_channel_pressure(status & 0x0f)),
      0b1110 => Some(self.decode_pitch_bend(status & 0x0f)),
      0b1111 => match status & 0x0f {
        0b0000 => self.decode_sysex_start(),
        0b0001 => Some(self.decode_mtc_quarter_frame()),
        0b0010 => Some(self.decode_song_position_pointer()),
        0b0011 => Some(self.decode_song_select()),
        0b0100 => Some(self.unknown(self.pos)),
        0b0101 => Some(self.unknown(self.pos)),
        0b0110 => Some(Message::TuneRequest),
        0b0111 => self.decode_sysex_end(),
        0b1000 => Some(Message::TimingClock),
        0b1001 => Some(self.unknown(self.pos)),
        0b1010 => Some(Message::Start),
        0b1011 => Some(Message::Continue),
        0b1100 => Some(Message::Stop),
        0b1101 => Some(self.unknown(self.pos)),
        0b1110 => Some(Message::ActiveSensing),
        0b1111 => Some(Message::SystemReset),
        _ => None, // It should never reach this path but the compiler complains otherwise
      },
      _ => Some(Message::Unknown(vec![status])),
    }
  }
}

impl<'a> Iterator for Decoder<'a> {
  type Item = Message;

  fn next(&mut self) -> Option<Message> {
    if self.sysex_decoding {
      self.decode_sysex_data()
    } else {
      if self.pos < self.data.len() {
        let status = self.data[self.pos];
        self.start = self.pos;
        self.pos += 1;
        self.decode(status)
      } else {
        None
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::midi::messages::Message;

  #[test]
  fn decode_empty_vec() {
    let data = &Vec::new();
    let mut dec = Decoder::new(data);
    assert_eq!(dec.next(), None);
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn next_data_unknown() {
    let data = &vec![0b1000_0000, 64, 0b1000_0001, 0b1000_0010, 12];
    let mut dec = Decoder::new(data);
    assert_eq!(dec.next(), Some(Message::Unknown(vec![0b1000_0000, 64])));
    assert_eq!(dec.next(), Some(Message::Unknown(vec![0b1000_0001])));
    assert_eq!(dec.next(), Some(Message::Unknown(vec![0b1000_0010, 12])));
  }

  #[test]
  fn decode_notes() {
    let data = &vec![0b1000_0101u8, 64, 127, 0b1001_1010, 0, 127];
    let mut dec = Decoder::new(data);
    assert_eq!(
      dec.next(),
      Some(Message::NoteOff {
        channel: 0b0101,
        key: 64,
        velocity: 127
      })
    );
    assert_eq!(
      dec.next(),
      Some(Message::NoteOn {
        channel: 0b1010,
        key: 0,
        velocity: 127
      })
    );
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_polyphonic_key_pressure() {
    let data = &vec![0b1010_0101u8, 64, 127];
    let mut dec = Decoder::new(data);
    assert_eq!(
      dec.next(),
      Some(Message::PolyphonicKeyPressure {
        channel: 0b0101,
        key: 64,
        value: 127
      })
    );
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_control_change() {
    let data = &vec![0b1011_0101u8, 64, 127];
    let mut dec = Decoder::new(data);
    assert_eq!(
      dec.next(),
      Some(Message::ControlChange {
        channel: 0b0101,
        controller: 64,
        value: 127
      })
    );
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_program_change() {
    let data = &vec![0b1100_0101u8, 0b0_1010101];
    let mut dec = Decoder::new(data);
    assert_eq!(
      dec.next(),
      Some(Message::ProgramChange {
        channel: 0b0101,
        value: 0b0_1010101
      })
    );
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_channel_pressure() {
    let data = &vec![0b1101_0101u8, 0b0_1010101];
    let mut dec = Decoder::new(data);
    assert_eq!(
      dec.next(),
      Some(Message::ChannelPressure {
        channel: 0b0101,
        value: 0b0_1010101
      })
    );
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_pitch_bend() {
    let data = &vec![0b1110_0101u8, 0b0_1010101, 0b0_0101010];
    let mut dec = Decoder::new(data);
    assert_eq!(
      dec.next(),
      Some(Message::PitchBend {
        channel: 0b0101,
        value: 0b0_01010101010101
      })
    );
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_mtc_quarter_frame() {
    let data = &vec![0b1111_0001u8, 0b0_101_1010];
    let mut dec = Decoder::new(data);
    assert_eq!(
      dec.next(),
      Some(Message::MTCQuarterFrame {
        msg_type: 0b101,
        value: 0b1010
      })
    );
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_song_position_pointer() {
    let data = &vec![
      0b1111_0010u8,
      0b0_1010101,
      0b0_0101010,
      0b1111_0010u8,
      0b0_0101010,
      0b0_1010101,
    ];
    let mut dec = Decoder::new(data);
    assert_eq!(
      dec.next(),
      Some(Message::SongPositionPointer {
        beats: 0b01010101010101
      })
    );
    assert_eq!(
      dec.next(),
      Some(Message::SongPositionPointer {
        beats: 0b10101010101010
      })
    );
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_song_select() {
    let data = &vec![0b1111_0011u8, 0b0_1010101];
    let mut dec = Decoder::new(data);
    assert_eq!(dec.next(), Some(Message::SongSelect { song: 0b1010101 }));
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_tune_request() {
    let data = &vec![0b1111_0110u8];
    let mut dec = Decoder::new(data);
    assert_eq!(dec.next(), Some(Message::TuneRequest));
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_timing_clock() {
    let data = &vec![0b1111_1000u8];
    let mut dec = Decoder::new(data);
    assert_eq!(dec.next(), Some(Message::TimingClock));
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_start_continue_stop() {
    let data = &vec![0b1111_1010u8, 0b1111_1011, 0b1111_1100];
    let mut dec = Decoder::new(data);
    assert_eq!(dec.next(), Some(Message::Start));
    assert_eq!(dec.next(), Some(Message::Continue));
    assert_eq!(dec.next(), Some(Message::Stop));
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_active_sensing() {
    let data = &vec![0b1111_1110u8];
    let mut dec = Decoder::new(data);
    assert_eq!(dec.next(), Some(Message::ActiveSensing));
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_system_reset() {
    let data = &vec![0b1111_1111u8];
    let mut dec = Decoder::new(data);
    assert_eq!(dec.next(), Some(Message::SystemReset));
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_reserved() {
    let data = &vec![0b1111_0100u8, 0b1111_0101, 0b1111_1001, 0b1111_1101];
    let mut dec = Decoder::new(data);
    assert_eq!(dec.next(), Some(Message::Unknown(vec![0b1111_0100])));
    assert_eq!(dec.next(), Some(Message::Unknown(vec![0b1111_0101])));
    assert_eq!(dec.next(), Some(Message::Unknown(vec![0b1111_1001])));
    assert_eq!(dec.next(), Some(Message::Unknown(vec![0b1111_1101])));
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_sysex_continuous() {
    let data = &vec![0b1111_0000u8, 1, 2, 3, 4, 0b1111_0111];
    let mut dec = Decoder::new(data);
    assert_eq!(
      dec.next(),
      Some(Message::SysEx {
        data: vec![1u8, 2, 3, 4]
      })
    );
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_sysex_interleaved() {
    let data = &vec![
      0b1111_0000u8,
      1,
      2,
      0b1000_0101u8,
      64,
      127,
      3,
      4,
      0b1111_0111,
    ];
    let mut dec = Decoder::new(data);
    assert_eq!(
      dec.next(),
      Some(Message::NoteOff {
        channel: 0b0101,
        key: 64,
        velocity: 127
      })
    );
    assert_eq!(
      dec.next(),
      Some(Message::SysEx {
        data: vec![1u8, 2, 3, 4]
      })
    );
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_sysex_unexpected_end_data() {
    let data = &vec![0b1111_0000u8, 1, 2];
    let mut dec = Decoder::new(data);
    assert_eq!(
      dec.next(),
      Some(Message::Unknown(vec![0b1111_0000u8, 1, 2]))
    );
    assert_eq!(dec.next(), None);
  }

  #[test]
  fn decode_sysex_unexpected_end_interleaved() {
    let data = &vec![0b1111_0000u8, 1, 2, 0b1000_0000u8, 64];
    let mut dec = Decoder::new(data);
    assert_eq!(dec.next(), Some(Message::Unknown(vec![0b1000_0000u8, 64])));
    assert_eq!(
      dec.next(),
      Some(Message::Unknown(vec![0b1111_0000u8, 1, 2]))
    );
    assert_eq!(dec.next(), None);
  }
}
