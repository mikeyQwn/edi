use std::io::Write;
use std::path::PathBuf;

#[derive(Debug)]
pub struct EdiCli {
    pub edit_file: Option<PathBuf>,
}

impl EdiCli {
    pub fn parse(mut args: impl Iterator<Item = String>) -> Self {
        let _program_path = args.next().unwrap_or_else(|| {
            let _ = write!(std::io::stderr(), "args[0] is not found");
            std::process::exit(1);
        });

        let path = args.next().map(PathBuf::from);

        let is_file = path
            .as_ref()
            .and_then(|p| p.metadata().ok())
            .map(|metadata| metadata.is_file())
            != Some(false);

        if !is_file {
            let _ = write!(std::io::stderr(), "specified file is not found");
            std::process::exit(1);
        }

        Self { edit_file: path }
    }
}
