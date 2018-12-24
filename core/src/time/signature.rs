#[derive(Debug, Clone, Copy)]
pub struct Signature {
  num_beats: u8,  // numerator
  note_value: u8, // denominator
}

impl Signature {
  pub fn new(num_beats: u8, note_value: u8) -> Signature {
    assert!(note_value <= 16);
    Signature {
      num_beats,
      note_value,
    }
  }

  pub fn get_num_beats(&self) -> u8 {
    self.num_beats
  }

  pub fn get_note_value(&self) -> u8 {
    self.note_value
  }
}

#[cfg(test)]
mod test {

  use super::Signature;

  #[test]
  pub fn signature_new() {
    let signature = Signature::new(3, 4);
    assert_eq!(signature.get_num_beats(), 3);
    assert_eq!(signature.get_note_value(), 4);
  }
}
