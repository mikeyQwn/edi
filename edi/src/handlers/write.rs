use edi::string::highlight::get_highlights;

use crate::{
    app::state::State,
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
        let _span = edi_lib::span!("write");

        match event.ty {
            event::Type::WriteChar => Self::write_char(app_state, event),
            event::Type::DeleteChar => Self::delete_char(app_state),
            _ => {
                return;
            }
        }

        buf.add_redraw();
    }

    fn interested_in(&self, event: &Event) -> bool {
        event.ty == event::Type::WriteChar || event.ty == event::Type::DeleteChar
    }
}

impl Handler {
    fn write_char(app_state: &mut State, event: &Event) {
        let Some(Payload::WriteChar(c)) = event.payload else {
            return;
        };

        match app_state.buffers.front_mut() {
            Some((b, m)) => {
                let is_empty = b.inner.is_empty();
                b.write(c);
                // Hack to always add a newline at the end of the file
                if is_empty {
                    b.write('\n');
                    b.cursor_offset -= 1;
                }
                m.flush_options.highlights = get_highlights(&b.inner, &m.filetype);
            }
            None => {
                edi_lib::debug!("no buffers to write to");
            }
        }
    }

    fn delete_char(app_state: &mut State) {
        match app_state.buffers.front_mut() {
            Some((b, m)) => {
                b.delete();
                m.flush_options.highlights = get_highlights(&b.inner, &m.filetype);
            }
            None => {
                edi_lib::debug!("no buffers to delete from");
            }
        }
    }
}
