mod cli;
use cli::Cli;
use rinex::*;
use std::path::{Path, PathBuf};

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

fn main() -> Result<(), rinex::Error> {
    let cli = Cli::new();

    let input_path = cli.input_path();
    let input_name = input_name(&input_path);
    println!("decompressing \"{}\"..", input_name);

    let workspace_path = workspace(&cli).join(&input_name);

    create_workspace(&workspace_path);

    let filepath = input_path.to_string_lossy();

    let mut rinex = Rinex::from_file(&filepath)?;
    rinex.crnx2rnx_mut(); // convert to RINEX

    // if input was gzip'ed: preserve it
    let suffix = if input_name.ends_with(".gz") {
        Some(".gz")
    } else {
        None
    };

    let output_name = match cli.output_name() {
        Some(name) => name.clone(),
        _ => {
            if cli.matches.get_flag("short") {
                rinex
                    .standardized_short_filename(false, None, suffix)
                    .expect(
                        "Failed to generate a standardized filename.
Your input is too far from standard naming conventions.
You should use --output then.",
                    )
            } else {
                rinex.standardized_filename(suffix).expect(
                    "Failed to generate a standardized filename.
Your input is too far from standard naming conventions.
You should use --output then.",
                )
            }
        },
    };

    let outputpath = format!("{}/{}", workspace_path.to_string_lossy(), output_name);

    rinex.to_file(&outputpath)?; // dump
    println!("\"{}\" generated", outputpath.clone());
    Ok(())
}
