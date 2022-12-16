use std::collections::HashMap;

use tui::{
    layout::{Alignment, Constraint, Direction},
    style::{Color, Style},
    widgets::Borders,
};

use crate::markup_element::MarkupElement;

const WIDGET_NAMES: &'static [&'static str] = &["block", "p"];

pub fn extract_attribute(data: HashMap<String, String>, attribute_name: &str) -> String {
    let default_value = "".to_string();
    let value = data.get(attribute_name).unwrap_or(&default_value);
    String::from(value)
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

pub fn is_widget(node_name: &str) -> bool {
    WIDGET_NAMES.contains(&node_name)
}

pub fn is_layout(node_name: &str) -> bool {
    node_name.eq("layout")
}

pub fn get_border(border: String) -> Borders {
    if border.contains("|") {
        let borders = border
            .split("|")
            .map(|s| String::from(s))
            .map(|s| get_border(s))
            .collect::<Vec<Borders>>();
        let size = borders.len();
        let mut res = borders[0];
        for i in 1..size {
            res |= borders[i];
        }
        return res;
    }
    let border = match border.to_lowercase().as_str() {
        "all" => Borders::ALL,
        "bottom" => Borders::BOTTOM,
        "top" => Borders::TOP,
        "left" => Borders::LEFT,
        "right" => Borders::RIGHT,
        _ => Borders::NONE,
    };
    border
}

pub fn get_constraint(constraint: String) -> Constraint {
    let res = if constraint.ends_with("%") {
        let constraint_value = constraint.replace("%", "");
        let constraint_value = constraint_value.parse::<u16>().unwrap_or(1);
        Constraint::Percentage(constraint_value)
    } else if constraint.ends_with("min") {
        let constraint_value = constraint.replace("min", "");
        let constraint_value = constraint_value.parse::<u16>().unwrap_or(1);
        Constraint::Min(constraint_value)
    } else if constraint.ends_with("max") {
        let constraint_value = constraint.replace("max", "");
        let constraint_value = constraint_value.parse::<u16>().unwrap_or(1);
        Constraint::Max(constraint_value)
    } else if constraint.contains(":") {
        let parts = constraint.split(":");
        let parts: Vec<&str> = parts.collect();
        let x = String::from(parts[0]).parse::<u32>().unwrap_or(1);
        let y = String::from(parts[1]).parse::<u32>().unwrap_or(1);
        Constraint::Ratio(x, y)
    } else {
        let constraint_value = constraint.parse::<u16>().unwrap_or(1);
        Constraint::Length(constraint_value)
    };
    res
}

pub fn get_direction(node: &MarkupElement) -> Direction {
    let direction = extract_attribute(node.attributes.clone(), "direction");
    if direction.eq("horizontal") {
        Direction::Horizontal
    } else {
        Direction::Vertical
    }
}

pub fn get_alignment(node: &MarkupElement) -> Alignment {
    let align_text = extract_attribute(node.attributes.clone(), "align");
    match align_text.as_str() {
        "center" => Alignment::Center,
        "left" => Alignment::Left,
        "right" => Alignment::Right,
        _ => Alignment::Left,
    }
}

pub fn get_styles(node: &MarkupElement) -> Style {
    let mut res = Style::default();
    let styles_text = extract_attribute(node.attributes.clone(), "styles");
    if styles_text.len() < 3 {
        return res;
    }
    let styles_vec = styles_text
        .split(";")
        .map(|style| style.split(":").map(|word| word.trim()).collect())
        .map(|data: Vec<&str>| (data[0], data[1]))
        .collect::<Vec<(&str, &str)>>();
    let styles: HashMap<&str, &str> = styles_vec.into_iter().collect();
    if styles.contains_key("bg") {
        let color = color_from_str(styles.get("bg").unwrap());
        res = res.bg(color);
    }
    if styles.contains_key("fg") {
        let color = color_from_str(styles.get("fg").unwrap());
        res = res.fg(color);
    }
    res
}
