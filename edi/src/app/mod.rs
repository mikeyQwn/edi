mod action;

pub mod buffer_bundle;
pub mod buffers;
pub mod context;
pub mod meta;
pub mod state;

use action::{Action, MoveAction};
use buffers::Selector;
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

use edi::buffer::Buffer;

use crate::{
    cli::EdiCli,
    controller::{Controller, Handle},
    event::{emitter, sources},
    handlers,
    query::{self, HistoryQuery, WriteQuery},
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
    ctrl: &mut Handle<State>,
) -> anyhow::Result<()> {
    let _span = edi_lib::span!("handle_action");

    match event {
        Action::SwitchMode(Mode::Terminal) => {
            let mut size = edi_term::get_size()
                .map(Vec2::from_dims)
                .unwrap_or(Vec2::new(10, 1))
                .map(|v| v as usize);
            size.y = 1;

            let mut buffer = Buffer::new(":");
            buffer.cursor_offset = 1;

            state
                .buffers
                .attach_first(buffer, BufferMeta::new(Mode::Terminal).with_size(size));
            ctrl.query_redraw();
        }
        Action::SwitchMode(mode) => {
            ctrl.add_switch_mode(Selector::Active, mode);
        }
        Action::InsertChar(c) => {
            ctrl.query_write(WriteQuery::WriteChar(c));
        }
        Action::DeleteChar => {
            ctrl.query_write(WriteQuery::DeleteChar);
        }
        Action::Submit => {
            let _span = edi_lib::span!("submit");
            edi_lib::debug!(
                "hit submit action with {buf_count} buffers",
                buf_count = state.buffers.len()
            );
            // TODO: Add proper error handling
            let bundle = state.buffers.first().unwrap();
            let cmd_buf = bundle.buffer();
            ctrl.query_redraw();
            let cmd: String = cmd_buf.inner.chars().collect();
            if cmd == ":q" {
                ctrl.query_quit();
                return Ok(());
            }
            if cmd == ":wq" {
                let Some(bundle) = state.buffers.second() else {
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
                        ctrl.query_quit();
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

                ctrl.query_quit();
            }

            edi_lib::debug!(
                "exit submit action with {buf_count} buffers",
                buf_count = state.buffers.len()
            );
            ctrl.add_switch_mode(Selector::Active, Mode::Normal);
        }

        Action::Move { action, repeat } => {
            state.within_active_buffer(
                |mut buffer, meta| {
                    handle_move(&mut buffer, meta, &action, repeat);
                    buffer.ctrl().query_redraw();
                },
                ctrl,
            );
        }
        Action::Undo => {
            ctrl.query_history(HistoryQuery::Undo(Selector::Active));
        }
        Action::Redo => {
            ctrl.query_history(HistoryQuery::Redo(Selector::Active));
        }
    }

    Ok(())
}

fn handle_move(
    buffer: &mut emitter::buffer::Buffer,
    meta: &mut BufferMeta,
    action: &MoveAction,
    repeat: usize,
) {
    match action {
        &MoveAction::Regular(direction) => {
            buffer.move_cursor(direction.into(), repeat);
        }
        &MoveAction::InLine(line_position) => {
            buffer.move_in_line(line_position);
        }
        &MoveAction::HalfScreen(direction) => {
            buffer.move_cursor(direction.into(), meta.size.y / 2);
        }
        &MoveAction::Global(global_position) => buffer.move_global(global_position),
    }
}

/// Runs the `edi` application, blocknig until receiving an error / close signal
pub fn run(args: EdiCli) -> anyhow::Result<()> {
    let mut controller = Controller::new();

    controller.attach_source(sources::input_source);

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

        init_handlers(&mut controller);

        controller.pipe_query(query::Payload::Redraw);

        let _ = controller.run(state);

        let _ = ANSIEscape::ChangeCursor(CursorStyle::Block).write_to_stdout();

        Ok(())
    })?
}

pub fn init_handlers(controller: &mut Controller<State>) {
    let input_handler = handlers::input::Handler::new();
    controller.attach_event_handler(input_handler);

    let write_handler = handlers::write::Handler::new();
    controller.attach_query_handler(query::Type::Write, write_handler);

    let draw_handler = handlers::draw::Handler::new();
    controller.attach_query_handler(query::Type::Redraw, draw_handler);

    let history_handler = handlers::history::Handler::new();
    controller.attach_query_handler(query::Type::History, history_handler);

    let mode_handler = handlers::mode::Handler::new();
    controller.attach_event_handler(mode_handler);
}
