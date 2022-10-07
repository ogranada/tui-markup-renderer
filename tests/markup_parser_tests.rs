#[cfg(test)]
mod markup_parser {
    use std::env::current_dir;
    use std::error::Error;
    use tui::{backend::TestBackend, buffer::Buffer, layout::Rect, widgets::Block, Terminal};
    use tui_markup_renderer::{
        markup_parser::MarkupParser,
        storage::{RendererStorage, TestRendererStorage},
    };

    //#[should_panic]

    #[test]
    fn creation() -> Result<(), String> {
        let filepath = match current_dir() {
            Ok(exe_path) => format!("{}/tests/assets/creation_sample.tml", exe_path.display()),
            Err(_e) => format!(""),
        };
        let mp = MarkupParser::new(filepath.clone());
        assert_eq!(mp.path, filepath);
        Ok(())
    }

    #[test]
    fn error_handling() {
        let filepath = match current_dir() {
            Ok(exe_path) => format!("{}/tests/assets/bad_sample.tml", exe_path.display()),
            Err(_e) => format!(""),
        };
        let mp = MarkupParser::new(filepath.clone());
        assert!(mp.failed);
        assert!(mp.error.is_some());
        assert_eq!(
            mp.error.unwrap(),
            "Unexpected closing tag: header, expected title"
        );
    }

    #[test]
    fn complete_parsing() {
        let filepath = match current_dir() {
            Ok(exe_path) => format!("{}/tests/assets/real_sample.tml", exe_path.display()),
            Err(_e) => format!(""),
        };
        let mp = MarkupParser::new(filepath.clone());
        assert!(!mp.failed);
        assert!(mp.error.is_none());
        let root = MarkupParser::get_element(mp.root.clone());
        assert_eq!(root.name, "layout");
        assert_eq!(root.children.len(), 2);
    }

    #[test]
    fn render_check() -> Result<(), Box<dyn Error>> {
        let filepath = match current_dir() {
            Ok(exe_path) => format!(
                "{}/tests/assets/sample_single_block.tml",
                exe_path.display()
            ),
            Err(_e) => format!(""),
        };

        let backend = TestBackend::new(10, 3);
        let mut store = TestRendererStorage::new();
        let b = String::from("b");
        store.add_renderer(
            &b,
            Box::new(|f| {
                let b = Block::default().title("Hi!");
                let a = Rect::new(0, 0, 5, 5);
                f.render_widget(b, a);
            }),
        );

        let mp = MarkupParser::new(filepath.clone());

        let mut terminal = Terminal::new(backend)?;
        let frame = terminal.draw(|f| {
            mp.render_ui(f);
        })?;

        assert_eq!(frame.buffer.get(1, 0).symbol, "B");

        let expected = Buffer::with_lines(vec!["┌BTitle──┐", "│        │", "└────────┘"]);
        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn render_check2() -> Result<(), Box<dyn Error>> {
        let filepath = match current_dir() {
            Ok(exe_path) => format!(
                "{}/tests/assets/sample_couple_blocks.tml",
                exe_path.display()
            ),
            Err(_e) => format!(""),
        };
        let mp = MarkupParser::new(filepath.clone());

        let backend = TestBackend::new(10, 10);
        let mut terminal = Terminal::new(backend)?;
        let frame = terminal.draw(|f| {
            mp.render_ui(f);
        })?;

        assert_eq!(frame.buffer.get(1, 0).symbol, "N");
        assert_eq!(frame.buffer.get(1, 3).symbol, "B");

        let expected = Buffer::with_lines(vec![
            "┌Nav─────┐",
            "│        │",
            "└────────┘",
            "┌Body────┐",
            "│        │",
            "│        │",
            "│        │",
            "│        │",
            "│        │",
            "└────────┘",
        ]);
        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn render_check3() -> Result<(), Box<dyn Error>> {
        let filepath = match current_dir() {
            Ok(exe_path) => format!("{}/tests/assets/sample_units.tml", exe_path.display()),
            Err(_e) => format!(""),
        };
        let mp = MarkupParser::new(filepath.clone());

        let backend = TestBackend::new(20, 10);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| {
            mp.render_ui(f);
        })?;

        let expected = Buffer::with_lines(vec![
            "┌Nav───────────────┐",
            "│                  │",
            "│                  │",
            "└──────────────────┘",
            "┌Ats┐┌Cnt──────────┐",
            "│   ││┌Inn┐┌More──┐│",
            "│   │││   ││      ││",
            "│   │││   ││      ││",
            "│   ││└───┘└──────┘│",
            "└───┘└─────────────┘",
        ]);
        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn render_check4() -> Result<(), Box<dyn Error>> {
        let filepath = match current_dir() {
            Ok(exe_path) => format!(
                "{}/tests/assets/sample_nested_blocks.tml",
                exe_path.display()
            ),
            Err(_e) => format!(""),
        };
        let mp = MarkupParser::new(filepath.clone());

        let backend = TestBackend::new(20, 10);
        let mut terminal = Terminal::new(backend)?;
        let frame = terminal.draw(|f| {
            mp.render_ui(f);
        })?;

        assert_eq!(frame.buffer.get(1, 0).symbol, "N");
        assert_eq!(frame.buffer.get(1, 3).symbol, "B");

        let expected = Buffer::with_lines(vec![
            "┌Nav───────────────┐",
            "│                  │",
            "└──────────────────┘",
            "┌Body──────────────┐",
            "│┌Ats┐┌Cnt────────┐│",
            "││   ││           ││",
            "││   ││           ││",
            "││   ││           ││",
            "│└───┘└───────────┘│",
            "└──────────────────┘",
        ]);
        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn render_check5() -> Result<(), Box<dyn Error>> {
        let filepath = match current_dir() {
            Ok(exe_path) => format!("{}/tests/assets/sample_widgets_1.tml", exe_path.display()),
            Err(_e) => format!(""),
        };
        let mp = MarkupParser::new(filepath.clone());

        let backend = TestBackend::new(20, 10);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| {
            mp.render_ui(f);
        })?;

        let expected = Buffer::with_lines(vec![
            "┌Container─────────┐",
            "│      Sample      │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "└──────────────────┘",
        ]);
        terminal.backend().assert_buffer(&expected);

        Ok(())
    }
}
