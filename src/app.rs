use std::{
    io::{stdout, Write},
    sync::mpsc::{Receiver, Sender},
};

use crate::{
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

    x: usize,
    y: usize,
}

impl App<Uninitialized> {
    #[must_use]
    pub const fn new() -> Self {
        App {
            window: Window::new(),
            state: Uninitialized,

            x: 0,
            y: 0,
        }
    }

    pub fn initialize(mut self) -> App<Initialized> {
        let (w, h) = Terminal::get_size().unwrap();
        self.window.resize(w as usize, h as usize);

        let terminal_state = Terminal::get_current_state().unwrap();
        Terminal::into_raw().unwrap();

        self.window.set_cursor(self.x, self.y);
        self.window.rerender().unwrap();
        Terminal::flush().unwrap();

        App {
            state: Initialized { terminal_state },
            window: self.window,

            x: self.x,
            y: self.y,
        }
    }
}

impl App<Initialized> {
    pub fn run(&mut self) {
        std::thread::scope(|s| {
            let (r_events, _, t_kill) = input::to_event_stream(std::io::stdin());
            s.spawn(|| self.handle_events(r_events, t_kill));
        });

        Terminal::restore_state(&self.state.terminal_state).unwrap();
    }

    fn handle_events(&mut self, event_queue: Receiver<InputEvent>, kill_signal: Sender<()>) {
        loop {
            let event = event_queue.recv().unwrap();
            match event {
                InputEvent::Keypress('q') => {
                    kill_signal.send(()).unwrap();
                    break;
                }
                InputEvent::Keypress(c) => match c {
                    'k' => {
                        self.y = self.y.saturating_sub(1);
                        self.window.set_cursor(self.x, self.y);
                        self.window.render_cursor().unwrap();
                    }
                    'j' => {
                        self.y = self.y.saturating_add(1);
                        self.window.set_cursor(self.x, self.y);
                        self.window.render_cursor().unwrap();
                    }
                    'h' => {
                        self.x = self.x.saturating_sub(1);
                        self.window.set_cursor(self.x, self.y);
                        self.window.render_cursor().unwrap();
                    }
                    'l' => {
                        self.x = self.x.saturating_add(1);
                        self.window.set_cursor(self.x, self.y);
                        self.window.render_cursor().unwrap();
                    }
                    v => {
                        eprintln!("{}", v as u32);
                        self.window
                            .put_cell(self.x, self.y, Cell::new(c, ANSIColor::Green));
                        self.x = self.x.saturating_add(1);
                        self.window.set_cursor(self.x, self.y);
                        self.window.render().unwrap();
                    }
                },
                v => {
                    eprintln!("{:?}", v);
                }
            }
        }
    }
}
