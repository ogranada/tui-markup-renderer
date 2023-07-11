# tui-markup-renderer
Rust library to use TUI and markup to build UI terminal interfaces.

## tl;dr;

Xml Code:
```xml
<layout id="root" direction="vertical">
  <block constraint="10"> <!-- Don't forget the size, 1 by default -->
    <p align="center">
        Press q to quit.
    </p>
  </block>
  <block id="bts_block" constraint="6">
    <button id="btn_hello" action="open_dialog" index="1"> Say Hello </button>
  </block>
  <dialog id="dlg" show="show_dialog" buttons="Okay" action="on_dialog_event">
    <block id="dlg_block" border="all">
      <p align="center">
        Hello World!!!
      </p>
    </block>
  </dialog>
</layout>
```

Rust Code:

```rust
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
```
<img width="1503" alt="image" src="https://github.com/ogranada/tui-markup-renderer/assets/1445677/f03ecbb0-9c81-4617-aa6b-4c6d749d26c9">


## Explanation

### How it works

As a developer is easier to create a known data structure describing the user interface.

Sample markup code:

```xml
<layout id="root" direction="vertical">
  <container id="nav_container" constraint="5">
    <p id="toolbar" title="Navigation" border="all" styles="fg:green">
      This is the navigation
    </p>
  </container>
  <container id="body_container" constraint="10min">
    <p id="body" title="Body" border="all" styles="fg:red">
      This is a sample
    </p>
  </container>
</layout>
```

generates:

![Simple Layout](./samples/tui-markup-sample/simple_layout.png)

### A more complex sample:

```xml
<layout id="root" direction="vertical">
  <container id="nav_container" constraint="5">
    <p id="toolbar" title="Actions" border="all" styles="fg:green">
      This is a sample
    </p>
  </container>
  <container id="body_container" constraint="10min">
    <block id="body_block" border="none">
      
      <layout id="content_info" direction="horizontal">
        <container id="ats_container" constraint="20%">
          <block id="ats_block" title="Ats" border="all">
      
          </block>
        </container>
        <container id="cnt_container" constraint="20min">
          <block id="cnt_block" title="Cnt" border="all">
            
          </block>
        </container>
      </layout>

    </block>
  </container>
  <container id="nav_container" constraint="5">
    <p id="footer" border="all" styles="bg:red;fg:black">
      This is a sample
    </p>
  </container>
</layout>
```

generates:

![Sample Layout](./samples/tui-markup-sample/layout.png)

## Planned features

* Add documentation to use it.
* Runtime template change.

## The Rules!

* A layout allow dev to define the direction flow.
* A block is a panel that can have:
  - borders
  - title
  - constarint to define size of the element.
* A block can be parent of a layout.
* A container is a alias of a block.
* A layout should contains blocks/containers as children in order to set user interfaces.
  However, the root layout cound have some elements (like dialogs).
* Every element can have an identifier (_id_), but the identifiers mut be uniques.
* You can create global styles using the _styles_ tag or the _styles_ property for elements.
* The styles cover (for now):
  - bg (background color).
  - fg (foreground color).
  - weight (font weight).
* You can have a UI state to store UI information. 

## A Sample

Layout code:

Something better?

```xml
<layout id="root" direction="vertical">
  <styles>

    button {
      fg: red;
      bg: black;
    }

    button:focus {
      fg: white;
      bg: red;
    }
    #footer {
      bg:black;
      fg:blue;
    }
  </styles>
  <container id="nav_container" constraint="5">
    <p id="toolbar" title="Actions" border="all" styles="fg:green">
      Header sample
    </p>
  </container>
  <container id="body_container" constraint="10min">
    <block id="body_block" border="none">

      <layout id="content_info" direction="horizontal">
        <container id="ats_container" constraint="20%" title="Ats" border="all">

          <layout id="vert_info" direction="vertical">
            <block id="ats_block" constraint="5">
              <button id="btn_hello" action="do_something" index="1" styles="fg:magenta" focus_styles="fg:white;bg:magenta"> Hello </button>
            </block>
            <block id="bts_block" constraint="5">
              <button id="btn_hello_2" action="do_something_else" index="3"> Simple </button>
            </block>
            <block id="bts_block" constraint="5">
              <button id="btn_hello_3" action="do_something_else" index="2"> World </button>
            </block>
          </layout>

        </container>
        <container id="cnt_container" constraint="20min">
          <block id="cnt_block" title="Cnt" border="all">
            <p>
              lorem ipsum dolor sit amet sample.
            </p>
          </block>
        </container>
      </layout>

    </block>
  </container>
  <container id="nav_container" constraint="5">
    <p id="footer" border="all">
      Footer sample
    </p>
  </container>
  <dialog id="dlg1" show="showQuitDialog" buttons="Yes|Cancel" action="on_dialog_event">
    <layout direction="vertical">
      <container constraint="3">
        <p align="center" styles="weight:bold">
          Close Application
        </p>
      </container>
      <container>
        <p align="center">
          Do you want to close the application?
        </p>
      </container>
    </layout>
  </dialog>
</layout>
```

Rust Code:

```rust
use clap::Parser;
use crossterm::event::KeyCode::{Char, self};
use std::{collections::HashMap, io};
use tui::backend::CrosstermBackend;
use tui_markup_renderer::{
    markup_parser::MarkupParser,
    event_response::EventResponse,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("run"))]
    execution_type: String,
    #[arg(short, long, default_value_t = String::from("./assets/layout1.tml"))]
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

    let mut mp = MarkupParser::new(layout.clone(), None, state);
    mp.add_action(
        "do_something",
        |_state: &mut HashMap<String, String>| {
            println!("hello!!!");
            EventResponse::NOOP
        },
    )
    .add_action(
        "do_something_else",
        |_state: &mut HashMap<String, String>| {
            println!("world!!!");
            EventResponse::NOOP
        },
    )
    .add_action(
        "on_dlg1_btn_Yes",
        |_state: &mut HashMap<String, String>| {
            EventResponse::QUIT
        },
    )
    .add_action(
        "on_dlg1_btn_Cancel",
        |state: &mut HashMap<String, String>| {
            let key = "showQuitDialog".to_string();
            state.insert(key, "false".to_string());
            EventResponse::STATE(state.clone())
        },
    )
    ;

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
                new_state.insert(
                    key,
                    new_value.to_string(),
                );
                return EventResponse::STATE(new_state);
            }

            if pressed == 'q' {
                let new_value = "true";
                new_state.insert(
                    key,
                    new_value.to_string(),
                );
                return EventResponse::STATE(new_state);
            }

            return EventResponse::NOOP;
        })
    } else {
        env_logger::init();
        mp.test_check(backend)
    }
}


```

Will generate this:

<img width="1506" alt="image" src="https://github.com/ogranada/tui-markup-renderer/assets/1445677/bfce5cf3-d5f7-495a-b4fc-aa4a8c1fd0f5">

<img width="1510" alt="image" src="https://github.com/ogranada/tui-markup-renderer/assets/1445677/59beb3bd-ec2b-4008-9bf7-d876e3c27df6">

<img width="1506" alt="image" src="https://github.com/ogranada/tui-markup-renderer/assets/1445677/cd987ed8-3001-4470-94eb-ac2e2d5dcebc">




