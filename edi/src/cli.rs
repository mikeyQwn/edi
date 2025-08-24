use std::path::PathBuf;

use crate::error::{AppError, Result};

#[derive(Debug)]
pub struct EdiCli {
    pub edit_file: Option<PathBuf>,
}

impl EdiCli {
    pub fn parse(mut args: impl Iterator<Item = String>) -> Result<Self> {
        use std::format as f;
        let program_path = args.next().ok_or_else(|| {
            AppError::unexpected("unable to read the application name, 0 arguments provided")
        })?;

        let path_str = args.next();
        let path = path_str.clone().map(PathBuf::from);

        let is_file = path.as_ref().map(|p| p.is_file()) != Some(false);

        if !is_file {
            return Err(AppError::invalid_argument(f!(
                "`{path}` does not exist or is a directory",
                path = path_str.unwrap_or_default()
            ))
            .with_hint(f!("run `{program_path} <file_to_edit>`")));
        }

        Ok(Self { edit_file: path })
    }
}
