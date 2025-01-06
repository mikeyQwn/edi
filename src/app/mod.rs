mod event;

use std::{
    collections::VecDeque,
    fs::OpenOptions,
    io::{BufWriter, Write},
};

use event::Event;

use crate::{
    buffer::{Buffer, FlushOptions},
    cli::EdiCli,
    escaping::ANSIColor,
    input::{self, Stream},
    log,
    terminal::Terminal,
    vec2::Vec2,
    window::Window,
};

#[derive(Debug)]
enum AppState {
    Stopped,
    Running { prev_state: termios::Termios },
}

#[derive(Debug, PartialEq, Eq)]
enum AppMode {
    Normal,
    Insert,
    Terminal,
}

pub struct App {
    state: AppState,

    mode: AppMode,
    window: Window,
    buffers: VecDeque<Buffer>,
}

impl App {
    #[must_use]
    pub const fn new() -> Self {
        App {
            mode: AppMode::Normal,
            state: AppState::Stopped,
            window: Window::new(),
            buffers: VecDeque::new(),
        }
    }

    pub fn setup(&mut self, args: EdiCli) -> Result<(), std::io::Error> {
        let size = Terminal::get_size()?;
        if let Some(f) = args.edit_file {
            let contents = std::fs::read_to_string(f)?;
            self.buffers.push_back(Buffer::new(
                contents,
                Vec2::new(size.0 as usize, size.1 as usize),
            ));
        }

        let (w, h) = Terminal::get_size()?;
        self.window.resize(w as usize, h as usize);

        Terminal::into_raw()?;

        self.window.set_cursor(Vec2::new(0, 0));
        self.window.rerender()?;
        Terminal::flush()?;

        Ok(())
    }

    pub fn run(&mut self, args: EdiCli) -> Result<(), std::io::Error> {
        self.state = AppState::Running {
            prev_state: Terminal::get_current_state()?,
        };
        self.setup(args)?;

        let input_stream = Stream::from_read(std::io::stdin());
        self.redraw();
        self.handle_inputs(&input_stream);

        Terminal::restore_state(&match self.state {
            AppState::Running { prev_state } => prev_state,
            AppState::Stopped => panic!("App::run: state is stopped"),
        })?;

        self.state = AppState::Stopped;

        Ok(())
    }

    fn handle_inputs(&mut self, input_stream: &Stream) {
        loop {
            let message = input_stream.recv().unwrap();
            let input = match message {
                input::Message::Input(event) => event,
                input::Message::Error(e) => {
                    log::debug!("handle_inputs: received an error {:?}", e);
                    continue;
                }
            };

            let Some(event) = event::map_input(&input, &self.mode) else {
                log::debug!("handle_inputs: no event for input {:?}", input);
                continue;
            };

            log::debug!("handle_inputs: received event {:?}", event);

            if self.handle_event(event) {
                break;
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> bool {
        match event {
            Event::SwitchMode(mode) => {
                self.mode = mode;
                if self.mode == AppMode::Terminal {
                    let size = Terminal::get_size().unwrap_or((10, 0));
                    let mut buf = Buffer::new(String::from(":"), Vec2::new(size.0 as usize, 1));
                    buf.cursor_offset = 1;
                    self.buffers.push_front(buf);
                    self.redraw();
                }
            }
            Event::InsertChar(c) => {
                match self.buffers.front_mut() {
                    Some(b) => {
                        b.write(c);
                        self.redraw();
                    }
                    None => {
                        log::debug!("handle_event: no buffers to write to");
                    }
                }
                let _ = self.window.render();
            }
            Event::DeleteChar => {
                match self.buffers.front_mut() {
                    Some(b) => {
                        b.delete();
                        self.redraw();
                    }
                    None => {
                        log::debug!("handle_event: no buffers to delete from");
                    }
                }
                let _ = self.window.render();
            }
            Event::MoveCursor(dir) => {
                match self.buffers.front_mut() {
                    Some(b) => {
                        b.move_cursor(1, dir);
                        self.redraw();
                    }
                    None => {
                        log::debug!("handle_event: no buffers to move cursor in");
                    }
                }
                let _ = self.window.render();
            }
            Event::Quit => return true,
            Event::Submit => {
                // TODO: Add proper error handling
                let cmd_buf = self.buffers.pop_front().unwrap();
                self.redraw();
                let cmd: String = cmd_buf.inner.chars().collect();
                if cmd == ":q" {
                    return self.handle_event(Event::Quit);
                }
                if cmd == ":wq" {
                    let Ok(file) = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open("out.swap")
                    else {
                        log::debug!("handle_event: unable to create output file");
                        return true;
                    };

                    let mut w = BufWriter::new(file);
                    if let Some(b) = self.buffers.front() {
                        b.inner.chars().for_each(|c| {
                            let _ = w.write(&c.to_string().bytes().collect::<Vec<_>>());
                        });
                    };

                    return self.handle_event(Event::Quit);
                }
            }
        }

        false
    }

    fn redraw(&mut self) {
        log::debug!("app::redraw drawing {} buffers", self.buffers.len());
        self.buffers.iter().rev().for_each(|b| {
            let opts = FlushOptions::default().with_highlights(highlight_naive(&b.text()));
            b.flush(&mut self.window, &opts);
        });
        let _ = self.window.render();
    }
}
fn highlight_naive(text: &str) -> Vec<(Vec2<usize>, ANSIColor)> {
    let hl_words = vec![
        ("as", ANSIColor::Magenta),
        ("break", ANSIColor::Magenta),
        ("const", ANSIColor::Magenta),
        ("continue", ANSIColor::Magenta),
        ("crate", ANSIColor::Magenta),
        ("else", ANSIColor::Magenta),
        ("enum", ANSIColor::Magenta),
        ("extern", ANSIColor::Magenta),
        ("false", ANSIColor::Magenta),
        ("fn", ANSIColor::Magenta),
        ("for", ANSIColor::Magenta),
        ("if", ANSIColor::Magenta),
        ("impl", ANSIColor::Magenta),
        ("in", ANSIColor::Magenta),
        ("let", ANSIColor::Magenta),
        ("loop", ANSIColor::Magenta),
        ("match", ANSIColor::Magenta),
        ("mod", ANSIColor::Magenta),
        ("move", ANSIColor::Magenta),
        ("mut", ANSIColor::Magenta),
        ("pub", ANSIColor::Magenta),
        ("ref", ANSIColor::Magenta),
        ("return", ANSIColor::Magenta),
        ("self", ANSIColor::Magenta),
        ("Self", ANSIColor::Magenta),
        ("static", ANSIColor::Magenta),
        ("struct", ANSIColor::Magenta),
        ("super", ANSIColor::Magenta),
        ("trait", ANSIColor::Magenta),
        ("true", ANSIColor::Magenta),
        ("type", ANSIColor::Magenta),
        ("unsafe", ANSIColor::Magenta),
        ("use", ANSIColor::Magenta),
        ("where", ANSIColor::Magenta),
        ("while", ANSIColor::Magenta),
        ("async", ANSIColor::Magenta),
        ("await", ANSIColor::Magenta),
        ("dyn", ANSIColor::Magenta),
    ];

    let mut highlights = Vec::new();

    for (word, color) in hl_words {
        let mut offs = 0;
        while let Some(pos) = text[offs..].find(word) {
            let pos = pos + offs;
            offs = pos + word.len();
            if pos > 0 && text.chars().nth(pos - 1).map(|c| !c.is_whitespace()) == Some(true) {
                continue;
            }
            if text
                .chars()
                .nth(pos + word.len())
                .map(|c| !c.is_whitespace())
                == Some(true)
            {
                continue;
            }
            let hl = Vec2::new(pos, pos + word.len());
            highlights.push((hl, color));
        }
    }

    highlights
}

impl Drop for App {
    fn drop(&mut self) {
        if let AppState::Running { prev_state } = self.state {
            let _ = Terminal::restore_state(&prev_state);
        }
    }
}
