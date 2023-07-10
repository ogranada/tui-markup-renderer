#![allow(unused_imports)]
use std::collections::HashMap;
use std::fmt;
use std::io::Stdout;

use tui::style::Style;

pub trait IStylesStorage {
    fn has_rule(self: &Self, name: String) -> bool;
    fn add_rule<'b>(self: &'b mut Self, name: String, styles: Style) -> &'b mut Self;
    fn get_rule(self: &Self, name: String) -> Style;
}

pub struct StylesStorage {
    storage: HashMap<String, Style>,
}

impl StylesStorage {
    pub fn new() -> Self {
        StylesStorage {
            storage: HashMap::new(),
        }
    }
}

impl IStylesStorage for StylesStorage {
    fn add_rule<'b>(self: &'b mut Self, name: String, styles: Style) -> &'b mut Self {
        self.storage.entry(name).or_insert(styles);
        self
    }

    fn has_rule(self: &Self, name: String) -> bool {
        self.storage.contains_key(&name)
    }

    fn get_rule(self: &Self, name: String) -> Style {
        let opt = self.storage.get(&name);
        if opt.is_some() {
            opt.unwrap().clone()
        } else {
            Style::default()
        }
    }
}

impl fmt::Debug for StylesStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut r = f.debug_struct("RenderStorage");
        r.field("Components", &self.storage.keys());
        r.finish()
    }
}
