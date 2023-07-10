use std::collections::HashMap;
use tui::style::{Color, Modifier};

pub fn extract_attribute(data: HashMap<String, String>, attribute_name: &str) -> String {
  let default_value = "".to_string();
  let value = data.get(attribute_name).unwrap_or(&default_value);
  String::from(value)
}

pub fn modifier_from_str(input: &str) -> Modifier {
  let input = input.to_lowercase();
  let input = input.as_str();
  match input {
      "dim" => Modifier::DIM,
      "bold" => Modifier::BOLD,
      "italic" => Modifier::ITALIC,
      "underlined" => Modifier::UNDERLINED,
      "crossed_out" => Modifier::CROSSED_OUT,
      "rapid_blink" => Modifier::RAPID_BLINK,
      "slow_blink" => Modifier::SLOW_BLINK,
      "reversed" => Modifier::REVERSED,
      "hidden" => Modifier::HIDDEN,
      _ => Modifier::DIM
  }
}

pub fn color_from_str(input: &str) -> Color {
  let input = input.to_lowercase();
  let input = input.as_str();
  match input {
      "reset" => Color::Reset,
      "black" => Color::Black,
      "red" => Color::Red,
      "green" => Color::Green,
      "yellow" => Color::Yellow,
      "blue" => Color::Blue,
      "magenta" => Color::Magenta,
      "cyan" => Color::Cyan,
      "gray" => Color::Gray,
      "darkGray" => Color::DarkGray,
      "lightRed" => Color::LightRed,
      "lightGreen" => Color::LightGreen,
      "lightYellow" => Color::LightYellow,
      "lightBlue" => Color::LightBlue,
      "lightMagenta" => Color::LightMagenta,
      "lightCyan" => Color::LightCyan,
      "white" => Color::White,
      _ => Color::Reset,
  }
}

