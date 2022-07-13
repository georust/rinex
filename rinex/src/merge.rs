//! `merging` operations related definitions 
use thiserror::Error;

#[derive(Error, Debug)]
/// `RINEX` merge ops related errors
pub enum MergeError {
    /// Type Mismatch: it is not possible to 
    /// merged different kinds of RINEX toghether
    #[error("file types mismatch: cannot merge different `rinex`")]
    FileTypeMismatch,
}

#[derive(Clone, Debug)]
/// `RINEX` merging options
pub struct MergeOpts {
    /// optionnal program name
    pub program: String,
    /// timestamp where new file was appended
    pub date: chrono::NaiveDateTime, 
}

/*impl std::str::FromStr for MergeOpts {
    Err = MergeError;
    /// Builds MergeOpts structure from "standard" RINEX comment line
    fn from_str (line: &str) -> Result<Self, Self::Err> {
        let (program, rem) = line.split_at(20);
        let (ops, rem) = rem.split_at(20);
        let (date, _) = rem.split_at(20);
        if !opts.trim().eq("FILE MERGE") {
            return Err(MergeError::MergeOptsDescriptionMismatch)
        }
        MergeOpts {
            program: program.trim().to_string(),
            date : chrono::DateTime::parse_from_str(date.split_at(16).0, "%Y%m%d %h%m%s")?, 
        }
    }
}*/
