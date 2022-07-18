use crate::bias;
use thiserror::Error;

/// Returns true if given content matches a Header line
pub fn is_valid_header (line: &str) -> bool {
    line.starts_with("%=")
}

#[derive(Debug, Clone)]
pub enum DocumentType {
    /// Bias Solutions
    BiasSolutions,
    /// Troposphere Coordinates
    TropoCoordinates,
}

impl Default for DocumentType {
    fn default() -> Self {
        Self::BiasSolutions
    }
}

#[derive(Debug, Error, Clone)]
pub enum DocumentTypeError {
    /// Sinex file type not recognized
    #[error("unknown file type \"{0}\"")]
    UnknownDocumentType(String),
}

impl std::str::FromStr for DocumentType {
    type Err = DocumentTypeError;
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        if content.eq("TRO") {
            Ok(Self::TropoCoordinates)
        } else if content.eq("BIA") {
            Ok(Self::BiasSolutions)
        } else {
            Err(DocumentTypeError::UnknownDocumentType(content.to_string()))
        }
    }
}

/// Header is Document Type dependent
#[derive(Debug, Clone)]
pub enum Header {
    /// Bias solutions header (BIA)
    BiasHeader(bias::header::Header),
    // /// Tropospheric file header (TRO)
    // TropoHeader(troposphere::header::Header),
}

impl Default for Header {
    fn default() -> Self {
        Self::BiasHeader(bias::header::Header::default())
    }
}

impl std::str::FromStr for Header {
    type Err = DocumentTypeError;
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        if let Ok(hd) = bias::header::Header::from_str(content) {
            Ok(Self::BiasHeader(hd))
        //} else if let Ok(hd) = troposphere::Header::from_str(content) {
        //    Ok(Self::TropoHeader(hd))
        } else {
            Err(DocumentTypeError::UnknownDocumentType(content.to_string()))
        }
    }
}

impl Header {
    pub fn bias_header (&self) -> Option<&bias::header::Header> {
        match self {
            Self::BiasHeader(h) => Some(h),
            _ => None,
        }
    }
    /*pub fn tropo_header (&self) -> Option<&troposhere::header::Header> {
        match self {
            Self::TropoHeader(h) => Some(h),
            _ => None,
        }
    }*/
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_header() {
        let content = "%=BIA";
        let header = Header::from_str(content);
        assert_eq!(header.is_ok(), true);
        let header = header.unwrap();
    }
}
