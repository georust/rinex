//! This package provides a set of tools to parse 
//! and analyze RINEX files.
//! 
//! This lib is work in progress
//! 
//! Homepage: <https://github.com/gwbres/rinex>

use thiserror::Error;
use std::str::FromStr;
use scan_fmt::scan_fmt;

mod header;
mod constellation;
//mod navigation; 
//mod observation; 

/// `Rinex` main structure,
/// describes a `RINEX` file
#[derive(Debug)]
struct Rinex {
    header: header::Header,
//    body: Vec<T>, // body frames
}

#[derive(Error, Debug)]
enum RinexError {
    #[error("Header delimiter not found")]
    MissingHeaderDelimiter,
    #[error("Header parsing error")]
    HeaderError(#[from] header::HeaderError),
}

impl Rinex {
    /// Builds a Rinex struct
    pub fn new (header: header::Header) -> Rinex {
        Rinex {
            header
        }
    }

    /// grabs header content of given file
    fn parse_header_content (fp: &std::path::Path) -> Result<String, RinexError> {
        let content: String = std::fs::read_to_string(fp)
            .unwrap()
                .parse()
                .unwrap();
        if !content.contains(header::HEADER_END_MARKER) {
            return Err(RinexError::MissingHeaderDelimiter)
        }
        let parsed: Vec<&str> = content.split(header::HEADER_END_MARKER)
            .collect();
        Ok(parsed[0].to_string())
    }

    /// Builds a `Rinex` from given file.
    /// Input file must respect the whitespace specifications
    /// for the entire header section.   
    /// The header section must respect the labelization standard too.
    pub fn from_file (fp: &std::path::Path) -> Result<Rinex, RinexError> {
        let name = fp.file_name()
            .unwrap();
        let extension = fp.extension()
            .unwrap();
        let extension = extension.to_str()
            .unwrap();
        //TODO
        //TODO .s (summary files) not supported 
        // standard pour le nom est
        // ssssdddf.yyt
        // ssss: acronyme de la station
        // ddd: jour de l'annee du premier record
        // f: numero de la session dans le jour avec 0 pour une journee complete
        /*if !extension.eq("crx") && !extension.eq("rnx") {
            // crinex, could have a regex prior "."
            // decompressed crinex ?
            let convention_re = Regex::new(r"\d\d\d\d\.\d\d[o|O|g|G|i|I|d|D|s|S]$")
                .unwrap();
            if !convention_re.is_match(
                name.to_str()
                    .unwrap()) {
                return Err(RinexError::FileNamingConvention)
            }
        }*/

        // parse header
        let header = header::Header::from_str(&Rinex::parse_header_content(fp)?)?;
        // parse body 
        //let body = RinexBody::from_str(&Rinex::parse_body_content(fp)?)?;

        Ok(Rinex{
            header, 
        })
    }
}

mod test {
    use super::*;
/*
    /// tests Rcvr object fromStr method
    fn rcvr_from_str() {
        assert_eq!(
            Rcvr::from_str("82205               LEICA RS500         4.20/1.39  ")
                .unwrap(),
            Rcvr{
                sn: String::from("82205"),
                model: String::from("LEICA RS500"),
                firmware: String::from("4.20/1.39")
            });
    }
*/
    #[test]
    /// Test `Rinex` constructor
    /// against all valid data resources
    fn rinex_constructor() {
        // open test resources
        let test_resources = std::path::PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/data");
        // walk test resources
        for entry in std::fs::read_dir(test_resources)
            .unwrap() {
            let entry = entry
                .unwrap();
            let path = entry.path();
            if !path.is_dir() { // only files..
                let fp = std::path::Path::new(&path);
                let rinex = Rinex::from_file(&fp);
                assert_eq!(rinex.is_err(), false);
                println!("File: {:?}\n{:#?}", &fp, rinex)
            }
        }
    }
}
