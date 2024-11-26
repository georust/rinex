mod cli;
use cli::Cli;

use rinex::prelude::{Epoch, FormattingError, ParsingError, Rinex};

use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error("parsing error")]
    ParsingError(#[from] ParsingError),
    #[error("formatting error")]
    FormattingError(#[from] FormattingError),
}

fn workspace(cli: &Cli) -> PathBuf {
    if let Some(workspace) = cli.workspace() {
        Path::new(workspace).to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("WORKSPACE")
    }
}

fn main() -> Result<(), Error> {
    let cli = Cli::new();
    let input_path = cli.input_path();
    let short_name = cli.matches.get_flag("short");
    let output_name = cli.output_name();
    let gzip_out = cli.gzip_encoding();

    let workspace = workspace(&cli);

    let extension = input_path
        .extension()
        .expect("failed to determine file extension")
        .to_string_lossy();

    let gzip_input = extension == "gz";

    let mut rinex = if gzip_input {
        Rinex::from_gzip_file(input_path)?
    } else {
        Rinex::from_file(input_path)?
    };

    rinex.rnx2crnx_mut();

    // output path
    let output_path = if let Some(output_name) = output_name {
        workspace.join(output_name)
    } else {
        let suffix = if gzip_out { Some(".gz") } else { None };

        let auto = rinex.standard_filename(short_name, suffix, None);
        workspace.join(auto)
    };

    let output_file_name = output_path
        .file_name()
        .expect("failed to determine output file")
        .to_string_lossy();

    rinex.to_file(&output_path)?;
    println!("{} generated", output_file_name);

    Ok(())
}
