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

type Callback<B> = fn(&mut Frame<B>);

pub trait IRendererStorage<B: Backend> {
    fn has_component(&self, tagname: &str) -> bool;
    fn add_renderer<'b>(&'b mut self, tagname: &'b str, render: Callback<B>) -> &'b mut Self;
    fn render(&self, tagname: &str, frame: &mut Frame<B>);
}

#[derive(Default)]
pub struct RendererStorage<B: Backend> {
    storage: HashMap<String, Callback<B>>,
}

impl<B: Backend> RendererStorage<B> {
    pub fn new() -> Self {
        RendererStorage {
            storage: HashMap::new(),
        }
    }
}

impl<B: Backend> IRendererStorage<B> for RendererStorage<B> {
    fn add_renderer<'b>(&'b mut self, tagname: &'b str, render: Callback<B>) -> &'b mut Self {
        self.storage.entry(tagname.to_owned()).or_insert(render);
        self
    }

    fn has_component(&self, tagname: &str) -> bool {
        self.storage.contains_key(tagname)
    }

    fn render(&self, tagname: &str, frame: &mut Frame<B>) {
        let opt = self.storage.get(tagname);
        if let Some(f) = opt {
            f(frame);
        }
    }
}

impl<B: Backend> fmt::Debug for RendererStorage<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut r = f.debug_struct("RenderStorage");
        r.field("Components", &self.storage.keys());
        r.finish()
    }
}

