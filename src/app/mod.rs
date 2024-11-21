mod event;

use event::Event;

use crate::{
    buffer::{Buffer, FlushOptions},
    cli::EdiCli,
    escaping::ANSIColor,
    input::{self, Input, InputStream},
    log,
    terminal::Terminal,
    vec2::Vec2,
    window::{Cell, Window},
};

#[derive(Debug)]
enum AppState {
    Stopped,
    Running { prev_state: termios::Termios },
}

#[derive(Debug)]
enum AppMode {
    Normal,
    Insert,
    Terminal { command: String },
}

pub struct App {
    state: AppState,

    mode: AppMode,
    window: Window,
    buffers: Vec<Buffer>,

    cursor_pos: Vec2<usize>,
}

impl App {
    #[must_use]
    pub const fn new() -> Self {
        App {
            mode: AppMode::Normal,
            state: AppState::Stopped,
            window: Window::new(),
            buffers: Vec::new(),

            cursor_pos: Vec2::new(0, 0),
        }
    }

    pub fn setup(&mut self, args: EdiCli) -> Result<(), std::io::Error> {
        let size = Terminal::get_size()?;
        if let Some(f) = args.edit_file {
            let contents = std::fs::read_to_string(f)?;
            self.buffers.push(Buffer::new(
                contents,
                Vec2::new(size.0 as usize, size.1 as usize),
            ));
        }

        let (w, h) = Terminal::get_size()?;
        self.window.resize(w as usize, h as usize);

        Terminal::into_raw()?;

        self.window.set_cursor(self.cursor_pos);
        self.window.rerender()?;
        Terminal::flush()?;

        Ok(())
    }

    pub fn run(&mut self, args: EdiCli) -> Result<(), std::io::Error> {
        self.state = AppState::Running {
            prev_state: Terminal::get_current_state()?,
        };
        self.setup(args)?;

        let input_stream = InputStream::from_read(std::io::stdin());
        self.redraw();
        self.handle_inputs(input_stream)?;

        Terminal::restore_state(&match self.state {
            AppState::Running { prev_state } => prev_state,
            AppState::Stopped => panic!("App::run: state is stopped"),
        })?;

        self.state = AppState::Stopped;

        Ok(())
    }

    fn handle_inputs(&mut self, input_stream: InputStream) -> Result<(), std::io::Error> {
        loop {
            let message = input_stream.recv().unwrap();
            let input = match message {
                input::Message::Input(event) => event,
                input::Message::Error(e) => {
                    log::debug!("handle_inputs: received an error {:?}", e);
                    continue;
                }
            };

            let event = match event::map_input(&input, &self.mode) {
                Some(event) => event,
                None => {
                    log::debug!("handle_inputs: no event for input {:?}", input);
                    continue;
                }
            };

            log::debug!("handle_inputs: received event {:?}", event);

            if self.handle_event(event) {
                break;
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> bool {
        match event {
            Event::SwitchMode(mode) => {
                self.mode = mode;
            }
            Event::InsertChar(c) => {
                self.window
                    .put_cell(self.cursor_pos, Cell::new(c, ANSIColor::Green));
                self.cursor_pos.x = self.cursor_pos.x.saturating_add(1);
                self.window.set_cursor(self.cursor_pos);
                let _ = self.window.render();
            }
            Event::Quit => return true,
        }

        false
    }

    fn redraw(&mut self) {
        log::debug!("redraw: drawing {} buffers", self.buffers.len());
        self.buffers.iter().for_each(|b| {
            let opts = FlushOptions::default()
                .with_wrap(true)
                .with_highlights(highlight_naive(b.inner()));
            b.flush(&mut self.window, opts)
        });
        let _ = self.window.render();
    }
}

fn highlight_naive(line: &str) -> Vec<(Vec2<usize>, ANSIColor)> {
    let hl_words = vec![
        "fn", "let", "mut", "use", "mod", "pub", "crate", "self", "super", "struct", "enum",
        "impl", "const", "derive",
    ];

    let mut highlights = Vec::new();

    for word in hl_words {
        let mut offs = 0;
        while let Some(pos) = line[offs..].find(word) {
            let pos = pos + offs;
            let hl = Vec2::new(pos, pos + word.len());
            highlights.push((hl, ANSIColor::Green));
            offs = pos + word.len();
        }
    }

    log::debug!("done highlighting, buf: {:?}", highlights);

    highlights
}

impl Drop for App {
    fn drop(&mut self) {
        if let AppState::Running { prev_state } = self.state {
            let _ = Terminal::restore_state(&prev_state);
        }
    }
}
