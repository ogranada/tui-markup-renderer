use crate::{
    markup_element::MarkupElement,
    utils::{extract_attribute, get_alignment, get_border, get_styles},
};
use std::collections::HashMap;
use tui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Paragraph},
    Frame,
};

fn process_block<B: Backend>(child: &MarkupElement, area: Rect, f: &mut Frame<B>) -> Option<()> {
    let title = extract_attribute(child.attributes.clone(), "title");
    let border = extract_attribute(child.attributes.clone(), "border");
    let border = get_border(border);
    let block = Block::default().title(title).borders(border);
    f.render_widget(block, area);
    Some(())
}

fn process_paragraph<B: Backend>(
    child: &MarkupElement,
    area: Rect,
    f: &mut Frame<B>,
) -> Option<()> {
    let styles = get_styles(&child.clone());
    let alignment = get_alignment(&child.clone());
    let title = extract_attribute(child.attributes.clone(), "title");
    let border = extract_attribute(child.attributes.clone(), "border");
    let border = get_border(border);
    let block = Block::default().title(title).borders(border);
    let p = Paragraph::new(child.text.clone())
        .style(styles)
        .alignment(alignment)
        .block(block);
    f.render_widget(p, area);
    Some(())
}

pub struct RenderActions<'a, B: Backend> {
    storage: HashMap<&'a str, fn(node: &MarkupElement, area: Rect, f: &mut Frame<B>) -> Option<()>>,
}

impl<'a, B: Backend> RenderActions<'a, B> {

    /// Creates a new RenderActions container
    ///
    pub fn new() -> RenderActions<'a, B> {
        let mut storage = HashMap::new();
        let fnc: fn(node: &MarkupElement, area: Rect, f: &mut Frame<B>) -> Option<()> =
            process_block::<B>;
        storage.insert("block", fnc);
        let fnc: fn(node: &MarkupElement, area: Rect, f: &mut Frame<B>) -> Option<()> =
            process_paragraph::<B>;
        storage.insert("p", fnc);
        RenderActions {
            storage,
        }
    }

    /// Adds a new block into RenderActions container.
    ///
    /// Arguments
    /// 
    /// * `name` - The block name
    /// * `action` - The render action
    ///
    pub fn add_action(
        &mut self,
        name: &'a str,
        action: fn(node: &MarkupElement, area: Rect, f: &mut Frame<B>) -> Option<()>,
    ) {
        self.storage.insert(name, action);
    }

    /// Returns a clone of the actions storage.
    ///
    pub fn get_storage(
        self: &Self,
    ) -> HashMap<&'a str, fn(node: &MarkupElement, area: Rect, f: &mut Frame<B>) -> Option<()>>
    {
        self.storage.clone()
    }
}
