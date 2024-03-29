use std::collections::HashMap;

pub enum EventResponse {
    NOOP,
    QUIT,
    STATE(HashMap<String, String>),
    CLEANFOCUS(HashMap<String, String>),
}

