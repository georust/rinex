//! Command line tool to decompress CRINEX files
use rinex::*;

mod cli;
use cli::Cli;
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
    std::fs::create_dir_all(&path).unwrap_or_else(|_| {
        panic!(
            "failed to create workspace \"{}\": permission denied",
            path.to_string_lossy(),
        )
    });
}

// deduce output name, from input name
fn output_filename<'a>(stem: &'a str, path: &PathBuf) -> String {

    let extension = path.extension()
        .expect("failed to determine input file extension")
        .to_str()
        .expect("failed to determine input file extension");

    if extension.eq("gz") {
        String::from("None")
    } else {
        if extension.contains('d') {
            extension.replace('d', "o").to_string()
        } else if extension.contains('D') {
            extension.replace('D', "O").to_string()
        } else {
            format!("{}.rnx", stem)
        }
    }
}

fn main() -> Result<(), rinex::Error> {
    let cli = Cli::new();

    let input_path = cli.input_path();
    let input_stem = input_path.file_stem()
        .expect("failed to determine input file name")
        .to_str()
        .expect("failed to determine input file name");
    
    println!("decompressing \"{}\"..", input_stem);

    let workspace_path = workspace(&cli)
        .join(input_stem);

    create_workspace(&workspace_path);

    let output_name = match cli.output_name() {
        Some(name) => name.clone(),
        _ => output_filename(input_stem, &input_path),
    };

    let filepath = input_path.to_string_lossy();
    
    let mut rinex = Rinex::from_file(&filepath)?;
    rinex.crnx2rnx_mut(); // convert to RINEX

    let outputpath = format!("{}/{}", 
        workspace_path.to_string_lossy(),
        output_name);

    rinex.to_file(&outputpath)?; // dump
    println!("\"{}\" generated", outputpath.clone());
    Ok(())
}
