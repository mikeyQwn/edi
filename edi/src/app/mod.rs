mod action;
mod meta;

use action::{Action, InputMapper, MoveAction};
use edi_frame::rect::Rect;
use edi_lib::{fs::filetype::Filetype, vec2::Vec2};
use edi_term::{coord::Coord, escaping::ANSIEscape, window::Window};
use meta::BufferMeta;

use std::{
    collections::VecDeque,
    fs::OpenOptions,
    io::{stdout, BufWriter, Write},
    path::PathBuf,
};

use edi::{
    buffer::Buffer,
    draw::{Surface, WindowBind},
    string::highlight::get_highlights,
};

use crate::{
    cli::EdiCli,
    event::{self, handlers, sources, EventManager},
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Mode {
    Normal,
    Insert,
    Terminal,
}

pub struct AppState {
    pub state: State,
    pub window: Window,
}

#[derive(Debug)]
pub struct State {
    pub mode: Mode,
    pub mapper: InputMapper,
    pub buffers: VecDeque<(Buffer, BufferMeta)>,
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

        let buffer = Buffer::new(&contents);
        let filetype = Filetype::from(filepath);

        let mut meta = BufferMeta::default()
            .with_filepath(Some(filepath.into()))
            .with_filetype(filetype)
            .with_size(buff_dimensions);

        meta.flush_options = meta
            .flush_options
            .with_highlights(get_highlights(&buffer.inner, &meta.filetype))
            .with_line_numbers(true);

        self.buffers.push_back((buffer, meta));

        Ok(())
    }
}

/// Handles a signle event, returning Ok(true), if the program should terminate
#[allow(
    clippy::too_many_lines,
    clippy::unnecessary_wraps,
    clippy::cognitive_complexity
)]
pub fn handle_action(
    event: Action,
    state: &mut State,
    sender: &event::Sender,
) -> anyhow::Result<()> {
    let _span = edi_lib::span!("handle_action");

    match event {
        Action::SwitchMode(mode) => {
            if state.mode == Mode::Terminal {
                let _ = state.buffers.pop_front();
                let _ = sender.send_redraw();
            }
            state.mode = mode;
            if state.mode == Mode::Terminal {
                let size = edi_term::get_size()
                    .map(Vec2::from_dims)
                    .unwrap_or(Vec2::new(10, 1));
                let mut buf = Buffer::new(":");
                buf.cursor_offset = 1;
                state.buffers.push_front((
                    buf,
                    BufferMeta::default().with_size(Vec2::new(size.x as usize, 1)),
                ));
                let _ = sender.send_redraw();
            }
        }
        Action::InsertChar(c) => {
            match state.buffers.front_mut() {
                Some((b, m)) => {
                    let is_empty = b.inner.is_empty();
                    b.write(c);
                    // Hack to always add a newline at the end of the file
                    if is_empty {
                        b.write('\n');
                        b.cursor_offset -= 1;
                    }
                    m.flush_options.highlights = get_highlights(&b.inner, &m.filetype);

                    let _ = sender.send_redraw();
                }
                None => {
                    edi_lib::debug!("no buffers to write to");
                }
            }
            let _ = sender.send_redraw();
        }
        Action::DeleteChar => {
            match state.buffers.front_mut() {
                Some((b, m)) => {
                    b.delete();
                    m.flush_options.highlights = get_highlights(&b.inner, &m.filetype);
                    sender.send_redraw();
                }
                None => {
                    edi_lib::debug!("no buffers to delete from");
                }
            }
            sender.send_redraw();
        }
        Action::Submit => {
            // TODO: Add proper error handling
            let (cmd_buf, _) = state.buffers.pop_front().unwrap();
            let _ = sender.send_redraw();
            let cmd: String = cmd_buf.inner.chars().collect();
            if cmd == ":q" {
                let _ = sender.send_quit();
                return Ok(());
            }
            if cmd == ":wq" {
                let Some((b, meta)) = state.buffers.pop_front() else {
                    edi_lib::fatal!("no buffer to write")
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
                        edi_lib::debug!("unable to create output file {e} {swap_name:?}");
                        let _ = sender.send_quit();
                        return Ok(());
                    }
                };

                let mut w = BufWriter::new(file);
                b.inner.lines().for_each(|line| {
                    let Err(err) = w
                        .write_all(line.contents.as_bytes())
                        .and_then(|()| w.write_all(b"\n"))
                    else {
                        return;
                    };
                    edi_lib::fatal!("unable to write line contents: {:?}", err);
                });

                if let Err(e) =
                    std::fs::rename(swap_name, meta.filepath.unwrap_or(PathBuf::from("out.txt")))
                {
                    edi_lib::debug!("app::handle_event failed to rename file {e}");
                }

                let _ = sender.send_quit();
            }
        }

        Action::Move { action, repeat } => {
            match state.buffers.front_mut() {
                Some((buffer, meta)) => {
                    handle_move(buffer, meta, action, repeat);
                    sender.send_redraw();
                }
                None => {
                    edi_lib::debug!("handle_event: no buffers to move cursor in");
                }
            }
            let _ = sender.send_redraw();
        }
        Action::Undo => {
            match state.buffers.front_mut() {
                Some((buffer, meta)) => {
                    edi_lib::debug!("undoing last action");
                    buffer.undo();
                    meta.flush_options.highlights = get_highlights(&buffer.inner, &meta.filetype);
                    let _ = sender.send_redraw();
                }
                None => {
                    edi_lib::debug!("handle_event: no buffers to undo in");
                }
            }
            let _ = sender.send_redraw();
        }
        Action::Redo => {
            match state.buffers.front_mut() {
                Some((buffer, meta)) => {
                    buffer.redo();
                    meta.flush_options.highlights = get_highlights(&buffer.inner, &meta.filetype);
                    let _ = sender.send_redraw();
                }
                None => {
                    edi_lib::debug!("handle_event: no buffers to undo in");
                }
            }
            let _ = sender.send_redraw();
        }
    }

    Ok(())
}

fn handle_move(buffer: &mut Buffer, meta: &mut BufferMeta, action: MoveAction, repeat: usize) {
    match action {
        MoveAction::Regular(direction) => {
            buffer.move_cursor(direction.into(), repeat);
        }
        MoveAction::InLine(line_position) => {
            buffer.move_in_line(line_position);
        }
        MoveAction::HalfScreen(direction) => {
            buffer.move_cursor(direction.into(), meta.size.y / 2);
        }
        MoveAction::Global(global_position) => buffer.move_global(global_position),
    }
}

pub fn redraw(state: &mut State, draw_window: &mut Window) -> std::io::Result<()> {
    let _span = edi_lib::span!("redraw");
    edi_lib::debug!("drawing {} buffers", state.buffers.len());

    draw_window.clear();
    state.buffers.iter_mut().rev().for_each(|(b, m)| {
        m.normalize(b);
        let mut bound = Rect::new_in_origin(m.size.x, m.size.y).bind(draw_window);
        bound.clear();
        b.flush(&mut bound, &m.flush_options);
    });
    draw_window.render()
}

/// Runs the `edi` application, blocknig until receiving an error / close signal
pub fn run(args: EdiCli) -> anyhow::Result<()> {
    let mut event_manager = EventManager::new();
    event_manager.attach_source(sources::input_source);

    edi_term::within_raw_mode(|| {
        let mut window = Window::new();
        let mut state = State::new();
        let _ = stdout().write(ANSIEscape::EnterAlternateScreen.to_str().as_bytes());

        let size = edi_term::get_size()?.map(|v| v as usize);

        if let Some(filepath) = args.edit_file {
            state.open_file(filepath, Vec2::from_dims(size))?;
        }

        window.set_size(size);
        window.set_cursor(Coord::new(0, 0));
        window.rerender()?;

        redraw(&mut state, &mut window)?;

        let input_handler = handlers::input::InputHandler::new();
        let draw_handler = handlers::draw::DrawHandler::new();

        event_manager.attach_handler(input_handler);
        event_manager.attach_handler(draw_handler);

        let _ = event_manager.run(AppState { state, window });

        let _ = stdout().write(ANSIEscape::ExitAlternateScreen.to_str().as_bytes());

        Ok(())
    })?
}
