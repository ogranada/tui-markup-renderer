use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use std::{borrow::BorrowMut, io};
use tui_markup::parser::MarkupParser;

use tui::{backend::CrosstermBackend, Terminal};

enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse_args();
    enable_raw_mode().expect("Can't run in raw mode.");

    let stdout = io::stdout();
    // execute!(stdout, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let (tx, rx) = mpsc::channel::<Event<KeyEvent>>();
    let tick_rate = Duration::from_millis(200);

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let mp = MarkupParser::new(String::from("./assets/layout.tml"));

    loop {
        let mut last_pressed = '\n';
        terminal.draw(|frame| {
            _mp.render_ui(frame.borrow_mut());
        })?;
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    last_pressed = 'q';
                }
                _ => {}
            },
            _ => {}
        }
        if last_pressed == 'q' {
            break;
        }
    }

    disable_raw_mode()?;
    terminal.clear()?;
    terminal.show_cursor()?;
    Ok(())
}
