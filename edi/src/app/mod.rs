pub mod action;
pub mod buffer_bundle;
pub mod buffers;
pub mod context;
pub mod meta;
pub mod state;

use action::MoveAction;
use edi_frame::unit::Unit;
use edi_lib::vec2::Vec2;
use edi_term::{
    coord::Coord,
    escaping::{ANSIEscape, CursorStyle},
    window::Window,
};
use meta::BufferMeta;

use state::State;

use crate::{
    cli::EdiCli,
    controller::Controller,
    event::{emitter, sources},
    handlers, query,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Mode {
    Normal,
    Insert,
    Terminal,
}

pub fn handle_move(
    buffer: &mut emitter::buffer::Buffer,
    meta: &mut BufferMeta,
    action: &MoveAction,
    repeat: usize,
) {
    match *action {
        MoveAction::Regular(direction) => {
            buffer.move_cursor(direction.into(), repeat);
        }
        MoveAction::InLine(line_position) => {
            buffer.move_in_line(line_position);
        }
        MoveAction::HalfScreen(direction) => {
            let Ok(dimensions) = edi_term::get_size() else {
                edi_lib::debug!("unable to get trminal dimensions");
                return;
            };
            let dimensions = dimensions.map(|v| v as usize);
            buffer.move_cursor(direction.into(), meta.size.y.resolve(dimensions) / 2);
        }
        MoveAction::Global(global_position) => buffer.move_global(global_position),
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
            state.open_file(filepath, Vec2::new(Unit::full_width(), Unit::full_height()))?;
        }

        init_handlers(&mut controller);

        controller.pipe_query(query::Payload::Draw(query::DrawQuery::Redraw));

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

    let history_handler = handlers::history::Handler::new();
    controller.attach_query_handler(query::Type::History, history_handler);

    let mode_handler = handlers::mode::Handler::new();
    controller.attach_query_handler(query::Type::SwitchMode, mode_handler);

    let spawn_handler = handlers::spawn::Handler::new();
    controller.attach_query_handler(query::Type::Spawn, spawn_handler);

    let move_handler = handlers::movement::Handler::new();
    controller.attach_query_handler(query::Type::Move, move_handler);

    let command_handler = handlers::command::Handler::new();
    controller.attach_query_handler(query::Type::Command, command_handler);

    let draw_handler = handlers::draw::Handler::new();
    controller.attach_query_handler(query::Type::Draw, draw_handler);
}
