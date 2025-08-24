use edi::{buffer::Buffer, string::highlight::get_highlights};
use edi_lib::{fs::filetype::Filetype, vec2::Vec2};
use edi_term::window::Window;

use crate::{
    app::{action::InputMapper, context::Context, meta::BufferMeta, Mode},
    controller::Handle,
    event::{emitter, sender::EventBuffer},
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
        buff_dimensions: Vec2<usize>,
    ) -> anyhow::Result<()> {
        let filepath = filepath.as_ref();
        let contents = std::fs::read_to_string(filepath)?;

        let buffer = Buffer::new(&contents);
        let filetype = Filetype::from(filepath);

        let mut meta = BufferMeta::new(Mode::Normal)
            .with_filepath(Some(filepath.into()))
            .with_filetype(filetype)
            .with_size(buff_dimensions);

        meta.flush_options = meta
            .flush_options
            .with_highlights(get_highlights(&buffer.inner, &meta.filetype))
            .with_line_numbers(true);

        self.buffers.attach(buffer, meta);

        Ok(())
    }

    pub fn within_active_buffer<F>(&mut self, mut f: F, ctrl: &mut Handle<State>)
    where
        F: FnMut(emitter::buffer::Buffer, &mut BufferMeta),
    {
        let _ = self
            .buffers
            .active_mut()
            .map(|bundle| bundle.as_split_mut(ctrl))
            .map(|(buffer, meta)| f(buffer, meta));
    }
}
