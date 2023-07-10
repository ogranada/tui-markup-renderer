use std::{fmt, cell::RefCell, collections::HashMap, rc::Rc};

pub struct MarkupElement {
    pub deep: usize,
    pub id: String,
    pub name: String,
    pub order: i32,
    pub text: Option<String>,
    pub attributes: HashMap<String, String>,
    pub children: Vec<Rc<RefCell<MarkupElement>>>,
    pub parent_node: Option<Rc<RefCell<MarkupElement>>>,
    pub dependencies: Vec<String>,
}

impl Clone for MarkupElement {
    fn clone(&self) -> Self {
        MarkupElement {
            deep: self.deep,
            id: self.id.clone(),
            name: self.name.clone(),
            text: self.text.clone(),
            order: self.order.clone(),
            attributes: self.attributes.clone(),
            children: self.children.clone(),
            parent_node: self.parent_node.clone(),
            dependencies: self.dependencies.clone(),
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

impl fmt::Debug for MarkupElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut r = f.debug_struct("MarkupElement");
        r.field("id", &self.id);
        r.field("name", &self.name);
        r.field("text", &self.text);
        if self.order != -1 {
            r.field("order", &self.order);
        }
        r.field("attributes", &self.attributes);
        r.field("dependencies", &self.dependencies);
        /*
        r.field("children", &self.children);
        // */
        r.finish()
    }
}

