#![allow(unused_imports)]
use std::collections::HashMap;
use std::io::Stdout;
use std::{
    borrow::{Borrow, BorrowMut},
    fmt,
    ops::Deref,
};
use tui::layout::Rect;
use tui::{
    backend::{Backend, CrosstermBackend, TestBackend},
    Frame,
};

use crate::event_response::EventResponse;
use crate::markup_element::MarkupElement;

type Callback = fn(HashMap<String, String>, Option<MarkupElement>) -> EventResponse;

pub trait IActionsStorage {
    fn has_action(&self, name: String) -> bool;
    fn add_action(&mut self, name: String, render: Callback) -> &mut Self;
    fn execute(&self, name: String, state: HashMap<String, String>, node: Option<MarkupElement>) -> Option<EventResponse>;
}

#[derive(Default)]
pub struct ActionsStorage {
    storage: HashMap<String, Callback>,
}

impl ActionsStorage {
    pub fn new() -> Self {
        ActionsStorage {
            storage: HashMap::new(),
        }
    }
}

impl IActionsStorage for ActionsStorage {
    fn add_action(&mut self, name: String, action: Callback) -> &mut Self {
        self.storage.entry(name).or_insert(action);
        self
    }

    fn has_action(&self, name: String) -> bool {
        self.storage.contains_key(&name)
    }

    fn execute(&self, name: String, state: HashMap<String, String>, node: Option<MarkupElement>) -> Option<EventResponse> {
        let opt = self.storage.get(&name);
        opt.map(|f| f(state, node.clone()))
    }
}

impl fmt::Debug for ActionsStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut r = f.debug_struct("RenderStorage");
        r.field("Components", &self.storage.keys());
        r.finish()
    }
}

