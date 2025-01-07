mod event;
mod meta;

use std::{
    collections::{HashMap, VecDeque},
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
};

use event::Event;
use meta::BufferMeta;

use crate::{
    buffer::{Buffer, Highlight},
    cli::EdiCli,
    escaping::ANSIColor,
    input::{self, Stream},
    log,
    rope::Rope,
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
    buffers: VecDeque<(Buffer, BufferMeta)>,
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
            let contents = std::fs::read_to_string(&f)?;
            let buffer = Buffer::new(contents, Vec2::new(size.0 as usize, size.1 as usize));
            let mut meta = BufferMeta::default().with_filepath(Some(f));
            highlight_naive(&buffer.inner, &mut meta.flush_options.highlights);
            self.buffers.push_back((buffer, meta));
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
                    self.buffers.push_front((buf, BufferMeta::default()));
                    self.redraw();
                }
            }
            Event::InsertChar(c) => {
                match self.buffers.front_mut() {
                    Some((b, m)) => {
                        b.write(c);
                        highlight_naive(&b.inner, &mut m.flush_options.highlights);
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
                    Some((b, _)) => {
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
                    Some((b, _)) => {
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
                let (cmd_buf, _) = self.buffers.pop_front().unwrap();
                self.redraw();
                let cmd: String = cmd_buf.inner.chars().collect();
                if cmd == ":q" {
                    return self.handle_event(Event::Quit);
                }
                if cmd == ":wq" {
                    let Some((b, meta)) = self.buffers.pop_front() else {
                        log::fatal!("app::handle_event no buffer to write")
                    };

                    let swap_name = meta
                        .filepath
                        .as_ref()
                        .map_or(PathBuf::from("out.swp"), |fp| {
                            let mut fp = fp.clone();
                            fp.set_extension(".swp");
                            fp
                        });

                    let file = match OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(&swap_name)
                    {
                        Ok(f) => f,
                        Err(e) => {
                            log::debug!(
                                "handle_event: unable to create output file {e} {swap_name:?}"
                            );
                            return true;
                        }
                    };

                    let mut w = BufWriter::new(file);
                    b.inner.chars().for_each(|c| {
                        let _ = w.write(&c.to_string().bytes().collect::<Vec<_>>());
                    });

                    if let Err(e) = std::fs::rename(
                        swap_name,
                        meta.filepath.unwrap_or(PathBuf::from("out.txt")),
                    ) {
                        log::debug!("app::handle_event failed to rename file {e}");
                    };

                    return self.handle_event(Event::Quit);
                }
            }
        }

        false
    }

    fn redraw(&mut self) {
        log::debug!("app::redraw drawing {} buffers", self.buffers.len());
        self.buffers.iter().rev().for_each(|(b, m)| {
            b.flush(&mut self.window, &m.flush_options);
        });
        let _ = self.window.render();
    }
}

fn highlight_naive(rope: &Rope, hm: &mut HashMap<usize, Vec<Highlight>>) {
    hm.clear();
    rope.lines()
        .map(|line| {
            (
                line.line_number,
                get_line_highlights(&line.contents, &C_KEYWORDS),
            )
        })
        .for_each(|(nr, hls)| {
            hm.insert(nr, hls);
        });
}

const RUST_KEYWORDS: [(&str, ANSIColor); 38] = [
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

const C_KEYWORDS: [(&str, ANSIColor); 32] = [
    ("auto", ANSIColor::Magenta),
    ("break", ANSIColor::Magenta),
    ("case", ANSIColor::Magenta),
    ("char", ANSIColor::Magenta),
    ("const", ANSIColor::Magenta),
    ("continue", ANSIColor::Magenta),
    ("default", ANSIColor::Magenta),
    ("do", ANSIColor::Magenta),
    ("double", ANSIColor::Magenta),
    ("else", ANSIColor::Magenta),
    ("enum", ANSIColor::Magenta),
    ("extern", ANSIColor::Magenta),
    ("float", ANSIColor::Magenta),
    ("for", ANSIColor::Magenta),
    ("if", ANSIColor::Magenta),
    ("int", ANSIColor::Magenta),
    ("long", ANSIColor::Magenta),
    ("register", ANSIColor::Magenta),
    ("return", ANSIColor::Magenta),
    ("short", ANSIColor::Magenta),
    ("signed", ANSIColor::Magenta),
    ("sizeof", ANSIColor::Magenta),
    ("static", ANSIColor::Magenta),
    ("struct", ANSIColor::Magenta),
    ("switch", ANSIColor::Magenta),
    ("typedef", ANSIColor::Magenta),
    ("union", ANSIColor::Magenta),
    ("unsigned", ANSIColor::Magenta),
    ("void", ANSIColor::Magenta),
    ("goto", ANSIColor::Magenta),
    ("volatile", ANSIColor::Magenta),
    ("while", ANSIColor::Magenta),
];

fn get_line_highlights(line: &str, keywords: &[(&str, ANSIColor)]) -> Vec<Highlight> {
    let mut line_highlights = Vec::new();
    for &(word, color) in keywords {
        line_highlights.extend(
            line.match_indices(word)
                .map(|(idx, _)| (idx, idx + word.len()))
                .filter(|&(start, end)| {
                    let starts_with_space = start == 0
                        || line
                            .chars()
                            .nth(start - 1)
                            .filter(|&c| !c.is_whitespace())
                            .is_none();

                    let ends_with_space = line
                        .chars()
                        .nth(end)
                        .filter(|&c| !c.is_whitespace())
                        .is_none();

                    starts_with_space && ends_with_space
                })
                .map(|(start, end)| (Vec2::new(start, end), color)),
        );
    }

    line_highlights
}

impl Drop for App {
    fn drop(&mut self) {
        if let AppState::Running { prev_state } = self.state {
            let _ = Terminal::restore_state(&prev_state);
        }
    }
}
