use edi_frame::unit::Unit;
use edi_lib::{
    brand::Id, buffer::Buffer, fs::filetype::Filetype, string::highlight::get_highlights,
    vec2::Vec2,
};
use edi_term::window::Window;

use crate::{
    app::{action::InputMapper, context::Context, meta::BufferMeta, Mode},
    controller::Handle,
    event::emitter,
};

use super::buffers::Buffers;

#[derive(Debug)]
pub struct State {
    pub context: Context,

    pub window: Window,

    pub mapper: InputMapper,
    pub buffers: Buffers,
}

impl State {
    /// Instantiates an empty `State` with nothing stored in buffers and mode set to `Normal`
    #[must_use]
    pub fn new(window: Window) -> Self {
        Self {
            context: Context::new(),

            window,
            mapper: InputMapper::default(),
            buffers: Buffers::new(),
        }
    }

    /// Opens a file with the given path, appending it's contents to the leftmost buffer
    pub fn open_file(
        &mut self,
        filepath: impl AsRef<std::path::Path>,
        buff_dimensions: Vec2<Unit>,
    ) -> anyhow::Result<()> {
        let filepath = filepath.as_ref();
        let contents = std::fs::read_to_string(filepath)?;

        let buffer = Buffer::new(&contents);
        let filetype = Filetype::from(filepath);

        let hl = get_highlights(&buffer.inner, &filetype);
        let meta = BufferMeta::new(Mode::Normal)
            .with_filepath(Some(filepath.into()))
            .with_filetype(filetype)
            .with_size(buff_dimensions)
            .with_statusline(true)
            .with_highlights(hl)
            .with_line_numbers(true);

        self.buffers.attach(buffer, meta);

        Ok(())
    }

    pub fn within_active_buffer<F>(&mut self, mut f: F, ctrl: &mut Handle<State>)
    where
        F: FnMut(Id, emitter::buffer::Buffer, &mut BufferMeta),
    {
        let _ = self
            .buffers
            .active_mut()
            .map(|bundle| (bundle.id(), bundle.as_split_mut(ctrl)))
            .map(|(id, (buffer, meta))| f(id, buffer, meta));
    }
}
