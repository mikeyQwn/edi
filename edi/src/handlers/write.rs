use edi::string::highlight::get_highlights;

use crate::{
    app::AppState,
    event::{self, manager::Handler, sender::EventBuffer, Event, Payload},
};

pub struct WriteHandler;

impl WriteHandler {
    pub const fn new() -> Self {
        Self
    }
}

impl Handler<AppState> for WriteHandler {
    fn handle(&mut self, app_state: &mut AppState, event: &Event, buf: &mut EventBuffer) {
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

impl WriteHandler {
    fn write_char(app_state: &mut AppState, event: &Event) {
        let Some(Payload::WriteChar(c)) = event.payload else {
            return;
        };

        match app_state.state.buffers.front_mut() {
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

    fn delete_char(app_state: &mut AppState) {
        match app_state.state.buffers.front_mut() {
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
