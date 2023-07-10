use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use log::{info, warn};
#[allow(unused_imports)]
use std::borrow::Borrow;
use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::BufReader,
    panic,
    path::Path,
    rc::Rc,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
    vec::Vec,
    {borrow::BorrowMut, cell::RefCell},
};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    // style::Style,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
    Terminal,
};
use xml::reader::{EventReader, XmlEvent};

use crate::{
    actions::{ActionsStorage, IActionsStorage},
    event_response::EventResponse,
    markup_element::MarkupElement,
    storage::{IRendererStorage, RendererStorage},
    styles::{IStylesStorage, StylesStorage},
    utils::{color_from_str, extract_attribute, modifier_from_str},
};

////////////// END LIBS //////////////

type ActionCallback = fn(&mut HashMap<String, String>) -> EventResponse;

pub enum Event<I> {
    Input(I),
    Tick,
}

const WIDGET_NAMES: &'static [&'static str] = &["p", "button"];

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

pub struct MarkupParser<B: Backend> {
    pub path: String,
    pub failed: bool,
    pub error: Option<String>,
    pub root: Option<Rc<RefCell<MarkupElement>>>,
    pub storage: Option<Rc<RefCell<RendererStorage<B>>>>,
    pub current: i32,
    pub indexed_elements: Vec<MarkupElement>,
    pub contexts: Vec<(String, Vec<MarkupElement>)>,
    pub state: HashMap<String, String>,
    pub actions: ActionsStorage,
    pub global_styles: StylesStorage,
}

impl<B: Backend> fmt::Debug for MarkupParser<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut r = f.debug_struct("MarkupParser");
        r.field("failed", &self.failed);
        r.field("root", &self.root);
        r.finish()
    }
}

impl<B: Backend> MarkupParser<B> {
    // Constructor
    pub fn new(
        path: String,
        optional_storage: Option<RendererStorage<B>>,
        initial_state: Option<HashMap<String, String>>,
    ) -> MarkupParser<B> {
        // env_logger::init();
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
        let mut global_styles = StylesStorage::new();
        let mut indexed_elements = vec![];
        let mut cntr = 0;
        for e in parser {
            cntr += 1;
            match e {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    let mut attrs = HashMap::new();
                    for attr in attributes {
                        attrs.insert(attr.name.local_name, attr.value);
                    }
                    let unknown_id = format!("unknown_elm_{}", cntr);
                    let _id = attrs.get("id").unwrap_or(&unknown_id);
                    let unknown_idx = "-1".to_owned();
                    let elm_idx = attrs
                        .get("index")
                        .unwrap_or(&unknown_idx)
                        .parse::<i32>()
                        .unwrap_or(-1);
                    let partial = MarkupElement {
                        deep: if parent_node.is_some() {
                            MarkupParser::<B>::get_element(parent_node.clone()).deep + 1
                        } else {
                            0
                        },
                        id: String::from(_id),
                        text: None,
                        order: elm_idx,
                        name: name.local_name,
                        attributes: attrs,
                        children: vec![],
                        parent_node: parent_node.clone(),
                        dependencies: vec![],
                    };

                    current_node = Some(Rc::new(RefCell::new(partial.clone())));

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

                    if elm_idx != -1 {
                        indexed_elements.push(partial);
                    }

                    parent_node = current_node.clone();
                }
                Ok(XmlEvent::Characters(ref r)) => {
                    let node = current_node.clone();
                    let node = node.unwrap();
                    let node = node.as_ref();
                    let mut node = node.borrow_mut();
                    node.text = Some(String::from(r.trim()));
                }
                Ok(XmlEvent::EndElement { .. }) => {
                    let p = MarkupParser::<B>::get_element(parent_node.clone());
                    let q = p.clone();
                    if q.name.eq("styles") {
                        global_styles = MarkupParser::<B>::process_styles(q);
                    }
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
                        current: -1,
                        indexed_elements: vec![],
                        contexts: vec![],
                        actions: ActionsStorage::new(),
                        state: HashMap::new(),
                        global_styles: StylesStorage::new(),
                    };
                }
                _ => {}
            };
        }
        indexed_elements.sort_by(|e1, e2| e1.order.cmp(&e2.order));
        let state = initial_state.unwrap_or(HashMap::new());
        MarkupParser {
            path: path.to_string(),
            failed: false,
            error: None,
            root: root_node.clone(),
            storage: Some(Rc::new(RefCell::new(storage))),
            current: -1,
            indexed_elements,
            contexts: vec![],
            actions: ActionsStorage::new(),
            state,
            global_styles,
        }
    }

    // Instance methods
    fn draw_block(
        &self,
        child: &MarkupElement,
        _area: Rect,
        focus: bool,
        base_styles: Style,
    ) -> Block {
        let styles = MarkupParser::<B>::get_styles(&child.clone(), focus);
        let styles = base_styles.patch(styles);
        let title = extract_attribute(child.attributes.clone(), "title");
        if focus {
            styles.fg(Color::Cyan);
        }
        let border = extract_attribute(child.attributes.clone(), "border");
        let border = MarkupParser::<B>::get_border(border.as_str());
        let block = Block::default().title(title).style(styles).borders(border);
        block
    }

    fn draw_paragraph(
        &self,
        child: &MarkupElement,
        area: Rect,
        focus: bool,
        base_styles: Style,
    ) -> Paragraph {
        let styles = MarkupParser::<B>::get_styles(&child.clone(), focus);
        let styles = base_styles.patch(styles);
        if focus {
            styles.fg(Color::Cyan);
        }
        let alignment = MarkupParser::<B>::get_alignment(&child.clone());
        let block = self.draw_block(&child.clone(), area.clone(), focus, base_styles.clone());
        let p = Paragraph::new(child.text.clone().unwrap_or(String::from("")))
            .style(styles)
            .alignment(alignment)
            .wrap(Wrap { trim: true })
            .block(block);
        p
    }

    fn draw_button(
        &self,
        child: &MarkupElement,
        area: Rect,
        focus: bool,
        base_styles: Style,
    ) -> Paragraph {
        let styles = MarkupParser::<B>::get_styles(&child.clone(), focus);
        let styles = base_styles.patch(styles);
        let mut elcnt = usize::from(area.height);
        if area.height > 0 {
            elcnt = usize::from(area.height / 2 - 1);
        }
        let text = child.text.clone().unwrap_or(String::from(""));
        let mut lns_cntt = vec![];
        for _i in 0..elcnt {
            lns_cntt.push(Spans::from(""));
        }
        lns_cntt.push(Spans::from(Span::styled(
            text,
            if focus {
                styles.add_modifier(Modifier::UNDERLINED)
            } else {
                styles
            },
        )));
        let block = Block::default().style(styles).borders(Borders::ALL);
        let p = Paragraph::new(lns_cntt)
            .style(styles)
            .alignment(Alignment::Center)
            .block(block);
        p
    }

    fn draw_dialog(
        &self,
        child: &MarkupElement,
        _area: Rect,
        focus: bool,
        base_styles: Style,
    ) -> Block {
        let styles = MarkupParser::<B>::get_styles(&child.clone(), focus);
        let styles = base_styles.patch(styles);
        let text = child.text.clone().unwrap_or(String::from(""));
        let mut lns_cntt = vec![];
        lns_cntt.push(Spans::from(""));
        lns_cntt.push(Spans::from(Span::styled(text, styles)));
        let block = Block::default()
            .style(styles)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        block
    }

    fn go_next(self: &mut Self) -> i32 {
        let size = i32::try_from(self.indexed_elements.len()).unwrap() - 2;
        if self.current > size {
            self.current = -1;
        } else {
            self.current += 1;
        }
        self.current
    }

    fn go_prev(self: &mut Self) -> i32 {
        let size = i32::try_from(self.indexed_elements.len()).unwrap() - 1;
        if self.current < 0 {
            self.current = size;
        } else {
            self.current -= 1;
        }
        self.current
    }

    fn do_action(self: &mut Self) -> EventResponse {
        if self.current > -1 {
            let current = self.indexed_elements[self.current as usize].clone();
            let action = extract_attribute(current.attributes.clone(), "action");
            if self.actions.has_action(action.clone()) {
                info!("Executing {}", action);
                let new_state = self
                    .actions
                    .execute(action.clone(), self.state.borrow_mut());
                if new_state.is_some() {
                    let event_response = new_state.unwrap();
                    return event_response
                }
            }
        }
        EventResponse::NOOP
    }

    fn draw_element(
        self: &mut Self,
        frame: &mut Frame<B>,
        area: Rect,
        node: &MarkupElement,
    ) -> bool {
        let name = node.name.clone();
        let name = name.as_str();
        let storage = self.storage.clone();
        let storage = storage.unwrap();
        let storage = storage.as_ref();
        let storage = storage.borrow_mut();
        if storage.has_component(name) {
            storage.render(name, frame);
            return true;
        } else {
            let mut cid = "".to_owned();
            if self.current > -1 {
                cid = self.indexed_elements[self.current as usize].id.clone();
            }
            let same = node.id.eq(&cid);
            let rulename = format!("{}{}", name, if same { ":focus" } else { "" });
            let base_styles = self.global_styles.get_rule(rulename);
            let rulename = format!("#{}", node.id);
            let elm_styles = self.global_styles.get_rule(rulename);
            let base_styles = base_styles.patch(elm_styles);
            frame.render_widget(Clear, area.clone());
            match name {
                "container" | "block" => {
                    let widget = self.draw_block(&node, area.clone(), same, base_styles);
                    frame.render_widget(widget, area);
                    return true;
                }
                "p" => {
                    // println!("--> {:?}", node.clone().dependencies);
                    let widget = self.draw_paragraph(&node, area.clone(), same, base_styles);
                    frame.render_widget(widget, area);
                    return true;
                }
                "dialog" => {
                    let new_node = node.clone();
                    let show_flag = extract_attribute(new_node.clone().attributes, "show");
                    let default_val = "false".to_string();
                    let state_value = self.state.get(&show_flag).unwrap_or(&default_val);
                    // print!("\r    =>> {:?}", self.contexts);
                    if state_value.clone().eq(&"true".to_string()) {
                        self.add_context(&node);
                        let widget = self.draw_dialog(&new_node, area.clone(), same, base_styles);
                        frame.render_widget(widget, area);
                        return true;
                    } else {
                        self.remove_context(&node);
                    }
                    return false;
                }
                "button" => {
                    let mut new_area = area.clone();
                    new_area.height = if new_area.height > 5 {
                        5
                    } else {
                        new_area.height
                    };
                    let widget = self.draw_button(&node, new_area.clone(), same, base_styles);
                    frame.render_widget(widget, new_area);
                    return true;
                }
                _ => {
                    let widget = Block::default();
                    frame.render_widget(widget, area);
                    return true;
                }
            };
        }
    }

    fn process_block(
        &self,
        frame: &mut Frame<B>,
        node: &MarkupElement,
        dependency: Option<MarkupElement>,
        place: Option<Rect>,
        _margin: Option<u16>, // remove or transform in padding?
        count: usize,
    ) -> Vec<(Rect, MarkupElement)> {
        let current = node.clone();
        let split_space = place.unwrap_or(frame.size());
        let border_value = extract_attribute(current.attributes.clone(), "border");
        let mut res: Vec<(Rect, MarkupElement)> = vec![];
        let mut constraints: Vec<Constraint> = vec![];
        let id = extract_attribute(current.attributes.clone(), "id");
        let mut widgets_info: Vec<(usize, MarkupElement)> = vec![];
        let mut children_nodes: Vec<(usize, MarkupElement)> = vec![];
        res.push((place.unwrap_or(frame.size()), current));

        info!(target: "MarkupParser",
            "{}Container #{}[[{:?}]]",
            "".repeat(count * 2),
            id,
            split_space.clone(),
        );

        for (position, base_child) in node.children.iter().enumerate() {
            let child = base_child.as_ref().borrow();
            let constraint = extract_attribute(child.clone().attributes, "constraint");
            constraints.push(MarkupParser::<B>::get_constraint(constraint));
            let child_name = child.clone().name;

            if MarkupParser::<B>::is_widget(child_name.as_str()) {
                widgets_info.push((position, child.clone()));
            } else {
                children_nodes.push((position, child.clone()));
            }
        }

        let new_margin = if border_value.eq("") || border_value.eq("none") {
            0 // margin.unwrap_or(0)
        } else {
            1
        };
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(new_margin)
            .constraints(constraints.clone().as_ref());
        let chunks = layout.split(split_space);

        // println!("{} => {:?} {:#?}", id, constraints.clone(), chunks.clone());

        for (cntr, base_child) in children_nodes.iter() {
            let counter = *cntr;
            let mut child = base_child.clone();
            if dependency.is_some() {
                child.dependencies.push(dependency.clone().unwrap().id);
            }
            let partial_res = self.process_node(
                frame,
                &child,
                dependency.clone(),
                Some(chunks[counter].clone()),
                None,
                count + 1,
            );
            for pair in partial_res.iter() {
                res.push((pair.0, pair.1.clone()));
            }
        }

        for (cntr, widget_info) in widgets_info.iter() {
            let counter = *cntr;
            let mut mkp_elm = widget_info.clone();
            if dependency.is_some() {
                let did = dependency.clone().unwrap().id;
                if !mkp_elm.dependencies.contains(&did) {
                    mkp_elm.dependencies.push(did);
                }
            }
            res.push((chunks[counter].clone(), mkp_elm));
        }

        return res;
    }

    fn process_layout(
        &self,
        frame: &mut Frame<B>,
        node: &MarkupElement,
        dependency: Option<MarkupElement>,
        place: Option<Rect>,
        margin: Option<u16>,
        count: usize,
    ) -> Vec<(Rect, MarkupElement)> {
        let current = node.clone();
        let split_space = place.unwrap_or(frame.size());
        let direction = MarkupParser::<B>::get_direction(node);
        let id = extract_attribute(current.attributes.clone(), "id");
        info!(target: "MarkupParser",
            "{}Layout #{}[{}]({} children) [[{:?}]]",
            " ".repeat(count * 2),
            id,
            current.attributes.get("direction").unwrap(),
            node.children.len(),
            split_space.clone(),
        );
        let mut res: Vec<(Rect, MarkupElement)> = vec![];
        let constraints: Vec<Constraint> = MarkupParser::<B>::get_constraints(node.clone());
        info!(target: "MarkupParser", "{}  ::>{:?}", "".repeat(count * 2), constraints);

        let layout = Layout::default()
            .direction(direction)
            .margin(margin.unwrap_or(0))
            .constraints(constraints.clone().as_ref());

        let chunks = layout.split(split_space);

        for (position, base_child) in node.children.iter().enumerate() {
            let mut child = base_child.as_ref().borrow().clone();
            if dependency.is_some() {
                child.dependencies.push(dependency.clone().unwrap().id);
            }
            let partial_res = self.process_node(
                frame,
                &child,
                dependency.clone(),
                Some(chunks[position].clone()),
                Some(1),
                count + 1,
            );
            for pair in partial_res.iter() {
                let mut mkp_elm = pair.1.clone();
                if dependency.is_some() {
                    let did = dependency.clone().unwrap().id;
                    if !mkp_elm.dependencies.contains(&did) {
                        mkp_elm.dependencies.push(did);
                    }
                }
                res.push((pair.0, mkp_elm));
            }
        }

        return res;
    }

    fn process_other(
        &self,
        frame: &mut Frame<B>,
        node: &MarkupElement,
        depends_on: Option<MarkupElement>,
        place: Option<Rect>,
        margin: Option<u16>,
        count: usize,
    ) -> Option<Vec<(Rect, MarkupElement)>> {
        let mut current = node.clone();
        let id = extract_attribute(current.attributes.clone(), "id");
        let mut split_space = place.unwrap_or(frame.size());
        let mut res: Vec<(Rect, MarkupElement)> = vec![];
        let mut subsequents: Vec<(Rect, MarkupElement)> = vec![];
        let mut dependency = depends_on;

        info!(target: "MarkupParser",
            "{}Other #{}[[{:?}]]",
            "".repeat(count * 2),
            id,
            split_space.clone(),
        );

        match node.name.as_str() {
            "dialog" => {
                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(margin.unwrap_or(0))
                    .constraints(
                        vec![
                            Constraint::Percentage(35),
                            Constraint::Percentage(30),
                            Constraint::Percentage(35),
                        ]
                        .as_ref(),
                    );
                let h_chunks = layout.split(frame.size());

                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(margin.unwrap_or(0))
                    .constraints(
                        vec![
                            Constraint::Percentage(35),
                            Constraint::Percentage(30),
                            Constraint::Percentage(35),
                        ]
                        .as_ref(),
                    );
                let chunks = layout.split(h_chunks[1]);

                let dialog_parts = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        vec![Constraint::Percentage(60), Constraint::Percentage(40)].as_ref(),
                    );
                let location = chunks[1].clone();
                let dialog_chunks = dialog_parts.split(chunks[1]);
                let buttons_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(1)
                    .constraints(
                        vec![Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                    );
                let button_chunks = buttons_layout.split(dialog_chunks[1]);
                let btns = extract_attribute(node.attributes.clone(), "buttons");
                let btns = btns.split("|");
                for (elm_idx, btn) in btns.enumerate() {
                    let btn_id = format!("{}_btn_{}", node.id, btn);
                    let btn_elm = MarkupElement {
                        deep: node.deep + 1,
                        id: btn_id.clone(),
                        text: Some(String::from(btn)),
                        order: elm_idx as i32,
                        name: String::from("button"),
                        attributes: HashMap::from([
                            ("id".to_string(), btn_id.clone()),
                            ("action".to_string(), format!("on_{}", btn_id)),
                            ("index".to_string(), format!("{}", elm_idx)),
                        ]),
                        children: vec![],
                        parent_node: Some(Rc::new(RefCell::new(node.clone()))),
                        dependencies: vec![node.id.clone()],
                    };
                    let btn_desc = Rc::new(RefCell::new(btn_elm.clone()));
                    current.children.push(btn_desc);
                    subsequents.push((button_chunks[elm_idx], btn_elm));
                }

                split_space = location;
                dependency = Some(node.clone());
            }
            _ => {
                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(margin.unwrap_or(0))
                    .constraints(vec![Constraint::Percentage(100)].as_ref());
                split_space = layout.split(place.unwrap_or(frame.size()))[0];
            }
        }
        res.push((split_space, current));

        for base_child in node.children.iter() {
            let mut child = base_child.as_ref().borrow().clone();
            if dependency.is_some() {
                child.dependencies.push(dependency.clone().unwrap().id);
            }
            let partial_res = self.process_node(
                frame,
                &child,
                dependency.clone(),
                Some(split_space.clone()),
                Some(1),
                count + 1,
            );
            for pair in partial_res.iter() {
                let mut mkp_elm = pair.1.clone();
                if dependency.is_some() {
                    let did = dependency.clone().unwrap().id;
                    if !mkp_elm.dependencies.contains(&did) {
                        mkp_elm.dependencies.push(did);
                    }
                }
                res.push((pair.0, mkp_elm));
            }
        }

        for shld in subsequents {
            res.push(shld);
        }

        Some(res)
    }

    fn process_node(
        &self,
        frame: &mut Frame<B>,
        node: &MarkupElement,
        depends_on: Option<MarkupElement>,
        place: Option<Rect>,
        margin: Option<u16>,
        count: usize,
    ) -> Vec<(Rect, MarkupElement)> {
        let name = node.name.clone();
        let name = name.as_str();
        let values: Vec<(Rect, MarkupElement)> = match name {
            "styles" => vec![],
            "layout" => {
                self.process_layout(frame.borrow_mut(), node, depends_on, place, margin, count)
            }
            "container" => {
                self.process_block(frame.borrow_mut(), node, depends_on, place, margin, count)
            }
            "block" => {
                self.process_block(frame.borrow_mut(), node, depends_on, place, margin, count)
            }
            _ => {
                let res =
                    self.process_other(frame.borrow_mut(), node, depends_on, place, margin, count);
                if res.is_none() {
                    warn!("Unknown node type \"{}\"", name);
                    vec![]
                } else {
                    res.unwrap()
                }
            }
        };
        return values;
    }

    pub fn add_action(self: &mut Self, name: &str, action: ActionCallback) -> &mut Self {
        self.actions.add_action(String::from(name), action);
        self
    }

    fn can_be_drawn(self: &Self, node: MarkupElement, drawn: &Vec<String>) -> bool {
        let others = node.dependencies;
        if others.len() == 0 {
            return true;
        }
        let mut res = false;
        // println!("-> {:?}", &node.dependencies);
        for eid in others {
            if drawn.contains(&eid) {
                res = true;
            }
        }
        res
    }

    /// Render the current state of the tree
    ///
    pub fn render_ui(self: &mut Self, frame: &mut Frame<B>) {
        let elm = self.root.clone();
        if elm.is_some() {
            let root = MarkupParser::<B>::get_element(elm);
            let drawables = self.process_node(frame.borrow_mut(), &root, None, None, None, 0);
            let mut drawn: Vec<String> = vec![];
            drawables.iter().for_each(|pair| {
                let area = pair.0;
                let node = pair.1.clone();
                if self.can_be_drawn(node.clone(), &drawn) {
                    let done = self.draw_element(frame, area, &node);
                    if done {
                        drawn.push(node.id);
                    }
                }
            });
        } else {
            panic!("Critical error parsing file. check file format.");
        }
    }

    pub fn add_context(self: &mut Self, node: &MarkupElement) {
        let loc = self.contexts.len();
        let current = self.contexts.get(loc);
        let must_insert = current.is_some() && !current.unwrap().0.eq(&node.id);
        if loc == 0 || must_insert {
            self.contexts
                .push((node.id.clone(), self.indexed_elements.clone()));
            let chld: Vec<MarkupElement> = node
                .clone()
                .children
                .iter()
                .map(|x| x.as_ref().borrow().clone())
                .filter(|x| x.order > -1)
                .collect();
            self.indexed_elements = chld;
            self.current = -1;
        }
    }

    pub fn remove_context(self: &mut Self, node: &MarkupElement) {
        let loc = self.contexts.len();
        if loc > 0 {
            let partial = self.contexts[loc - 1].clone();
            if partial.0.eq(&node.id) {
                self.indexed_elements = partial.1;
                self.contexts.pop();
                self.current = -1;
            }
        }
    }

    pub fn test_check(self: &Self, backend: B) -> Result<(), Box<dyn std::error::Error>> {
        let elm = self.root.clone();
        if elm.is_some() {
            let mut terminal = Terminal::new(backend)?;
            let root = MarkupParser::<B>::get_element(elm);
            terminal.draw(|frame| {
                let drawables = self.process_node(frame.borrow_mut(), &root, None, None, None, 0);
                let ids: Vec<String> = drawables
                    .iter()
                    .map(|x| format!("{}#{}", x.1.name, x.1.id))
                    .collect();
                println!("{:#?}", drawables);
                println!("{:#?}", ids);
            })?;
        }
        println!("{:#?}", self.global_styles);
        Ok(())
    }

    /// Starts a render loop. the loop receive a callback thar will return true
    /// if the loop must finish.
    ///
    /// - *on_event*: callback thar receive a key event.
    ///
    pub fn ui_loop(
        self: &mut Self,
        backend: B,
        on_event: impl Fn(crossterm::event::KeyEvent, HashMap<String, String>) -> EventResponse,
        // on_event: impl Fn(crossterm::event::KeyEvent) -> bool,
    ) -> Result<(), Box<dyn std::error::Error>>
// pub fn ui_loop<Fut>(
//     on_event: impl Fn(crossterm::event::KeyEvent) -> Fut,
     // ) -> Result<(), Box<dyn std::error::Error>>
     // where
     //     Fut: Future<Output = bool>,
    {
        let mut terminal = Terminal::new(backend)?;

        enable_raw_mode().expect("Can't run in raw mode.");
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

        let mut should_quit: bool = false;
        loop {
            terminal.draw(|frame| {
                self.render_ui(frame);
            })?;
            let evt: Event<crossterm::event::KeyEvent> = rx.recv()?;
            if let Event::Input(key_event) = evt {
                let event = key_event.clone();
                match event.code {
                    KeyCode::Tab => {
                        self.go_next();
                    }
                    KeyCode::BackTab => {
                        self.go_prev();
                    }
                    KeyCode::Enter => {
                        let res = self.do_action();
                        match res {
                            EventResponse::QUIT => {
                                should_quit = true;
                            }
                            EventResponse::STATE(state) => {
                                self.state = state;
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        info!("{:?}", key_event);
                    }
                }
                let response =
                    on_event(key_event as crossterm::event::KeyEvent, self.state.clone());
                match response {
                    EventResponse::QUIT => {
                        should_quit = true;
                    }
                    EventResponse::STATE(new_state) => {
                        self.state = new_state;
                    }
                    EventResponse::NOOP => {}
                }
                if should_quit {
                    break;
                }
            }
        }

        disable_raw_mode()?;
        terminal.clear()?;
        terminal.show_cursor()?;
        Ok(())
    }

    // Static

    fn get_constraints(node: MarkupElement) -> Vec<Constraint> {
        let mut constraints: Vec<Constraint> = vec![];
        if node.children.len() > 0 {
            for (_position, base_child) in node.children.iter().enumerate() {
                let child = base_child.as_ref().borrow().clone();
                let constraint = extract_attribute(child.attributes.clone(), "constraint");
                constraints.push(MarkupParser::<B>::get_constraint(constraint));
            }
        }
        constraints
    }

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
        if direction.eq("vertical") {
            Direction::Vertical
        } else {
            Direction::Horizontal
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

    pub fn process_styles(node: MarkupElement) -> StylesStorage {
        let mut global_styles = StylesStorage::new();
        if node.text.is_some() {
            let text = node.text.clone().unwrap();
            let text = text
                .replace("\n", "")
                .replace("\r", "")
                .replace(" ", "")
                .replace("{", " {")
                .replace("}", "}\n");
            let rules: Vec<_> = text
                .split("\n")
                .filter(|x| x.len() > 0)
                .map(|text| {
                    let nt = String::from(text.clone());
                    let rule_info = nt.replace("}", "");
                    let rule_info: Vec<String> =
                        rule_info.split(" {").map(|x| String::from(x)).collect();
                    let rules = rule_info.clone();
                    let rulename: String = rules.get(0).unwrap().to_string();
                    let properties: String = rules.get(1).unwrap().to_string();
                    (rulename, MarkupParser::<B>::generate_styles(properties))
                })
                .collect();
            for (rulename, styles) in rules.iter() {
                global_styles.add_rule(rulename.clone(), styles.clone());
            }
        }
        global_styles
    }

    fn generate_styles(styles_text: String) -> Style {
        let mut res = Style::default();
        if styles_text.len() < 3 {
            return res;
        }
        let styles_vec = styles_text
            .split(";")
            .filter(|x| x.len() > 0)
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
        if styles.contains_key("weight") {
            let weight = modifier_from_str(styles.get("weight").unwrap());
            res = res.add_modifier(weight);
        }
        res
    }

    pub fn get_styles(node: &MarkupElement, focus: bool) -> Style {
        let key = if focus { "focus_styles" } else { "styles" };
        let styles_text = extract_attribute(node.attributes.clone(), key);
        MarkupParser::<B>::generate_styles(styles_text)
    }
}
