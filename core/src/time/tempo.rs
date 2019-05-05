#[derive(Debug, Clone, Copy)]
pub struct Tempo(u16);

impl Tempo {
  pub fn new(value: u16) -> Tempo {
    Tempo(value)
  }

  pub fn get_value(&self) -> u16 {
    self.0
  }
}

impl From<Tempo> for f64 {
  fn from(item: Tempo) -> Self {
    f64::from(item.0)
  }
}

impl From<Tempo> for u64 {
  fn from(item: Tempo) -> Self {
    u64::from(item.0)
  }
}

impl From<Tempo> for u16 {
  fn from(item: Tempo) -> Self {
    item.0
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
