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
    /// Tropospheric file header (TRO)
    TropoHeader(troposphere::header::Header),
}

impl Header {
    pub fn bias_header (&self) -> Option<bias::header::Header> {
        match self {
            Self::BiasHeader(h) => Some(h),
            _ => None,
        }
    }
    pub fn tropo_header (&self) -> Option<troposhere::header::Header> {
        match self {
            Self::TroposphereHeader(h) => Some(h),
            _ => None,
        }
    }
}
