use edi_lib::brand::Id;
use edi_term::escaping::{ANSIEscape, CursorStyle};

use crate::{
    app::{buffers::Selector, state::State, Mode},
    controller::{self, Handle},
    event::{self, Event, Payload},
};

pub struct Handler;

impl Handler {
    pub const fn new() -> Self {
        Self
    }
}

impl controller::EventHandler<State> for Handler {
    fn handle(&mut self, app_state: &mut State, event: &Event, ctrl: &mut Handle<State>) {
        let _span = edi_lib::span!("mode");
        let Payload::SwitchMode {
            selector,
            target_mode,
        } = event.payload()
        else {
            return;
        };

        let Some(bundle) = app_state.buffers.get_mut(selector) else {
            edi_lib::debug!("no buffer found by selector: {selector:?}");
            return;
        };

        let id = bundle.id();
        edi_lib::debug!("ID: {id:?}");
        let prev_mode = bundle.meta().mode();
        bundle.meta_mut().set_mode(*target_mode);

        if !bundle.is_active() {
            return;
        }

        if prev_mode == Mode::Terminal {
            let _ = app_state.buffers.remove(id);
            edi_lib::debug!(
                "removed active buffer, buffers left: {buffers_left}, target: {target_mode:?}",
                buffers_left = app_state.buffers.len()
            );
            ctrl.add_switch_mode(Selector::Active, *target_mode);
            return;
        }

        if bundle.meta().mode() == Mode::Insert {
            let _ = ANSIEscape::ChangeCursor(CursorStyle::Line).write_to_stdout();
        } else {
            let _ = ANSIEscape::ChangeCursor(CursorStyle::Block).write_to_stdout();
        }

        edi_lib::debug!("mode switched to: {target_mode:?}");
        ctrl.query_redraw();
    }

    fn interested_in(&self, _own_id: Id, event: &Event) -> bool {
        event.ty() == event::Type::SwtichMode
    }
}
