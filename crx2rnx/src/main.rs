mod cli;

use cli::Cli;
use std::path::{Path, PathBuf};
use thiserror::Error;

use rinex::prelude::{FormattingError, ParsingError, Rinex};

fn workspace(cli: &Cli) -> PathBuf {
    if let Some(workspace) = cli.workspace() {
        Path::new(workspace).to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("WORKSPACE")
    }
}

fn create_workspace(path: &PathBuf) {
    std::fs::create_dir_all(path).unwrap_or_else(|_| {
        panic!(
            "failed to create workspace \"{}\": permission denied",
            path.to_string_lossy(),
        )
    });
}

fn input_name(path: &PathBuf) -> String {
    let stem = path
        .file_stem()
        .expect("failed to determine input file name")
        .to_str()
        .expect("failed to determine input file name");

    if stem.ends_with(".crx") {
        stem.strip_suffix(".crx")
            .expect("failed to determine input file name")
            .to_string()
    } else {
        stem.to_string()
    }
}

#[derive(Debug, Error)]
enum Error {
    #[error("parsing error")]
    ParsingError(#[from] ParsingError),
    #[error("formatting error")]
    FormattingError(#[from] FormattingError),
}

fn main() -> Result<(), Error> {
    let cli = Cli::new();

    let input_path = cli.input_path();
    let input_name = input_name(&input_path);

    let workspace_path = workspace(&cli).join(&input_name);

    create_workspace(&workspace_path);

    let extension = input_path
        .extension()
        .expect("failed to determine file extension")
        .to_string_lossy();

    let gzip_input = extension == "gz";
    let gzip_output = cli.gzip_output();

    let mut rinex = if gzip_input {
        Rinex::from_gzip_file(&input_path)?
    } else {
        Rinex::from_file(&input_path)?
    };

    let suffix = if gzip_input || gzip_output {
        Some(".gz")
    } else {
        None
    };

    rinex.crnx2rnx_mut(); // convert to RINEX

    let output_name = match cli.output_name() {
        Some(name) => name.clone(),
        _ => rinex.standard_filename(cli.matches.get_flag("short"), suffix, None),
    };

    let output_path = workspace_path.join(output_name.clone());

    // dump as RINEX
    if gzip_input || gzip_output {
        rinex.to_gzip_file(&output_path)?;
    } else {
        rinex.to_file(output_path)?;
    }

    println!("\"{}\" generated", output_name);
    Ok(())
}
