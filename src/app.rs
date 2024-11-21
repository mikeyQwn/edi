use crate::{
    buffer::Buffer,
    cli::EdiCli,
    escaping::ANSIColor,
    input::{self, Input, InputStream},
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
        if let Some(f) = args.edit_file {
            let contents = std::fs::read_to_string(f)?;
            self.buffers.push(Buffer::new(contents, Vec2::new(50, 20)));
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
                    eprintln!("{:?}", e);
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
                        eprintln!("{}", v as u32);
                        self.window
                            .put_cell(self.cursor_pos, Cell::new(c, ANSIColor::Green));
                        self.cursor_pos.x = self.cursor_pos.x.saturating_add(1);
                        self.window.set_cursor(self.cursor_pos);
                        self.window.render()?;
                    }
                },
                ref v => {
                    eprintln!("{:?}", v);
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
        self.buffers
            .iter()
            .for_each(|b| b.flush(&mut self.window, true));
        let _ = self.window.render();
    }
}
