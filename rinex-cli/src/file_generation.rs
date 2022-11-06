use rinex::*;

/// Macro to generate a file from an input RINEX
/// with possible control over the file name, from the command line
pub fn generate(rnx: &Rinex, fname: Option<&str>, default: &str) -> Result<(), rinex::Error> {
    let path: &str = match fname {
        Some(p) => p,
        _ => { // no option from the command line
            default
            /* TODO
                if observation
                    if self.is a crinex
                      and user requested a non crinex format
                       ==> crx2rnx()
                    else if self is not a crinex
                      and user requested a crinex format
                       ==> rnx2crnx()
                    
                for all:
                   if ".gz" => secondary compression layer
                             should be naturally added
            */
        },
    };
    rnx.to_file(path)
        .expect(&format!("failed to generate \"{}\"", path));
    println!("\"{}\" has been generated", path);
    Ok(())
}
