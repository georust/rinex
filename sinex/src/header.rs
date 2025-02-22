use crate::bias;
use thiserror::Error;

use bias::header::Header as BiasHeader;

/// Returns true if given content matches a Header line
pub fn is_valid_header(line: &str) -> bool {
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
    fn from_str(content: &str) -> Result<Self, Self::Err> {
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
    BiasHeader(BiasHeader),
    // /// Tropospheric file header (TRO)
    // TropoHeader(troposphere::header::Header),
}

impl Default for Header {
    fn default() -> Self {
        Self::BiasHeader(Default::default())
    }
}

impl std::str::FromStr for Header {
    type Err = DocumentTypeError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
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
    pub fn bias_header(&self) -> Option<&BiasHeader> {
        match self {
            Self::BiasHeader(h) => Some(h),
        }
    }
}
