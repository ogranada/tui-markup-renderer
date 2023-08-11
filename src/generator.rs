use clap::Parser;
use std::fs::File;
use std::io::{Write, Error};
use std::process::{Command, self};
use std::str::FromStr;
use std::{fs::create_dir, path::Path};

#[derive(Debug, Clone)]
enum ProjectType {
    SIMPLE,
    SHELL,
}

impl FromStr for ProjectType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "simple" => Ok(ProjectType::SIMPLE),
            "shell" => Ok(ProjectType::SHELL),
            _ => Ok(ProjectType::SIMPLE),
        }
    }
}

/// Program to generate
#[derive(Parser, Debug)]
#[command(name = "tui markup generator")]
#[command(author = "Oscar Andrés Granada <oscar.andres.granadab@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "Create your TUI Markup based apps simplily", long_about = None)]
struct Args {
    /// Project name
    #[arg(short = 'n', long)]
    project_name: String,

    /// Output project path
    #[arg(short = 'o', long)]
    output_path: String,

    /// Project type
    #[arg(short = 't', long)]
    project_type: ProjectType,
}

fn draw_header() {
    println!("");
    println!("╭──────────────────────────╮");
    println!("│   TUI Markup Generator   │");
    println!("╰──────────────────────────╯");
    println!("");
}

fn mkdir(path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(fldr) = path {
        create_dir(fldr)?;
    }
    Ok(())
}

fn write_main_file(path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(target_path) = path {
        let mut file = File::create(target_path)?;
        file.write(
            b"use crossterm::event::KeyCode::Char;
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
    let mut mp = MarkupParser::new(\"./assets/main.xml\".to_string(), None, state);

    // Dialogs generate button identifiers following the convention \"on_<dialog id>_btn_<button name>\"
    mp.add_action(\"do_something\", |state, _node| {
        let mut state = state.clone();
        state.insert(\"new_value\".to_string(), \"true\".to_string());
        EventResponse::STATE(state)
    })
    .ui_loop(backend, |key_event, _state| {
        let mut pressed = \"none\";
        match key_event.code {
            Char('q') => {
                pressed = \"close\";
            }
            _ => {}
        }

        match pressed {
            \"close\" => {
                EventResponse::QUIT
            }
            _ => EventResponse::NOOP,
        }
    })
}"
        )?;
    }
    Ok(())
}

fn write_shell_layout_file(
    project_name: String,
    path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(target_path) = path {
        let mut file = File::create(target_path)?;
        file.write_fmt(format_args!(
            "<layout id=\"root\" direction=\"vertical\">
  <styles>
  </styles>
  <container id=\"nav_container\" constraint=\"5\">
    <p id=\"toolbar\" title=\"Actions\" border=\"all\">
      {}
    </p>
  </container>
  <container id=\"body_container\" constraint=\"10min\">
    <block id=\"body_block\" border=\"none\">
      <layout id=\"content_info\" direction=\"horizontal\">
        <container id=\"ats_container\" constraint=\"20%\" title=\" Side \" border=\"all\">
          <layout id=\"vert_info\" direction=\"vertical\">
            <block id=\"ats_block\" constraint=\"3\">
              <button id=\"btn_hello\" action=\"do_something\" index=\"1\" focus_styles=\"fg:white;bg:gray\"> Action </button> 
            </block>
          </layout>
        </container>
        <container id=\"cnt_container\" constraint=\"20min\">
          <block id=\"cnt_block\" title=\"Cnt\" border=\"all\">
            <tabs id=\"tabs-cmp\" constraint=\"100%\" border=\"all\">
              <tabs-header id=\"t-header\" title=\"Actions\">
                <tab-item id=\"tab1\"> Tab 1 </tab-item>
                <tab-item id=\"tab2\"> Tab 2 </tab-item>
              </tabs-header>
              <tabs-body id=\"t-body\" linked-to=\"tabs1\">
                <tab-content id=\"ctt-1\" for=\"tab1\">
                  <p id=\"prg-1\">
                    Content 1
                  </p>
                </tab-content>
                <tab-content id=\"ctt-2\" for=\"tab2\">
                  <p id=\"prg-2\">
                    Content 2
                  </p>
                </tab-content>
              </tabs-body>
            </tabs>
          </block>
        </container>
      </layout>
    </block>
  </container>
  <container id=\"nav_container\" constraint=\"5\">
    <p id=\"footer\" border=\"all\">
      © Jhon Doe
    </p>
  </container>
  <dialog id=\"dlg1\" show=\"showQuitDialog\" buttons=\"Yes|Cancel\">
    <layout direction=\"vertical\">
      <container constraint=\"3\">
        <p align=\"center\" styles=\"weight:bold\">
          Close Application
        </p>
      </container>
      <container>
        <p align=\"center\">
          Do you want to close the application?
        </p>
      </container>
    </layout>
  </dialog>
  <dialog id=\"dlg2\" show=\"showMessageDialog\" buttons=\"Okay\" action=\"on_close_dialog\">
    <layout direction=\"vertical\">
      <container constraint=\"3\">
        <p align=\"center\" styles=\"weight:bold\">
          Message!
        </p>
      </container>
      <container>
        <p align=\"center\">
          This is a simple message.
        </p>
      </container>
    </layout>
  </dialog>
</layout>",
            project_name,
        ))?;
    }
    Ok(())
}

fn write_simple_layout_file(
    project_name: String,
    path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(target_path) = path {
        let mut file = File::create(target_path)?;
        file.write_fmt(format_args!(
            "<layout>
    <block border=\"all\" title=\"{}\">
        <layout direction=\"vertical\">
            <block constraint=\"5\">
                <p align=\"center\">
                    Hello {}!!!
                </p>
            </block>
            <block constraint=\"5\">
                <p align=\"center\">
                    To quit press Q.
                </p>
            </block>
        </layout>
    </block>
</layout>",
            project_name, project_name,
        ))?;
    }
    Ok(())
}

fn run<'a>(cmd: &'a str, args: &[&'a str], curr_dir: Option<String>) -> (bool, String, String) {
    let cd = curr_dir.unwrap_or(".".to_string());
    let res = Command::new(cmd)
        .args(args)
        .current_dir(cd)
        .output()
        .expect("failed to execute command");
    if res.status.success() {
        let out = String::from_utf8(res.stdout);
        let out = out.unwrap_or(String::default());
        let err = String::from_utf8(res.stderr);
        let err = err.unwrap_or(String::default());
        (true, out.clone(), err.clone())
    } else {
        (false, String::default(), String::default())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let Args {
        output_path,
        project_name,
        project_type,
    } = args;

    draw_header();
    println!(" • Creating project '{}'", project_name);
    let base_path = Path::new(&output_path);
    let base_path = base_path.join(&project_name);
    let path_opt = base_path.to_str();
    let path_assets = base_path.join("assets");
    let path_assets = path_assets.to_str();
    let path_lyt = base_path.join("assets").join("main.xml");
    let path_lyt = path_lyt.to_str();
    let path_main = base_path.join("src").join("main.rs");
    let path_main = path_main.to_str();

    if let Some(target_path) = path_opt {
        let (success, _out, err) = run(
            "cargo",
            &["new", "--name", &project_name.as_str(), target_path],
            None,
        );
        if success {
            println!("\t✓ {}", err.trim());
            println!(" • Installing dependencies.");
            let (deps_success, _, _) = run(
                "cargo",
                &[
                    "add",
                    "tui-markup-renderer@1.1.0",
                    "tui@0.19.0",
                    "crossterm@0.25.0",
                ],
                Some(target_path.to_string()),
            );
            if deps_success {
                run("cargo", &["build"], Some(target_path.to_string()));
                println!("\t✓ Dependencies installed.");
                println!(" • Creating UI files.");
                mkdir(path_assets)?;
                match project_type {
                    ProjectType::SIMPLE => write_simple_layout_file(project_name, path_lyt)?,
                    ProjectType::SHELL => write_shell_layout_file(project_name, path_lyt)?,
                };
                write_main_file(path_main)?;
                println!("\t✓ Files created.");
                println!("\nProject location: {}", target_path);
            }
        } else {
            eprintln!("\n\n!Failed creating project, maybe the project already exists.");
            process::exit(1);
        }
    }
    Ok(())
}
