use std::sync::mpsc::Receiver;

use crate::{
    buffer::Buffer,
    cli::EdiCli,
    escaping::ANSIColor,
    input::{self, InputEvent},
    terminal::Terminal,
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

    x: usize,
    y: usize,
}

impl App<Uninitialized> {
    #[must_use]
    pub const fn new() -> Self {
        App {
            window: Window::new(),
            state: Uninitialized,
            buffers: Vec::new(),

            x: 0,
            y: 0,
        }
    }

    pub fn initialize(mut self, args: EdiCli) -> Result<App<Initialized>, std::io::Error> {
        if let Some(f) = args.edit_file {
            let contents = std::fs::read_to_string(f)?;
            self.buffers.push(Buffer::new(contents, 50, 10));
        }

        let (w, h) = Terminal::get_size()?;
        self.window.resize(w as usize, h as usize);

        let terminal_state = Terminal::get_current_state()?;
        Terminal::into_raw()?;

        self.window.set_cursor(self.x, self.y);
        self.window.rerender()?;
        Terminal::flush()?;

        Ok(App {
            window: self.window,
            state: Initialized { terminal_state },
            buffers: self.buffers,

            x: self.x,
            y: self.y,
        })
    }
}

impl App<Initialized> {
    pub fn run(&mut self) -> Result<(), std::io::Error> {
        let (r_events, _, t_kill) = input::to_event_stream(std::io::stdin());
        self.redraw();
        self.handle_events(r_events)?;
        t_kill.send(()).unwrap();

        Terminal::restore_state(&self.state.terminal_state)?;
        Ok(())
    }

    fn handle_events(&mut self, event_queue: Receiver<InputEvent>) -> Result<(), std::io::Error> {
        loop {
            let event = event_queue.recv().unwrap();
            match event {
                InputEvent::Keypress('q') => {
                    break;
                }
                InputEvent::Keypress(c) => match c {
                    'k' => {
                        self.y = self.y.saturating_sub(1);
                        self.window.set_cursor(self.x, self.y);
                        self.window.render_cursor()?;
                    }
                    'j' => {
                        self.y = self.y.saturating_add(1);
                        self.window.set_cursor(self.x, self.y);
                        self.window.render_cursor()?;
                    }
                    'h' => {
                        self.x = self.x.saturating_sub(1);
                        self.window.set_cursor(self.x, self.y);
                        self.window.render_cursor()?;
                    }
                    'l' => {
                        self.x = self.x.saturating_add(1);
                        self.window.set_cursor(self.x, self.y);
                        self.window.render_cursor()?;
                    }
                    v => {
                        eprintln!("{}", v as u32);
                        self.window
                            .put_cell(self.x, self.y, Cell::new(c, ANSIColor::Green));
                        self.x = self.x.saturating_add(1);
                        self.window.set_cursor(self.x, self.y);
                        self.window.render()?;
                    }
                },
                v => {
                    eprintln!("{:?}", v);
                }
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
