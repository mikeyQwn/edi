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

pub struct Initialized {
    terminal_state: termios::Termios,
}

#[derive(Default)]
pub struct Uninitialized;

pub struct App<State = Uninitialized> {
    state: State,
    window: Window,
    buffers: Vec<Buffer>,

    cursor_pos: Vec2<usize>,
}

impl App<Uninitialized> {
    #[must_use]
    pub const fn new() -> Self {
        App {
            window: Window::new(),
            state: Uninitialized,
            buffers: Vec::new(),

            cursor_pos: Vec2::new(0, 0),
        }
    }

    pub fn initialize(mut self, args: EdiCli) -> Result<App<Initialized>, std::io::Error> {
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

        let terminal_state = Terminal::get_current_state()?;
        Terminal::into_raw()?;

        self.window.set_cursor(self.cursor_pos);
        self.window.rerender()?;
        Terminal::flush()?;

        Ok(App {
            window: self.window,
            state: Initialized { terminal_state },
            buffers: self.buffers,

            cursor_pos: self.cursor_pos,
        })
    }
}

impl App<Initialized> {
    pub fn run(&mut self) -> Result<(), std::io::Error> {
        let input_stream = InputStream::from_read(std::io::stdin());
        self.redraw();
        self.handle_inputs(input_stream)?;

        Terminal::restore_state(&self.state.terminal_state)?;

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

            match input {
                Input::Keypress('q') => {
                    break;
                }
                Input::Keypress(c) => match c {
                    'k' => {
                        self.cursor_pos.y = self.cursor_pos.y.saturating_sub(1);
                    }
                    'j' => {
                        self.cursor_pos.y = self.cursor_pos.y.saturating_add(1);
                    }
                    'h' => {
                        self.cursor_pos.x = self.cursor_pos.x.saturating_sub(1);
                    }
                    'l' => {
                        self.cursor_pos.x = self.cursor_pos.x.saturating_add(1);
                    }
                    v => {
                        log::debug!(
                            "handle_inputs: received keypress that is not handled {:?}",
                            v
                        );
                        self.window
                            .put_cell(self.cursor_pos, Cell::new(c, ANSIColor::Green));
                        self.cursor_pos.x = self.cursor_pos.x.saturating_add(1);
                        self.window.set_cursor(self.cursor_pos);
                        self.window.render()?;
                    }
                },
                ref v => {
                    log::debug!("handle_inputs: received input that is not handled {:?}", v);
                }
            }
            if let Input::Keypress('h' | 'k' | 'j' | 'l') = input {
                self.window.set_cursor(self.cursor_pos);
                self.window.render_cursor()?;
            }
        }

        Ok(())
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
