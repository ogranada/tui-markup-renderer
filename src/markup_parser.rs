use crate::markup_element::MarkupElement;
use crate::storage::{IRendererStorage, RendererStorage};
use crate::utils::{color_from_str, extract_attribute};
#[allow(unused_imports)]
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::rc::Rc;
use std::vec::Vec;
use std::{borrow::BorrowMut, cell::RefCell};
use tui::layout::{Alignment, Rect};
use tui::style::Style;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use xml::reader::{EventReader, XmlEvent};

const WIDGET_NAMES: &'static [&'static str] = &["block", "p"];

/**
 * To use specific features you can use the macro:
 *   - #[cfg(feature = "test")]
 * also you can negate something:
 *   - #[cfg(not(test))]
 * To enable something only for test use:
 *   - #[cfg(test)]
 * To allow make a struct printable for debug use:
 *   - #[derive(Debug)]
 */

#[derive(Debug)]
pub struct MarkupParser<B: Backend> {
    pub path: String,
    pub failed: bool,
    pub error: Option<String>,
    pub root: Option<Rc<RefCell<MarkupElement>>>,
    pub storage: Option<Rc<RefCell<RendererStorage<B>>>>,
}

impl<B: Backend> MarkupParser<B> {
    // Constructor

    pub fn new(path: String, optional_storage: Option<RendererStorage<B>>) -> MarkupParser<B> {
        if !Path::new(&path).exists() {
            panic!("Markup file does not exist at {}", &path);
        }
        let file = File::open(&path).unwrap();
        let buffer = BufReader::new(file);
        let parser = EventReader::new(buffer);
        let storage = optional_storage.unwrap_or(RendererStorage::new());
        let mut root_node: Option<Rc<RefCell<MarkupElement>>> = None;
        let mut current_node: Option<Rc<RefCell<MarkupElement>>> = None;
        let mut parent_node: Option<Rc<RefCell<MarkupElement>>> = None;
        for e in parser {
            match e {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    let mut attrs = HashMap::new();
                    for attr in attributes {
                        attrs.insert(attr.name.local_name, attr.value);
                    }
                    let _id = attrs.get("id").unwrap_or(&String::from("unknown"));
                    let partial = MarkupElement {
                        deep: if parent_node.is_some() {
                            MarkupParser::<B>::get_element(parent_node.clone()).deep + 1
                        } else {
                            0
                        },
                        text: String::from("PENDING FROM XML"),
                        name: name.local_name,
                        attributes: attrs,
                        children: vec![],
                        parent_node: parent_node.clone(),
                    };

                    current_node = Some(Rc::new(RefCell::new(partial)));

                    let is_root_defined = root_node.clone().as_ref().is_some();
                    if !is_root_defined {
                        root_node = current_node.clone();
                    }

                    if parent_node.is_some() {
                        let parent = parent_node.clone();
                        let parent = parent.unwrap();
                        let parent = parent.as_ref();
                        let mut parent = parent.borrow_mut();
                        let son = current_node.clone().unwrap();
                        parent.children.push(son);
                    }
                    parent_node = current_node.clone();
                }
                Ok(XmlEvent::Characters(ref r)) => {
                    let node = current_node.clone();
                    let node = node.unwrap();
                    let node = node.as_ref();
                    let mut node = node.borrow_mut();
                    node.text = String::from(r.trim());
                }
                Ok(XmlEvent::EndElement { .. }) => {
                    let p = MarkupParser::<B>::get_element(parent_node.clone());
                    parent_node = p.parent_node;
                }
                Ok(XmlEvent::EndDocument { .. }) => {}
                Err(e) => {
                    println!("error: {:?}", e);
                    return MarkupParser {
                        path: path.to_string(),
                        failed: true,
                        error: Some(format!("{}", e.msg())),
                        root: None,
                        storage: None,
                    };
                }
                _ => {}
            };
        }
        MarkupParser {
            path: path.to_string(),
            failed: false,
            error: None,
            root: root_node.clone(),
            storage: Some(Rc::new(RefCell::new(storage))),
        }
    }

    // Instance methods

    fn process_block(&self, child: &MarkupElement) -> Block {
        let title = extract_attribute(child.attributes.clone(), "title");
        let border = extract_attribute(child.attributes.clone(), "border");
        let border = MarkupParser::<B>::get_border(border.as_str());
        let block = Block::default().title(title).borders(border);
        block
    }

    fn process_paragraph(&self, child: &MarkupElement) -> Paragraph {
        let styles = MarkupParser::<B>::get_styles(&child.clone());
        let alignment = MarkupParser::<B>::get_alignment(&child.clone());
        let block = self.process_block(&child.clone());
        let p = Paragraph::new(child.text.clone())
            .style(styles)
            .alignment(alignment)
            .block(block);
        p
    }

    fn draw_element(&self, frame: &mut Frame<B>, area: Rect, node: &MarkupElement) {
        let name = node.name.clone();
        let name = name.as_str();
        let storage = self.storage.clone();
        let storage = storage.unwrap();
        let storage = storage.as_ref();
        let storage = storage.borrow_mut();
        if storage.has_component(name) {
            storage.render(name, frame);
        } else {
            match name {
                "block" => {
                    let widget = self.process_block(&node);
                    frame.render_widget(widget, area);
                }
                "p" => {
                    let widget = self.process_paragraph(&node);
                    frame.render_widget(widget, area);
                }
                _ => {
                    let widget = Block::default();
                    frame.render_widget(widget, area);
                }
            };
        }
    }

    fn process_layout(
        &self,
        frame: &mut Frame<B>,
        node: &MarkupElement,
        place: Option<Rect>,
        margin: Option<u16>,
    ) -> Vec<(Rect, MarkupElement)> {
        let direction = MarkupParser::<B>::get_direction(node);
        let mut res: Vec<(Rect, MarkupElement)> = vec![];
        let mut constraints: Vec<Constraint> = vec![];
        let mut widgets_info: Vec<(usize, MarkupElement)> = vec![];
        let mut layouts_info: Vec<(usize, MarkupElement)> = vec![];
        for (position, child) in node.children.iter().enumerate() {
            let borrowed_child = child.as_ref().borrow();
            if borrowed_child.name.eq("container") {
                let constraint = extract_attribute(borrowed_child.attributes.clone(), "constraint");
                constraints.push(MarkupParser::<B>::get_constraint(constraint));
                let children = borrowed_child.children.clone();
                children
                    .iter()
                    .map(|child| child.as_ref().borrow())
                    .for_each(|child| {
                        let child_name = child.name.as_str();
                        if MarkupParser::<B>::is_widget(child_name) {
                            let son = child.clone();
                            if son.children.len() > 0 {
                                let son = son.children[0].clone();
                                let son = son.as_ref();
                                let son = son.borrow();
                                let son_name = son.name.as_str();
                                if son_name.eq("layout") {
                                    layouts_info.push((position, son.clone()));
                                    widgets_info.push((position, child.clone()));
                                } else {
                                    widgets_info.push((position, child.clone()));
                                }
                            } else {
                                widgets_info.push((position, child.clone()));
                            }
                        } else if MarkupParser::<B>::is_layout(child_name) {
                            let partial_res = self.process_node(frame, node, None, None);
                            for pair in partial_res.iter() {
                                res.push((pair.0, pair.1.clone()));
                            }
                        }
                    })
            }
        }

        let layout = Layout::default()
            .direction(direction)
            .margin(margin.unwrap_or(0))
            .constraints(constraints.clone().as_ref());

        let chunks = layout.split(place.unwrap_or(frame.size()));

        for (cntr, widget_info) in widgets_info.iter() {
            let counter = *cntr;
            res.push((chunks[counter].clone(), widget_info.clone()));
        }

        for (cntr, layout_info) in layouts_info.iter() {
            let counter = *cntr;
            let place = Some(chunks[counter].clone());
            let parent = layout_info.parent_node.clone().unwrap();
            let parent = parent.as_ref().borrow();
            let border_value = extract_attribute(parent.attributes.clone(), "border");
            let margin = if border_value.eq("none") {
                None
            } else {
                Some(1)
            };
            let partial_res = self.process_node(frame, &layout_info, place, margin);
            for pair in partial_res.iter() {
                res.push((pair.0, pair.1.clone()));
            }
        }
        res
    }

    fn process_node(
        &self,
        frame: &mut Frame<B>,
        node: &MarkupElement,
        place: Option<Rect>,
        margin: Option<u16>,
    ) -> Vec<(Rect, MarkupElement)> {
        let name = node.name.clone();
        let name = name.as_str();
        let values: Vec<(Rect, MarkupElement)> = match name {
            "layout" => self.process_layout(frame.borrow_mut(), node, place, margin),
            _ => {
                panic!("Invalid node type \"{}\"", name);
            }
        };

        return values;
    }

    pub fn render_ui(&self, frame: &mut Frame<B>) {
        let root = MarkupParser::<B>::get_element(self.root.clone());
        let drawables = self.process_node(frame.borrow_mut(), &root, None, None);
        drawables.iter().for_each(|pair| {
            let area = pair.0;
            let node = pair.1.clone();
            self.draw_element(frame, area, &node);
        });
    }

    // Static

    pub fn get_element(node: Option<Rc<RefCell<MarkupElement>>>) -> MarkupElement {
        let r = node.clone().unwrap();
        let r = r.as_ref().borrow().to_owned();
        r
    }

    pub fn is_widget(node_name: &str) -> bool {
        WIDGET_NAMES.contains(&node_name)
    }

    pub fn is_layout(node_name: &str) -> bool {
        node_name.eq("layout")
    }

    pub fn get_border(border_value: &str) -> Borders {
        let border = String::from(border_value);
        if border.contains("|") {
            let borders = border
                .split("|")
                .map(|s| String::from(s))
                .map(|s| MarkupParser::<B>::get_border(&s))
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
}
