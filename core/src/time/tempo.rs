pub struct Tempo(u16);

impl Tempo {
  pub fn new(value: u16) -> Tempo {
    Tempo(value)
  }

  pub fn get_value(&self) -> u16 {
    self.0
  }
}

#[cfg(test)]
mod test {

  use super::Tempo;

  #[test]
  pub fn tempo_new() {
    let tempo = Tempo::new(120);
    assert_eq!(tempo.get_value(), 120);
  }
}
