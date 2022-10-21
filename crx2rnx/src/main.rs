//! Command line tool to decompress CRINEX files 
use rinex::*;

mod cli;
use cli::Cli;

fn main() -> Result<(), rinex::Error> {
    let cli = Cli::new();
    let input_path = cli.input_path(); 
    let output_path = match cli.output_path() {
        Some(path) => path.clone(),
        _ => { // deduce from input path
            let mut outpath = String::with_capacity(64);
            if let Some(prefix) = input_path.strip_suffix("d") { // CRNX1
                outpath = prefix.to_owned() + "o" // RNX1
            } else {
                if let Some(prefix) = input_path.strip_suffix("crx") { // CRNX3
                    outpath = prefix.to_owned() + "rnx" // RNX3
                }
            }
            outpath
        },
    };

    let mut rinex = Rinex::from_file(input_path)?; // parse
    rinex.crx2rnx(); // convert to RINEX
    rinex.to_file(&output_path)?; // dump
    println!("{} generated", output_path);
    Ok(())
}
