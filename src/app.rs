use std::{
    io::{stdout, Read, Write},
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    escaping::ANSIColor,
    input::{self, InputEvent},
    terminal::Terminal,
    window::{Cell, Window},
};

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
        self.window.resize(w as usize, h as usize);

        self.window.debug_fill();

        let terminal_state = Terminal::get_current_state().unwrap();
        Terminal::into_raw().unwrap();
        self.window.rerender().unwrap();
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
        std::thread::scope(|s| {
            let (r_events, r_errors, t_kill) = input::to_event_stream(std::io::stdin());
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
                        self.window
                            .put_cell(self.x, self.y, Cell::new(c, ANSIColor::Green));
                        self.window.render().unwrap();
                        self.x = self.x.saturating_add(1);
                        Terminal::set_position(self.x, self.y).unwrap();
                        stdout().flush().unwrap();
                    }
                },
                v => {
                    dbg!(v);
                }
            }
        }
    }
}
