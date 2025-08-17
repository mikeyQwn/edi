mod action;

pub mod buffer_bundle;
pub mod buffers;
pub mod meta;
pub mod state;

use action::{Action, MoveAction};
use edi_lib::vec2::Vec2;
use edi_term::{
    coord::Coord,
    escaping::{ANSIEscape, CursorStyle},
    window::Window,
};
use meta::BufferMeta;

use state::State;

use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
};

use edi::string::highlight::get_highlights;

use crate::{
    cli::EdiCli,
    event::{emitter, manager::EventManager, sender::EventBuffer, sources, Event},
    handlers,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Mode {
    Normal,
    Insert,
    Terminal,
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
    buf: &mut EventBuffer,
) -> anyhow::Result<()> {
    let _span = edi_lib::span!("handle_action");

    match event {
        Action::SwitchMode(mode) => {
            buf.add_switch_mode(mode);
        }
        Action::InsertChar(c) => {
            buf.add_write_char(c);
        }
        Action::DeleteChar => {
            buf.add_delete_char();
        }
        Action::Submit => {
            // TODO: Add proper error handling
            let bundle = state.buffers.remove_first().unwrap();
            let cmd_buf = bundle.buffer();
            buf.add_redraw();
            let cmd: String = cmd_buf.inner.chars().collect();
            if cmd == ":q" {
                buf.add_quit();
                return Ok(());
            }
            if cmd == ":wq" {
                let Some(bundle) = state.buffers.remove_first() else {
                    edi_lib::fatal!("no buffer to write")
                };
                let (b, meta) = bundle.as_split();

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
                        buf.add_quit();
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

                if let Err(e) = std::fs::rename(
                    swap_name,
                    meta.filepath.as_ref().unwrap_or(&PathBuf::from("out.txt")),
                ) {
                    edi_lib::debug!("app::handle_event failed to rename file {e}");
                }

                buf.add_quit();
            }
        }

        Action::Move { action, repeat } => {
            match state.buffers.first_mut() {
                Some(bundle) => {
                    let (mut buffer, meta) = bundle.as_split_mut(buf);
                    handle_move(&mut buffer, meta, action, repeat);
                    buf.add_redraw();
                }
                None => {
                    edi_lib::debug!("handle_event: no buffers to move cursor in");
                }
            }
            buf.add_redraw();
        }
        Action::Undo => {
            match state.buffers.first_mut() {
                Some(bundle) => {
                    let (mut buffer, meta) = bundle.as_split_mut(buf);
                    let buffer = buffer.as_mut();
                    edi_lib::debug!("undoing last action");
                    buffer.undo();
                    meta.flush_options.highlights = get_highlights(&buffer.inner, &meta.filetype);
                    buf.add_redraw();
                }
                None => {
                    edi_lib::debug!("handle_event: no buffers to undo in");
                }
            }
            buf.add_redraw();
        }
        Action::Redo => {
            match state.buffers.first_mut() {
                Some(bundle) => {
                    let (mut buffer, meta) = bundle.as_split_mut(buf);
                    let buffer = buffer.as_mut();
                    buffer.redo();
                    meta.flush_options.highlights = get_highlights(&buffer.inner, &meta.filetype);
                    buf.add_redraw();
                }
                None => {
                    edi_lib::debug!("handle_event: no buffers to undo in");
                }
            }
            buf.add_redraw();
        }
    }

    Ok(())
}

fn handle_move(
    buffer: &mut emitter::buffer::Buffer,
    meta: &mut BufferMeta,
    action: MoveAction,
    repeat: usize,
) {
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

/// Runs the `edi` application, blocknig until receiving an error / close signal
pub fn run(args: EdiCli) -> anyhow::Result<()> {
    let mut event_manager = EventManager::new();
    event_manager.attach_source(sources::input_source);

    edi_term::within_alternative_screen_mode(|| {
        let mut window = Window::new();

        let size = edi_term::get_size()?.map(|v| v as usize);

        window.set_size(size);
        window.set_cursor(Coord::new(0, 0));
        window.rerender()?;

        let mut state = State::new(window);

        if let Some(filepath) = args.edit_file {
            state.open_file(filepath, Vec2::from_dims(size))?;
        }

        init_handlers(&mut event_manager);

        event_manager.pipe_event(Event::Redraw);

        let _ = event_manager.run(state);

        let _ = ANSIEscape::ChangeCursor(CursorStyle::Block).write_to_stdout();

        Ok(())
    })?
}

pub fn init_handlers(event_manager: &mut EventManager<State>) {
    let input_handler = handlers::input::Handler::new();
    event_manager.attach_handler(input_handler);

    let draw_handler = handlers::draw::Handler::new();
    event_manager.attach_handler(draw_handler);

    let write_handler = handlers::write::Handler::new();
    event_manager.attach_handler(write_handler);

    let history_handler = handlers::history::Handler::new();
    event_manager.attach_handler(history_handler);

    let mode_handler = handlers::mode::Handler::new();
    event_manager.attach_handler(mode_handler);
}
