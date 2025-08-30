use edi_lib::brand::Id;

use crate::{
    app::{
        action::Action, buffer_bundle::BufferBundle, buffers::Selector, meta::Flags, state::State,
        Mode,
    },
    controller::{self, Handle},
    event::{self, Event, Payload},
    query::{CommandQuery, HistoryQuery, MoveQuery, SpawnQuery, WriteQuery},
};

pub struct Handler;

impl Handler {
    pub const fn new() -> Self {
        Self
    }

    fn handle_action(ctrl: &mut Handle<State>, state: &State, action: Action) {
        let _span = edi_lib::span!("handle_action");

        match action {
            Action::SwitchMode(Mode::Terminal) => {
                ctrl.query_spawn(SpawnQuery::TerminalBuffer);
            }
            Action::SwitchMode(mode) => {
                ctrl.query_switch_mode(Selector::Active, mode);
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
                let Some(bundle) = state.buffers.first() else {
                    edi_lib::debug!("invalid submit query, no buffers are found");
                    return;
                };

                let cmd_buf = bundle.buffer();
                let command: String = cmd_buf.inner.chars().collect();

                ctrl.query_command(CommandQuery { command });

                edi_lib::debug!(
                    "exit submit action with {buf_count} buffers",
                    buf_count = state.buffers.len()
                );
                ctrl.query_switch_mode(Selector::Active, Mode::Normal);
            }

            Action::Move { action, repeat } => {
                ctrl.query_move(MoveQuery::Action { action, repeat });
            }
            Action::Undo => {
                ctrl.query_history(HistoryQuery::Undo(Selector::Active));
            }
            Action::Redo => {
                ctrl.query_history(HistoryQuery::Redo(Selector::Active));
            }
        }
    }
}

impl controller::EventHandler<State> for Handler {
    fn handle(&mut self, app_state: &State, event: &Event, ctrl: &mut Handle<State>) {
        let Payload::Input(input) = event.payload() else {
            return;
        };

        let _span = edi_lib::span!("input");

        let (active_mode, active_flags) = app_state
            .buffers
            .active()
            .map(BufferBundle::meta)
            .map(|bundle| (bundle.mode(), bundle.flags))
            .unwrap_or((Mode::Normal, Flags::empty()));

        let actions = app_state.mapper.map_input(input, active_mode, active_flags);
        for action in actions {
            Self::handle_action(ctrl, app_state, action);
        }
    }

    fn interested_in(&self, _own_id: Id, event: &Event) -> bool {
        event.ty() == event::Type::Input
    }
}
