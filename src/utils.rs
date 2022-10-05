use std::collections::HashMap;

pub fn extract_attribute(data: HashMap<String, String>, attribute_name: &str) -> String {
  let default_value = "".to_string();
  let value = data.get(attribute_name).unwrap_or(&default_value);
  String::from(value)
}

