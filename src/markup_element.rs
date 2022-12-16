use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct MarkupAttribute {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct MarkupElement {
    pub deep: usize,
    pub name: String,
    pub text: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<Rc<RefCell<MarkupElement>>>,
    pub parent_node: Option<Rc<RefCell<MarkupElement>>>,
}

impl Clone for MarkupElement {
    fn clone(&self) -> Self {
        MarkupElement {
            deep: self.deep,
            name: self.name.clone(),
            text: self.text.clone(),
            attributes: self.attributes.clone(),
            children: self.children.clone(),
            parent_node: self.parent_node.clone(),
        }
    }
}

impl fmt::Display for MarkupElement {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let attr_vls: String = self
            .attributes
            .keys()
            .map(|key| {
                let value = self.attributes.get(key);
                let value = if value.is_some() { value.unwrap() } else { "" };
                format!(" {}=\"{}\"", key, value)
            })
            .collect();
        let children: String = self
            .children
            .iter()
            .map(|child| format!("{}", child.as_ref().borrow()))
            .collect();
        let tab = "\t".repeat(self.deep);
        let new_str = format!(
            "{}<{}{}>\n{}\n{}</{}>\n",
            tab, self.name, attr_vls, children, tab, self.name
        );
        fmt::Display::fmt(&new_str, f)
    }
}
