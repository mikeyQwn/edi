use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
};

use crate::{
    app::state::State,
    controller::{self, Handle},
    query::{CommandQuery, Payload, Query},
};

pub struct Handler;

impl Handler {
    pub const fn new() -> Self {
        Self
    }
}

impl controller::QueryHandler<State> for Handler {
    fn handle(&mut self, state: &mut State, query: Query, ctrl: &mut Handle<State>) {
        let _span = edi_lib::span!("write");

        let Payload::Command(command_query) = query.payload() else {
            edi_lib::debug!(
                "non-spawn query submitted to spawn query handler, this is likely a bug"
            );
            return;
        };

        match command_query {
            CommandQuery { command } => Self::handle_command(state, ctrl, command),
        }

        ctrl.query_redraw();
    }
}

impl Handler {
    fn handle_command(state: &mut State, ctrl: &mut Handle<State>, command: &str) {
        if command == ":q" {
            ctrl.query_quit();
        }
        if command == ":wq" {
            let Some(bundle) = state.buffers.second() else {
                edi_lib::fatal!("no buffer to write")
            };
            let (b, meta) = bundle.as_split();

            let swap_name = meta
                .filepath
                .as_ref()
                .map_or(PathBuf::from("out.swp"), |fp| {
                    let mut fp = fp.clone();
                    fp.set_extension(".swp");
                    fp
                });

            let file = match OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&swap_name)
            {
                Ok(f) => f,
                Err(e) => {
                    edi_lib::debug!("unable to create output file {e} {swap_name:?}");
                    ctrl.query_quit();
                    return;
                }
            };

            let mut w = BufWriter::new(file);
            b.inner.lines().for_each(|line| {
                let Err(err) = w
                    .write_all(line.contents.as_bytes())
                    .and_then(|()| w.write_all(b"\n"))
                else {
                    return;
                };
                edi_lib::debug!("unable to write line contents: {:?}", err);
            });

            if let Err(e) = std::fs::rename(
                swap_name,
                meta.filepath.as_ref().unwrap_or(&PathBuf::from("out.txt")),
            ) {
                edi_lib::debug!("app::handle_event failed to rename file {e}");
            }

            ctrl.query_quit();
        }
    }
}
