mod action;
mod meta;

use action::{Action, InputMapper};
use meta::BufferMeta;

use std::{
    collections::VecDeque,
    fs::OpenOptions,
    io::{stdout, BufWriter, Write},
    path::PathBuf,
};

use edi::{
    draw::WindowBind,
    fs::Filetype,
    rect::Rect,
    string::highlight::get_highlights,
    terminal::{
        self,
        escaping::ANSIEscape,
        input::{self, Stream},
        window::Window,
    },
    vec2::Vec2,
};

use edi::buffer::Buffer;

use crate::cli::EdiCli;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Mode {
    Normal,
    Insert,
    Terminal,
}

#[derive(Debug)]
struct State {
    mode: Mode,
    mapper: InputMapper,
    buffers: VecDeque<(Buffer, BufferMeta)>,
}

impl State {
    /// Instantiates an empty `State` with nothing stored in buffers and mode set to `Normal`
    #[must_use]
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
            mapper: InputMapper::default(),
            buffers: VecDeque::new(),
        }
    }

    /// Opens a file with the given path, appending it's contents to the leftmost buffer
    pub fn open_file(
        &mut self,
        filepath: impl AsRef<std::path::Path>,
        buff_dimensions: Vec2<usize>,
    ) -> anyhow::Result<()> {
        let filepath = filepath.as_ref();
        let contents = std::fs::read_to_string(filepath)?;

        let buffer = Buffer::new(contents);
        let filetype = Filetype::from_ext(
            filepath
                .extension()
                .and_then(|v| v.to_str())
                .unwrap_or("unknown"),
        );

        let mut meta = BufferMeta::default()
            .with_filepath(Some(filepath.into()))
            .with_filetype(filetype)
            .with_size(buff_dimensions);

        meta.flush_options = meta
            .flush_options
            .with_highlights(get_highlights(&buffer.inner, &meta.filetype));

        self.buffers.push_back((buffer, meta));

        Ok(())
    }
}

fn handle_inputs(
    input_stream: &Stream,
    state: &mut State,
    render_window: &mut Window,
) -> anyhow::Result<()> {
    edi::debug!("handle_inputs: running");
    loop {
        let message = input_stream.recv()?;
        let input = match message {
            input::Message::Input(event) => event,
            input::Message::Error(e) => {
                edi::debug!("handle_inputs: received an error {:?}", e);
                continue;
            }
        };

        let Some(event) = state.mapper.map_input(&input, state.mode) else {
            edi::debug!("handle_inputs: no event for input {:?}", input);
            continue;
        };

        edi::debug!("handle_inputs: received event {:?}", event);

        match handle_event(event, state, render_window) {
            Ok(true) => break,
            Err(err) => return Err(err)?,
            _ => {}
        }
    }

    Ok(())
}

// TODO: Refactor this mess to map-based handler system

/// Handles a signle event, returning Ok(true), if the program should terminate
fn handle_event(
    event: Action,
    state: &mut State,
    render_window: &mut Window,
) -> anyhow::Result<bool> {
    match event {
        Action::SwitchMode(mode) => {
            state.mode = mode;
            if state.mode == Mode::Terminal {
                let size = terminal::get_size().unwrap_or(Vec2::new(10, 1));
                let mut buf = Buffer::new(String::from(":"));
                buf.cursor_offset = 1;
                state.buffers.push_front((
                    buf,
                    BufferMeta::default().with_size(Vec2::new(size.x as usize, 1)),
                ));
                redraw(state, render_window)?;
            }
        }
        Action::InsertChar(c) => {
            match state.buffers.front_mut() {
                Some((b, m)) => {
                    b.write(c);
                    m.flush_options.highlights = get_highlights(&b.inner, &m.filetype);

                    redraw(state, render_window)?;
                }
                None => {
                    edi::debug!("handle_event: no buffers to write to");
                }
            }
            render_window.render()?;
        }
        Action::DeleteChar => {
            match state.buffers.front_mut() {
                Some((b, m)) => {
                    b.delete();
                    m.flush_options.highlights = get_highlights(&b.inner, &m.filetype);
                    redraw(state, render_window)?;
                }
                None => {
                    edi::debug!("handle_event: no buffers to delete from");
                }
            }
            render_window.render()?;
        }
        Action::MoveCursor(dir, steps) => {
            match state.buffers.front_mut() {
                Some((b, _)) => {
                    b.move_cursor(dir, steps);
                    redraw(state, render_window)?;
                }
                None => {
                    edi::debug!("handle_event: no buffers to move cursor in");
                }
            }
            render_window.render()?;
        }
        Action::Quit => return Ok(true),
        Action::Submit => {
            // TODO: Add proper error handling
            let (cmd_buf, _) = state.buffers.pop_front().unwrap();
            redraw(state, render_window)?;
            let cmd: String = cmd_buf.inner.chars().collect();
            if cmd == ":q" {
                return handle_event(Action::Quit, state, render_window);
            }
            if cmd == ":wq" {
                let Some((b, meta)) = state.buffers.pop_front() else {
                    edi::fatal!("app::handle_event no buffer to write")
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
                        edi::debug!("handle_event: unable to create output file {e} {swap_name:?}");
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
                    edi::debug!("app::handle_event failed to rename file {e}");
                };

                return handle_event(Action::Quit, state, render_window);
            }
        }

        Action::MoveHalfScreen(dir) => {
            match state.buffers.front_mut() {
                Some((b, m)) => {
                    b.move_cursor(dir, m.size.y / 2);
                    redraw(state, render_window)?;
                }
                None => {
                    edi::debug!("handle_event: no buffers to move cursor in");
                }
            }
            render_window.render()?;
        }

        Action::MoveToLineStart => {
            match state.buffers.front_mut() {
                Some((b, _)) => {
                    b.move_to(
                        b.inner
                            .line_info(b.current_line())
                            .unwrap()
                            .character_offset,
                    );
                    redraw(state, render_window)?;
                }
                None => {
                    edi::debug!("handle_event: no buffers to move cursor in");
                }
            }
            render_window.render()?;
        }
    }

    Ok(false)
}

fn redraw(state: &mut State, draw_window: &mut Window) -> std::io::Result<()> {
    edi::debug!("app::redraw drawing {} buffers", state.buffers.len());

    let size = terminal::get_size()?;
    state.buffers.iter_mut().rev().for_each(|(b, m)| {
        m.normalize(b);
        let mut bound = Rect::new_in_origin(size.x as usize, size.y as usize).bind(draw_window);
        b.flush(&mut bound, &m.flush_options);
    });
    draw_window.render()
}

/// Runs the `edi` application, blocknig until receiving an error / close signal
pub fn run(args: EdiCli) -> anyhow::Result<()> {
    terminal::within_raw_mode(|| {
        let mut render_window = Window::new();
        let mut app_state = State::new();
        let input_stream = Stream::from_stdin();

        let _ = stdout().write(ANSIEscape::EnterAlternateScreen.to_str().as_bytes());

        let size = terminal::get_size()?.map(|v| v as usize);

        if let Some(filepath) = args.edit_file {
            app_state.open_file(filepath, size)?;
        }

        render_window.set_size(size);
        render_window.set_cursor(Vec2::new(0, 0));
        render_window.rerender()?;

        redraw(&mut app_state, &mut render_window)?;

        handle_inputs(&input_stream, &mut app_state, &mut render_window)?;

        let _ = stdout().write(ANSIEscape::ExitAlternateScreen.to_str().as_bytes());

        Ok(())
    })?
}
