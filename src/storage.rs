use std::collections::HashMap;
use std::io::Stdout;
use tui::{
    backend::{Backend, CrosstermBackend, TestBackend},
    Frame,
};

type GenFrame<B> = Box<dyn FnMut(&mut Frame<B>) -> ()>;
pub trait RendererStorage<B: Backend> {
    fn add_renderer<'b>(self: &'b mut Self, tagname: &'b str, render: GenFrame<B>) -> &'b mut Self;
}

pub struct TestRendererStorage<'a> {
    storage: HashMap<String, Box<dyn FnMut(&mut Frame<TestBackend>) + 'a>>,
}

impl<'a> TestRendererStorage<'a> {
    pub fn new() -> Self {
        TestRendererStorage {
            storage: HashMap::new(),
        }
    }
}

impl<'a> RendererStorage<TestBackend> for TestRendererStorage<'a> {
    fn add_renderer<'b>(
        self: &'b mut Self,
        tagname: &'b str,
        render: GenFrame<TestBackend>,
    ) -> &'b mut Self {
        self.storage.entry(tagname.to_owned()).or_insert(render);
        self
    }
}

pub struct CrosstermRendererStorage<'a> {
    storage: HashMap<String, Box<dyn FnMut(&mut Frame<CrosstermBackend<Stdout>>) + 'a>>,
}

impl<'a> CrosstermRendererStorage<'a> {
    pub fn new() -> Self {
        CrosstermRendererStorage {
            storage: HashMap::new(),
        }
    }
}

impl<'a> RendererStorage<CrosstermBackend<Stdout>> for CrosstermRendererStorage<'a> {
    fn add_renderer<'b>(
        self: &'b mut Self,
        tagname: &'b str,
        render: GenFrame<CrosstermBackend<Stdout>>,
    ) -> &'b mut Self {
        self.storage.entry(tagname.to_owned()).or_insert(render);
        self
    }
}
