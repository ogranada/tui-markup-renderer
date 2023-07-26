use clap::Parser;
use crossterm::event::KeyCode::{self, Char};
use std::{collections::HashMap, io};
use tui::backend::CrosstermBackend;
use tui_markup_renderer::{event_response::EventResponse, markup_parser::MarkupParser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("run"))]
    execution_type: String,
    #[arg(short, long, default_value_t = String::from("./assets/layout.tml"))]
    layout: String,
    #[arg(short, long, default_value_t = false)]
    print_args: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args {
        layout,
        execution_type,
        print_args,
    } = Args::parse();

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let state = Some(HashMap::new());
    /*
    let state = Some(HashMap::from([(
        "tabs-cmp:index".to_string(),
        "tab2".to_string(),
    )]));
    */

    let mut mp = MarkupParser::new(layout.clone(), None, state);
    mp.add_action("do_something", |state, _node| {
        let mut state = state.clone();
        let key = "showMessageDialog".to_string();
        state.insert(key, "true".to_string());
        EventResponse::STATE(state.clone())
    })
    .add_action("do_something_else", |state, _node| {
        let mut state = state.clone();
        let key = "showMessageDialog".to_string();
        state.insert(key, "true".to_string());
        EventResponse::STATE(state.clone())
    })
    .add_action("on_dlg1_btn_Yes", |_state, _node| EventResponse::QUIT)
    .add_action("on_close_dialog", |state, _node| {
        let mut state = state.clone();
        let key = "showMessageDialog".to_string();
        state.insert(key, "false".to_string());
        EventResponse::STATE(state.clone())
    })
    .add_action("on_dlg1_btn_Cancel", |state, _node| {
        let mut state = state.clone();
        let key = "showQuitDialog".to_string();
        state.insert(key, "false".to_string());
        EventResponse::STATE(state.clone())
    });

    if print_args {
        println!(
            "[layout: {}, execution_type: {}, print_args: {}]",
            layout, execution_type, print_args
        );
    }

    if execution_type == String::from("run") {
        // async move
        mp.ui_loop(backend, |key_event, state| {
            let mut new_state = state.clone();
            let key = "showQuitDialog".to_string();
            // let back_value = String::new();
            let mut pressed = '\n';
            match key_event.code {
                KeyCode::Esc => {
                    pressed = '\r';
                }
                Char(character) => {
                    pressed = character;
                }
                _ => {}
            }

            if pressed == '\r' {
                let new_value = "false";
                new_state.insert(key, new_value.to_string());
                return EventResponse::STATE(new_state);
            }

            if pressed == 'q' {
                let new_value = "true";
                new_state.insert(key, new_value.to_string());
                return EventResponse::STATE(new_state);
            }

            if pressed == 'p' {
                return EventResponse::QUIT;
            }
            return EventResponse::NOOP;
        })
    } else {
        env_logger::init();
        mp.test_check(backend)
    }
}
