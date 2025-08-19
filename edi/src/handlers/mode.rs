use edi::buffer::Buffer;
use edi_lib::{brand::Id, vec2::Vec2};
use edi_term::escaping::{ANSIEscape, CursorStyle};

use crate::{
    app::{meta::BufferMeta, state::State, Mode},
    event::{self, manager, sender::EventBuffer, Event, Payload},
};

pub struct Handler;

impl Handler {
    pub const fn new() -> Self {
        Self
    }
}

impl manager::Handler<State> for Handler {
    fn handle(&mut self, app_state: &mut State, event: &Event, buf: &mut EventBuffer) {
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

        let _ = bundle.meta_mut().set_mode(*target_mode);

        if !bundle.is_active() {
            return;
        }

        if app_state.mode == Mode::Terminal {
            let _ = app_state.buffers.remove_first();
        }
        app_state.mode = *target_mode;
        if app_state.mode == Mode::Insert {
            let _ = ANSIEscape::ChangeCursor(CursorStyle::Line).write_to_stdout();
        } else {
            let _ = ANSIEscape::ChangeCursor(CursorStyle::Block).write_to_stdout();
        }

        edi_lib::debug!("mode switched to: {target_mode:?}");
        if app_state.mode == Mode::Terminal {
            let size = edi_term::get_size()
                .map(Vec2::from_dims)
                .unwrap_or(Vec2::new(10, 1));
            let mut buffer = Buffer::new(":");
            buffer.cursor_offset = 1;
            app_state.buffers.attach_first(
                buffer,
                BufferMeta::new(Mode::Terminal).with_size(Vec2::new(size.x as usize, 1)),
            );
        }
        buf.add_redraw();
    }

    fn interested_in(&self, _own_id: Id, event: &Event) -> bool {
        event.ty() == event::Type::SwtichMode
    }
}
