//! Command line tool to decompress CRINEX files
use rinex::*;

mod cli;
use cli::Cli;

fn main() -> Result<(), rinex::Error> {
    let cli = Cli::new();
    let input_path = cli.input_path();
    println!("decompressing \"{}\"..", input_path);

    let output_path = match cli.output_path() {
        Some(path) => path.clone(),
        _ => {
            // deduce from input path
            match input_path.strip_suffix('d') {
                Some(prefix) => prefix.to_owned() + "o",
                _ => match input_path.strip_suffix('D') {
                    Some(prefix) => prefix.to_owned() + "O",
                    _ => match input_path.strip_suffix("crx") {
                        Some(prefix) => prefix.to_owned() + "rnx",
                        _ => String::from("output.rnx"),
                    },
                },
            }
        },
    };
    let mut rinex = Rinex::from_file(input_path)?; // parse
    rinex.crnx2rnx_mut(); // convert to RINEX
    rinex.to_file(&output_path)?; // dump
    println!("\"{}\" generated", output_path.clone());
    Ok(())
}
