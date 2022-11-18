mod cli;
use cli::Cli;
use rinex::{
    Error,
    epoch,
    prelude::*,
    version::Version,
    observation::Crinex,
};
fn main() -> Result<(), Error> {
    let cli = Cli::new(); 
    let input_path = cli.input_path();
    // CRINEX attributes
    let mut crinex = Crinex::default();
    if cli.crx1() {
        crinex.version = Version {
            major: 1,
            minor: 0,
        };
    }
    if cli.crx3() {
        crinex.version = Version {
            major: 3,
            minor: 0,
        };
    }
    let date = cli.date();
    let time = cli.time();
    if let Some((y, m, d)) = date {
        let (hh, mm, ss) = cli.time().unwrap_or((0,0,0));
        crinex.date = Epoch::from_gregorian_utc(y, m, d, hh, mm, ss, 0);
    } else if let Some(time) = time {
        crinex.date = Epoch::now().expect("failed to retrieve system time");
    }
    // output path
    let output_path = match cli.output_path() {
        Some(path) => path.clone(),
        _ => { // deduce from input
            let mut outpath = String::with_capacity(64);
            if let Some(stripped) = input_path.strip_suffix("o") { // RNX1 
                outpath = stripped.to_owned() + "d" // CRNX1
            } else if let Some(stripped) = input_path.strip_suffix("O") { // RNX1 
                outpath = stripped.to_owned() + "D" // CRNX1
            } else {
                if let Some(stripped) = input_path.strip_suffix("rnx") { // RNX3
                    outpath = stripped.to_owned() + "crx" // CRNX3
                }
            }
            outpath
        },
    };
    println!("Compressing \"{}\"..", input_path);
    let mut rinex = Rinex::from_file(input_path)?; // parse
    // convert
    rinex.header = rinex.header.clone()
        .with_crinex(crinex);
    rinex.to_file(&output_path)?;
    println!("{} generated", output_path);
    Ok(())
}
