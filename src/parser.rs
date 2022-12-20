use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::rc::Rc;
use std::vec::Vec;
use tui::layout::Rect;
use xml::reader::{EventReader, XmlEvent};

use crate::markup_element::MarkupElement;
use crate::render_actions::RenderActions;
use crate::utils::{
    extract_attribute, get_alignment, get_border, get_constraint, get_direction, get_styles,
    is_layout, is_widget,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Layout},
    widgets::{Block, Paragraph},
    Frame,
};

use crossterm::{
    event::{self, Event as CEvent, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tui::{backend::CrosstermBackend, Terminal};

pub enum Event<I> {
    Input(I),
    Tick,
}

type TCallback = fn(KeyEvent) -> bool;

/*
#[cfg(not(test))]
use std::io::Stdout;
#[cfg(not(test))]
use tui::backend::CrosstermBackend;

#[cfg(test)]
use tui::backend::TestBackend;
*/
#[derive(Debug)]
pub struct MarkupParser {
    pub path: String,
    pub failed: bool,
    pub error: Option<String>,
    pub root: Option<Rc<RefCell<MarkupElement>>>,
}

impl MarkupParser {
    pub fn get_element(node: Option<Rc<RefCell<MarkupElement>>>) -> MarkupElement {
        let r = node.clone().unwrap();
        let r = r.as_ref().borrow().to_owned();
        r
    }
    fn process_block(&self, child: &MarkupElement) -> Block {
        let title = extract_attribute(child.attributes.clone(), "title");
        let border = extract_attribute(child.attributes.clone(), "border");
        let border = get_border(border);
        let block = Block::default().title(title).borders(border);
        block
    }

    fn process_paragraph(&self, child: &MarkupElement) -> Paragraph {
        let styles = get_styles(&child.clone());
        let alignment = get_alignment(&child.clone());
        let block = self.process_block(&child.clone());
        let p = Paragraph::new(child.text.clone())
            .style(styles)
            .alignment(alignment)
            .block(block);
        p
    }

    fn process_layout(
        &self,
        frame_size: Rect,
        node: &MarkupElement,
        place: Option<Rect>,
        margin: Option<u16>,
    ) -> Vec<(Rect, MarkupElement)> {
        let direction = get_direction(node);
        let mut res: Vec<(Rect, MarkupElement)> = vec![];
        let mut constraints: Vec<Constraint> = vec![];
        let mut widgets_info: Vec<(usize, MarkupElement)> = vec![];
        let mut layouts_info: Vec<(usize, MarkupElement)> = vec![];
        for (position, child) in node.children.iter().enumerate() {
            let borrowed_child = child.as_ref().borrow();
            if borrowed_child.name.eq("container") {
                let constraint = extract_attribute(borrowed_child.attributes.clone(), "constraint");
                constraints.push(get_constraint(constraint));
                let children = borrowed_child.children.clone();
                children
                    .iter()
                    .map(|child| child.as_ref().borrow())
                    .for_each(|child| {
                        let child_name = child.name.as_str();
                        if is_widget(child_name) {
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
                        } else if is_layout(child_name) {
                            let partial_res = self.process_node(frame_size, node, None, None);
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

        let chunks = layout.split(place.unwrap_or(frame_size));

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
            let partial_res = self.process_node(frame_size, &layout_info, place, margin);
            for pair in partial_res.iter() {
                res.push((pair.0, pair.1.clone()));
            }
        }
        res
    }

    fn process_node(
        &self,
        frame_size: Rect,
        node: &MarkupElement,
        place: Option<Rect>,
        margin: Option<u16>,
    ) -> Vec<(Rect, MarkupElement)> {
        let name = node.name.clone();
        let name = name.as_str();
        let values: Vec<(Rect, MarkupElement)> = match name {
            "layout" => self.process_layout(frame_size, node, place, margin),
            _ => {
                panic!("Invalid node type \"{}\"", name);
            }
        };

        return values;
    }

    fn draw_element<B: Backend>(&self, frame: &mut Frame<B>, area: Rect, node: &MarkupElement) {
        let name = node.name.clone();
        let name = name.as_str();
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

    pub fn render_ui<B: Backend>(
        &self,
        frame: &mut Frame<B>,
        render_actions: Option<RenderActions<B>>,
    ) {
        let root = MarkupParser::get_element(self.root.clone());
        let drawables = self.process_node(frame.size(), &root, None, None);

        let draw_functions = render_actions.unwrap_or(RenderActions::new());
        let draw_functions = draw_functions.get_storage();

        drawables.iter().for_each(|pair| {
            let area = pair.0;
            let node = pair.1.clone();
            let node_name = node.name.as_str();
            if draw_functions.contains_key(node_name) {
                let fnc = draw_functions.get(node_name).unwrap();
                fnc(&node, area, frame);
            } else {
                self.draw_element(frame, area, &node);
            }
        });
    }

    pub fn new(path: String) -> MarkupParser {
        if !Path::new(&path).exists() {
            panic!("Markup file does not exist at {}", &path);
        }
        let file = File::open(&path).unwrap();
        let buffer = BufReader::new(file);
        let parser = EventReader::new(buffer);
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
                            MarkupParser::get_element(parent_node.clone()).deep + 1
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
                    let p = MarkupParser::get_element(parent_node.clone());
                    parent_node = p.parent_node;
                }
                Ok(XmlEvent::EndDocument { .. }) => {}
                Err(e) => {
                    return MarkupParser {
                        path: path.to_string(),
                        failed: true,
                        error: Some(format!("{}", e.msg())),
                        root: None,
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
        }
    }

    /// Starts a render loop. the loop receive a callback thar will return true
    /// if the loop must finish.
    ///
    /// - *on_event*: callback thar receive a key event.
    ///
    pub fn ui_loop(self: &Self, on_event: TCallback) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode().expect("Can't run in raw mode.");
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let (tx, rx) = mpsc::channel::<Event<KeyEvent>>();
        let tick_rate = Duration::from_millis(200);

        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).expect("poll works") {
                    if let CEvent::Key(key) = event::read().expect("can read events") {
                        tx.send(Event::Input(key)).expect("can send events");
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if let Ok(_) = tx.send(Event::Tick) {
                        last_tick = Instant::now();
                    }
                }
            }
        });

        loop {
            // let mut last_pressed = '\n';
            terminal.draw(|frame| {
                self.render_ui(frame, None);
            })?;
            let evt: Event<KeyEvent> = rx.recv()?;
            if let Event::Input(key_code) = evt {
                let should_quit = on_event(key_code);
                if should_quit {
                    break;
                }
            }
            /*
            match rx.recv()? {
                Event::Input(event) => match event.code {
                    KeyCode::Char('q') => {
                        last_pressed = 'q';
                    }
                    _ => {}
                },
                _ => {}
            }
            if last_pressed == 'q' {
                break;
            }
            */
        }

        disable_raw_mode()?;
        terminal.clear()?;
        terminal.show_cursor()?;
        Ok(())
    }
}
