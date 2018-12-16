pub struct Color(String);

impl Color {
  pub fn new(value: String) -> Color {
    Color(value)
  }

  pub fn from_rgb(r: u8, g: u8, b: u8) -> Color {
    Color(format!("rgb({},{},{})", r, g, b))
  }

  pub fn get_value(&self) -> &str {
    self.0.as_str()
  }
}

#[cfg(test)]
mod test {

  use super::Color;

  #[test]
  pub fn new() {
    let color = Color::new("red".into());
    assert_eq!(color.get_value(), "red");
  }

  #[test]
  pub fn from_rgb() {
    let color = Color::from_rgb(10, 20, 30);
    assert_eq!(color.get_value(), "rgb(10,20,30)");
  }


}