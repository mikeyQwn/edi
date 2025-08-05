use std::collections::VecDeque;

use edi::{buffer::Buffer, string::highlight::get_highlights};
use edi_lib::{fs::filetype::Filetype, vec2::Vec2};
use edi_term::window::Window;

use crate::app::{action::InputMapper, meta::BufferMeta, Mode};

#[derive(Debug)]
pub struct State {
    pub window: Window,

    pub mode: Mode,
    pub mapper: InputMapper,
    pub buffers: VecDeque<(Buffer, BufferMeta)>,
}

impl State {
    /// Instantiates an empty `State` with nothing stored in buffers and mode set to `Normal`
    #[must_use]
    pub fn new(window: Window) -> Self {
        Self {
            window,
            mode: Mode::Normal,
            mapper: InputMapper::default(),
            buffers: VecDeque::new(),
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

        let mut meta = BufferMeta::default()
            .with_filepath(Some(filepath.into()))
            .with_filetype(filetype)
            .with_size(buff_dimensions);

        meta.flush_options = meta
            .flush_options
            .with_highlights(get_highlights(&buffer.inner, &meta.filetype))
            .with_line_numbers(true);

        self.buffers.push_back((buffer, meta));

        Ok(())
    }
}
