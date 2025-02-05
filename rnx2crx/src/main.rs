mod cli;
use cli::Cli;

use rinex::prelude::{FormattingError, ParsingError, Rinex};

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

fn main() -> Result<(), Error> {
    let cli = Cli::new();

    let input_path = cli.input_path();
    let input_name = input_name(&input_path);

    let short_name = cli.matches.get_flag("short");
    let output_name = cli.output_name();
    let gzip_out = cli.gzip_encoding();

    let workspace_path = workspace(&cli).join(&input_name);
    create_workspace(&workspace_path);

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
        workspace_path.join(output_name)
    } else {
        let suffix = if gzip_out { Some(".gz") } else { None };

        let auto = rinex.standard_filename(short_name, suffix, None);
        workspace_path.join(auto)
    };

    let output_file_name = output_path
        .file_name()
        .expect("failed to determine output file")
        .to_string_lossy();

    rinex.to_file(&output_path)?;
    println!("{} generated", output_file_name);

    Ok(())
}
