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

type Callback = fn(HashMap<String, String>) -> EventResponse;

pub trait IActionsStorage {
    fn has_action(self: &Self, name: String) -> bool;
    fn add_action<'b>(self: &'b mut Self, name: String, render: Callback) -> &'b mut Self;
    fn execute(self: &Self, name: String, state: HashMap<String, String>) -> Option<EventResponse>;
}

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
    fn add_action<'b>(self: &'b mut Self, name: String, action: Callback) -> &'b mut Self {
        self.storage.entry(name).or_insert(action);
        self
    }

    fn has_action(self: &Self, name: String) -> bool {
        self.storage.contains_key(&name)
    }

    fn execute(self: &Self, name: String, state: HashMap<String, String>) -> Option<EventResponse> {
        let opt = self.storage.get(&name).clone();
        if opt.is_some() {
            let f = opt.unwrap();
            Some(f(state))
        } else {
            None
        }
    }
}

impl fmt::Debug for ActionsStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut r = f.debug_struct("RenderStorage");
        r.field("Components", &self.storage.keys());
        r.finish()
    }
}

