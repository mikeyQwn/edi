use edi::buffer::Buffer;
use edi_lib::vec2::Vec2;
use edi_term::escaping::{ANSIEscape, CursorStyle};

use crate::{
    app::{meta::BufferMeta, state::State, Mode},
    event::{self, manager, sender::EventBuffer, Event},
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
        let &Event::SwitchMode(target_mode) = event else {
            return;
        };

        if app_state.mode == Mode::Terminal {
            let _ = app_state.buffers.remove_first();
        }
        app_state.mode = target_mode;
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
                BufferMeta::default().with_size(Vec2::new(size.x as usize, 1)),
            );
        }
        buf.add_redraw();
    }

    fn interested_in(&self, event: &Event) -> bool {
        event.ty() == event::Type::SwtichMode
    }
}
