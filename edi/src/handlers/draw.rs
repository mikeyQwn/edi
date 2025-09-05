use edi_frame::rect::Rect;
use edi_frame::{cell::Color, prelude::*};
use edi_lib::string::highlight::get_highlights;
use edi_term::escaping::ANSIColor;

use crate::{
    app::{buffers::Selector, state::State},
    controller::{self, Handle},
    query::{DrawQuery, Payload, Query},
};

pub struct Handler;

impl Handler {
    pub const fn new() -> Self {
        Self
    }

    fn redraw(state: &mut State, ctrl: &mut Handle<State>) {
        let ctx = &state.context;

        edi_lib::debug!(
            "drawing {buffer_count} buffers",
            buffer_count = state.buffers.len()
        );

        let Ok(dimensions) = edi_term::get_size() else {
            edi_lib::debug!("unable to get trminal dimensions");
            return;
        };
        let dimensions = dimensions.map(|v| v as usize);

        state.window.clear(ANSIColor::Reset);
        state.buffers.iter_mut().rev().for_each(|bundle| {
            let (buffer, meta) = bundle.as_split_mut(ctrl);
            meta.normalize(ctx, buffer.as_ref(), dimensions);

            let (offset_x, offset_y) = (
                meta.offset.x.resolve(dimensions),
                meta.offset.y.resolve(dimensions),
            );
            let (size_x, size_y) = (
                meta.size.x.resolve(dimensions),
                meta.size.y.resolve(dimensions),
            );

            let mut bound = Rect::new(offset_x, offset_y, size_x, size_y).bind(&mut state.window);
            bound.clear(Color::None);

            buffer
                .as_ref()
                .flush(&mut bound, &meta.updated_flush_options(ctx));
        });

        if let Err(err) = state.window.render() {
            edi_lib::debug!("{err}");
        }
    }

    fn rehighlight(state: &mut State, ctrl: &mut Handle<State>, selector: &Selector) {
        let _span = edi_lib::span!("rehighlight");

        let Some(bundle) = state.buffers.get_mut(selector) else {
            edi_lib::debug!("invalid selector passed {selector:?}");
            return;
        };

        let (buffer, meta) = bundle.as_split_mut(ctrl);
        meta.flush_options.highlights = get_highlights(&buffer.as_ref().inner, &meta.filetype);
        edi_lib::debug!("buffer with id: {id:?} rehighlighted", id = bundle.id());
    }
}

impl controller::QueryHandler<State> for Handler {
    fn handle(&mut self, state: &mut State, query: Query, ctrl: &mut Handle<State>) {
        let _span = edi_lib::span!("draw");

        let Payload::Draw(draw_query) = query.payload() else {
            edi_lib::debug!("non-draw query submitted to draw query handler, this is likely a bug");
            return;
        };

        match draw_query {
            DrawQuery::Redraw => Self::redraw(state, ctrl),
            DrawQuery::Rehighlight(selector) => Self::rehighlight(state, ctrl, selector),
        }
    }
}
