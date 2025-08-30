use edi_term::escaping::{ANSIEscape, CursorStyle};

use crate::{
    app::{buffers::Selector, state::State, Mode},
    controller::{self, Handle},
    event::{self},
    query::{Payload, Query},
};

pub struct Handler;

impl Handler {
    pub const fn new() -> Self {
        Self
    }
}

impl controller::QueryHandler<State> for Handler {
    fn handle(&mut self, app_state: &mut State, query: Query, ctrl: &mut Handle<State>) {
        let _span = edi_lib::span!("mode");
        let Payload::SwitchMode {
            buffer_selector,
            target_mode,
        } = query.into_payload()
        else {
            edi_lib::debug!("non-mode query submitted to mode query handler, this is likely a bug");
            return;
        };

        let Some(bundle) = app_state.buffers.get_mut(&buffer_selector) else {
            edi_lib::debug!("no buffer found by selector: {buffer_selector:?}");
            return;
        };

        let id = bundle.id();
        edi_lib::debug!("ID: {id:?}");
        bundle.meta_mut().set_mode(target_mode);

        if !bundle.is_active() {
            return;
        }

        if bundle.meta().flags.is_terminal() {
            let _ = app_state.buffers.remove(id);
            edi_lib::debug!(
                "removed active buffer, buffers left: {buffers_left}, target: {target_mode:?}",
                buffers_left = app_state.buffers.len()
            );
            ctrl.query_switch_mode(Selector::Active, target_mode);
            return;
        }

        if bundle.meta().mode() == Mode::Insert {
            let _ = ANSIEscape::ChangeCursor(CursorStyle::Line).write_to_stdout();
        } else {
            let _ = ANSIEscape::ChangeCursor(CursorStyle::Block).write_to_stdout();
        }

        edi_lib::debug!("mode switched to: {target_mode:?}");

        ctrl.add_event(event::Payload::ModeSwitched {
            buffer_id: id,
            target_mode,
        });

        ctrl.query_redraw();
    }
}
