use crate::{
    app::{self, action::MoveAction, state::State},
    controller::{self, Handle},
    query::{MoveQuery, Payload, Query},
};

pub struct Handler;

impl Handler {
    pub const fn new() -> Self {
        Self
    }
}

impl controller::QueryHandler<State> for Handler {
    fn handle(&mut self, state: &mut State, query: Query, ctrl: &mut Handle<State>) {
        let _span = edi_lib::span!("write");

        let Payload::Move(move_query) = query.into_payload() else {
            edi_lib::debug!("non-move query submitted to move query handler, this is likely a bug");
            return;
        };

        match move_query {
            MoveQuery::Action { action, repeat } => {
                Self::handle_action(state, ctrl, &action, repeat);
            }
        }

        ctrl.query_redraw();
    }
}

impl Handler {
    fn handle_action(
        state: &mut State,
        ctrl: &mut Handle<State>,
        action: &MoveAction,
        repeat: usize,
    ) {
        state.within_active_buffer(
            |mut buffer, meta| {
                app::handle_move(&mut buffer, meta, action, repeat);
                buffer.ctrl().query_redraw();
            },
            ctrl,
        );
    }
}
