mod event;
mod meta;

use event::Event;
use meta::BufferMeta;

use std::{
    collections::{HashMap, VecDeque},
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
};

use edi::{
    rope::Rope,
    terminal::{
        self,
        escaping::ANSIColor,
        input::{self, Stream},
        window::Window,
    },
    vec2::Vec2,
};

use crate::{
    buffer::{Buffer, Highlight},
    cli::EdiCli,
    log,
};

#[derive(Debug, PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
    Terminal,
}

#[derive(Debug)]
struct State {
    mode: Mode,
    buffers: VecDeque<(Buffer, BufferMeta)>,
}

impl State {
    /// Instantiates an empty `State` with nothing stored in buffers and mode set to `Normal`
    #[must_use]
    pub const fn new() -> Self {
        Self {
            mode: Mode::Normal,
            buffers: VecDeque::new(),
        }
    }

    /// Opens a file with the given path, appending it's contents to the leftmost buffer
    pub fn open_file(
        &mut self,
        filepath: impl AsRef<std::path::Path>,
        buff_dimensions: Vec2<usize>,
    ) -> anyhow::Result<()> {
        let contents = std::fs::read_to_string(&filepath)?;

        let buffer = Buffer::new(contents, buff_dimensions);
        let mut meta = BufferMeta::default().with_filepath(Some(filepath.as_ref().into()));

        highlight_naive(&buffer.inner, &mut meta.flush_options.highlights);
        self.buffers.push_back((buffer, meta));

        Ok(())
    }
}

fn handle_inputs(
    input_stream: &Stream,
    state: &mut State,
    render_window: &mut Window,
) -> anyhow::Result<()> {
    loop {
        let message = input_stream.recv()?;
        let input = match message {
            input::Message::Input(event) => event,
            input::Message::Error(e) => {
                log::debug!("handle_inputs: received an error {:?}", e);
                continue;
            }
        };

        let Some(event) = event::map_input(&input, &state.mode) else {
            log::debug!("handle_inputs: no event for input {:?}", input);
            continue;
        };

        log::debug!("handle_inputs: received event {:?}", event);

        match handle_event(event, state, render_window) {
            Ok(true) => break,
            Err(err) => return Err(err)?,
            _ => {}
        }
    }

    Ok(())
}

// TODO: Refactor this mess into map-based handler system

/// Handles a signle event, returning Ok(true), if the program should terminate
fn handle_event(
    event: Event,
    state: &mut State,
    render_window: &mut Window,
) -> anyhow::Result<bool> {
    match event {
        Event::SwitchMode(mode) => {
            state.mode = mode;
            if state.mode == Mode::Terminal {
                let size = terminal::get_size().unwrap_or(Vec2::new(10, 1));
                let mut buf = Buffer::new(String::from(":"), Vec2::new(size.x as usize, 1));
                buf.cursor_offset = 1;
                state.buffers.push_front((buf, BufferMeta::default()));
                redraw(state, render_window)?;
            }
        }
        Event::InsertChar(c) => {
            match state.buffers.front_mut() {
                Some((b, m)) => {
                    b.write(c);
                    highlight_naive(&b.inner, &mut m.flush_options.highlights);
                    redraw(state, render_window)?;
                }
                None => {
                    log::debug!("handle_event: no buffers to write to");
                }
            }
            render_window.render()?;
        }
        Event::DeleteChar => {
            match state.buffers.front_mut() {
                Some((b, _)) => {
                    b.delete();
                    redraw(state, render_window)?;
                }
                None => {
                    log::debug!("handle_event: no buffers to delete from");
                }
            }
            render_window.render()?;
        }
        Event::MoveCursor(dir, steps) => {
            match state.buffers.front_mut() {
                Some((b, _)) => {
                    b.move_cursor(dir, steps);
                    redraw(state, render_window)?;
                }
                None => {
                    log::debug!("handle_event: no buffers to move cursor in");
                }
            }
            render_window.render()?;
        }
        Event::Quit => return Ok(true),
        Event::Submit => {
            // TODO: Add proper error handling
            let (cmd_buf, _) = state.buffers.pop_front().unwrap();
            redraw(state, render_window)?;
            let cmd: String = cmd_buf.inner.chars().collect();
            if cmd == ":q" {
                return handle_event(Event::Quit, state, render_window);
            }
            if cmd == ":wq" {
                let Some((b, meta)) = state.buffers.pop_front() else {
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
                        log::debug!("handle_event: unable to create output file {e} {swap_name:?}");
                        return Ok(true);
                    }
                };

                let mut w = BufWriter::new(file);
                b.inner.chars().for_each(|c| {
                    let _ = w.write(&c.to_string().bytes().collect::<Vec<_>>());
                });

                if let Err(e) =
                    std::fs::rename(swap_name, meta.filepath.unwrap_or(PathBuf::from("out.txt")))
                {
                    log::debug!("app::handle_event failed to rename file {e}");
                };

                return handle_event(Event::Quit, state, render_window);
            }
        }

        Event::MoveHalfScreen(dir) => {
            match state.buffers.front_mut() {
                Some((b, _)) => {
                    b.move_cursor(dir, b.size.y / 2);
                    redraw(state, render_window)?;
                }
                None => {
                    log::debug!("handle_event: no buffers to move cursor in");
                }
            }
            render_window.render()?;
        }
    }

    Ok(false)
}

fn redraw(state: &State, draw_window: &mut Window) -> std::io::Result<()> {
    log::debug!("app::redraw drawing {} buffers", state.buffers.len());
    state.buffers.iter().rev().for_each(|(b, m)| {
        b.flush(draw_window, &m.flush_options);
    });
    draw_window.render()
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
        .filter(|(_, hls)| !hls.is_empty())
        .for_each(|(nr, hls)| {
            hm.insert(nr, hls);
        });
}

#[allow(unused)]
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

#[allow(unused)]
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

/// Runs the `edi` application, blocknig until receiving an error / close signal
pub fn run(args: EdiCli) -> anyhow::Result<()> {
    let initial_state = terminal::get_current_state()?;

    // Make sure the initial state is restored even in case of application error
    (|| -> anyhow::Result<()> {
        terminal::into_raw()?;

        let mut render_window = Window::new();
        let mut app_state = State::new();
        let input_stream = Stream::from_stdin();

        let size = terminal::get_size()?.map(|v| v as usize);

        if let Some(filepath) = args.edit_file {
            app_state.open_file(filepath, size)?;
        }

        render_window.set_size(size);
        render_window.set_cursor(Vec2::new(0, 0));
        render_window.rerender()?;

        redraw(&app_state, &mut render_window)?;

        handle_inputs(&input_stream, &mut app_state, &mut render_window)
    })()?;

    terminal::restore_state(&initial_state)?;

    Ok(())
}
