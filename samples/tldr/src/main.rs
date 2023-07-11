use crossterm::event::KeyCode::{self, Char};
use std::{collections::HashMap, io};
use tui::backend::CrosstermBackend;
use tui_markup_renderer::{event_response::EventResponse, markup_parser::MarkupParser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get access to StdOut
    let stdout = io::stdout();
    // Get the backend for TUI
    let backend = CrosstermBackend::new(stdout);
    // prepare the internal state for the app info
    let state = Some(HashMap::new());

    // prepare the markup parser
    let mut mp = MarkupParser::new("./assets/layout.tml".to_string(), None, state);

    // Dialogs generate button identifiers following the convention "on_<dialog id>_btn_<button name>"
    mp.add_action("open_dialog", |state| {
        let mut state = state.clone();
        state.insert("show_dialog".to_string(), "true".to_string());
        EventResponse::STATE(state)
    })
    .add_action("on_dlg_btn_Okay", |state| {
        let mut state = state.clone();
        state.insert("show_dialog".to_string(), "false".to_string());
        EventResponse::STATE(state)
    })
    .ui_loop(backend, |key_event, mut state| {
        let mut pressed = "none";
        match key_event.code {
            KeyCode::Esc => {
                pressed = "close_dialog";
            }
            Char('q') => {
                pressed = "close";
            }
            _ => {}
        }

        match pressed {
            "close_dialog" => {
                state.insert("show_dialog".to_string(), "false".to_string());
                EventResponse::STATE(state)
            }
            "close" => {
                state.insert("show_dialog".to_string(), "false".to_string());
                EventResponse::QUIT
            }
            _ => EventResponse::NOOP,
        }
    })
}
