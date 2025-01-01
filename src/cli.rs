use std::path::PathBuf;

use crate::log;

#[derive(Debug)]
pub struct EdiCli {
    pub edit_file: Option<PathBuf>,
}

impl EdiCli {
    pub fn parse(mut args: impl Iterator<Item = String>) -> Self {
        let _program_path = args
            .next()
            .unwrap_or_else(|| log::fatal!("args[0] is not found"));

        let path = args.next().map(PathBuf::from);

        let is_file = path
            .as_ref()
            .and_then(|p| p.metadata().ok())
            .map(|metadata| metadata.is_file())
            != Some(false);

        if !is_file {
            log::fatal!("specified file is not found");
        }

        Self { edit_file: path }
    }
}
