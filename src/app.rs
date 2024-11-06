use std::{
    io::{stdout, Read, Write},
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
};

use crate::{terminal::Terminal, window::Window};

pub enum AppEvent {
    QuitEvent,
    KeyPressEvent(char),
}

pub struct Initialized {
    terminal_state: termios::Termios,
}

#[derive(Default)]
pub struct Uninitialized {}

pub struct App<State = Uninitialized> {
    state: State,
    window: Window,

    x: usize,
    y: usize,
}

impl App<Uninitialized> {
    pub fn new() -> Self {
        App {
            window: Window::new(),
            state: Uninitialized::default(),

            x: 0,
            y: 0,
        }
    }

    pub fn initialize(mut self) -> App<Initialized> {
        let (w, h) = Terminal::get_size().unwrap();
        // FIXME: This is a workaround for tmux
        let h = h - 1;
        self.window.resize(w as usize, h as usize);

        let terminal_state = Terminal::get_current_state().unwrap();
        Terminal::into_raw().unwrap();
        Terminal::clear_screen().unwrap();
        Terminal::set_position(self.x, self.y).unwrap();
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
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::scope(|s| {
            s.spawn(|| Self::handle_inputs(event_tx, rx));
            s.spawn(|| self.handle_events(event_rx, tx));
        });

        Terminal::restore_state(&self.state.terminal_state).unwrap();
    }

    fn handle_inputs(event_queue: Sender<AppEvent>, kill_signal: std::sync::mpsc::Receiver<bool>) {
        let mut reader = timeout_readwrite::TimeoutReader::new(
            std::io::stdin(),
            Some(std::time::Duration::from_millis(200)),
        );
        let mut buffer = [0u8; 1];

        loop {
            let res = reader.read(&mut buffer);
            if kill_signal.try_recv().is_ok() {
                break;
            }

            if res.is_err() {
                continue;
            }

            let event = match buffer[0] {
                b'q' => AppEvent::QuitEvent {},
                c if c.is_ascii() => AppEvent::KeyPressEvent(c as char),
                _ => continue,
            };

            event_queue.send(event).unwrap();
        }
    }

    fn handle_events(&mut self, event_queue: Receiver<AppEvent>, kill_signal: Sender<bool>) {
        loop {
            let event = event_queue.recv().unwrap();
            match event {
                AppEvent::QuitEvent => {
                    kill_signal.send(true).unwrap();
                    break;
                }
                AppEvent::KeyPressEvent(c) => match c {
                    'k' => {
                        self.y = self.y.saturating_sub(1);
                        Terminal::set_position(self.x, self.y).unwrap();
                        stdout().flush().unwrap();
                    }
                    'j' => {
                        self.y = self.y.saturating_add(1);
                        Terminal::set_position(self.x, self.y).unwrap();
                        stdout().flush().unwrap();
                    }
                    'h' => {
                        self.x = self.x.saturating_sub(1);
                        Terminal::set_position(self.x, self.y).unwrap();
                        stdout().flush().unwrap();
                    }
                    'l' => {
                        self.x = self.x.saturating_add(1);
                        Terminal::set_position(self.x, self.y).unwrap();
                        stdout().flush().unwrap();
                    }
                    _ => {
                        self.window.put_char(self.x, self.y, c);
                        self.window.render().unwrap();
                        self.x = self.x.saturating_add(1);
                        Terminal::set_position(self.x, self.y).unwrap();
                        stdout().flush().unwrap();
                    }
                },
            }
        }
    }
}
