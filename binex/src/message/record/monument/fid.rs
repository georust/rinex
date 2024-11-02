//! Monument / Geodetic Field ID

/// [FieldID] describes the content to follow
/// in Geodetic marker frames
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldID {
    /// Comment: simple comment (readable string)
    /// about the Geodetic marker. Several RINEX comments
    /// are described by several BINEX Geodetic comments (repeated frames).
    Comments = 0,
    /// Software (=Program) name used in the creation of this BINEX Geodetic Record.   
    /// Must be unique in any BINEX Geodetic Record. Field length (bytewise) must follow
    SoftwareName = 1,
    /// Operator (=RunBy) name who created this BINEX Geodetic Record.  
    /// Must be unique, requires field length (bytewise)
    OperatorName = 2,
    /// Country / State / Province / City location of the data producer.  
    /// Must be unique, requires field length (bytewise).
    SiteLocation = 3,
    /// Site Agency name (=MARKER NAME).  
    /// Must be unique, requires field length (bytewise).
    SiteName = 4,
    /// Site Agency number.  
    /// Must be unique, requires field length (bytewise).
    SiteNumber = 5,
    /// Monument name and description.  
    /// Must be unique, requires field length (bytewise).
    MonumentName = 6,
    /// Monument number (=MARKER NUMBER).   
    /// Must be unique, requires field length (bytewise).
    MonumentNumber = 7,
    /// Marker name and description.  
    /// Must be unique, requires field length (bytewise).
    MarkerName = 8,
    /// Marker number (=MARKER NUMBER).  
    /// Must be unique, requires field length (bytewise).
    MarkerNumber = 9,
    /// Name for the Reference Coordinates.  
    /// Must be unique, requires field length (bytewise).
    ReferenceName = 10,
    /// Official Number (=DOMES) (=MARKER NUMBER) for the Reference Coordinates.  
    /// Must be unique, requires field length (bytewise).
    ReferenceNumber = 11,
    /// Date of the coordinates determination and marker installation.  
    /// Must be unique. Follows:
    ///   * number of ascii bytes
    ///   * ascii date description
    ///   * year (sint2)
    ///   * minutes into year (uint4)
    /// Must be unique.
    ReferenceDate = 12,
    /// Site geologic / geophyiscal characteristics
    /// (for example: tectonic plate of this site).
    Geophysical = 13,
    /// Climatic (=gross meteorological) local profile.
    Climatic = 14,
    /// Custom User defined 4 character ID associated to this
    /// data & metadata. Must always be 4 byte long (fill with space).
    /// Must be unique.
    UserID = 15,
    /// Project Name / description. Must be unique.
    ProjectName = 16,
    /// Observer (=OBSERVER), sometimes refered to as "Investigator".
    /// Several entities or people can be described: repeat as need be.
    ObserverName = 17,
    /// Agency Name (entity/employer) (=OBSERVER AGENCY).
    /// Must be unique.
    AgencyName = 18,
    /// Observer Contact. Repeat as need be.
    ObserverContact = 19,
    /// Site Operator (=OBSERVER). Must be unique.
    SiteOperator = 20,
    /// Site Operator Agency (=OBSERVER AGENCY). Must be unique.
    SiteOperatorAgency = 21,
    /// Site Operator Contact. Must be unique.
    SiteOperatorContact = 22,
    /// Antenna Type (=ANTENNA TYPE). Must be unique.
    AntennaType = 23,
    /// Antenna Number (=ANTENNA #). Must be unique
    AntennaNumber = 24,
    /// Receiver Type (=RECEIVER TYPE). Must be unique.
    ReceiverType = 25,
    /// Receiver Number (=RECEIVER #). Must be unique.
    ReceiverNumber = 26,
    /// Receiver Firmware Version (=RECEIVER VERS). Must be unique.
    ReceiverFirmwareVersion = 27,
    /// Antenna mount description. Must be unique.
    AntennaMount = 28,
    /// Antenna ECEF X/Y/Z coordinates (=APPROX POSITION XYZ), follows:
    ///   * ubnxi number of bytes in ECEF/ellipsoid model (may be 0)
    ///   * ECEF/ellipsoid model description. (When 0: WGS84 is assumed).
    ///   * ECEF(x) [m] (real8)
    ///   * ECEF(y) [m] (real8)
    ///   * ECEF(z) [m] (real8)
    /// Must be unique
    AntennaEcef3D = 29,
    /// Antenna Geographic Position (Geo. Coordinates). Follows:
    ///   * ubnxi number of bytes in ECEF/ellipsoid model (may be 0)
    ///   * ECEF/ellipsoid model description. (When 0: WGS84 assumed)
    ///   * East/West longitude [ddeg] (real8)
    ///   * North/South latitude [ddeg] (real8)
    ///   * Elevation [m] (real8)
    /// Must be unique
    AntennaGeo3D = 30,
    /// Antenna offset from reference point (= ANTENNA DELTA H/E/N). Follows:
    ///   * Height offset [m] (real8)
    ///   * East/West offset [m] (real8)
    ///   * North/South offset [m] (real8)
    AntennaOffset3D = 31,
    /// Antenna Radome Type (=TYPE). Must be unique.
    AntennaRadomeType = 32,
    /// Antenna Radom Number. Must be unique.
    AntennaRadomeNumber = 33,
    /// Geocode. Must be unique.
    Geocode = 34,
    /// Extra / Additional information, very similar to [Self::Comment]
    Extra = 127,
    /// Unknown / Invalid
    Unknown = 0xffffffff,
}

impl From<u32> for FieldID {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::Comments,
            1 => Self::SoftwareName,
            2 => Self::OperatorName,
            3 => Self::SiteLocation,
            4 => Self::SiteName,
            5 => Self::SiteNumber,
            6 => Self::MonumentName,
            7 => Self::MonumentNumber,
            8 => Self::MarkerName,
            9 => Self::MarkerNumber,
            10 => Self::ReferenceName,
            11 => Self::ReferenceNumber,
            12 => Self::ReferenceDate,
            13 => Self::Geophysical,
            14 => Self::Climatic,
            15 => Self::UserID,
            16 => Self::ProjectName,
            17 => Self::ObserverName,
            18 => Self::AgencyName,
            19 => Self::ObserverContact,
            20 => Self::SiteOperator,
            21 => Self::SiteOperatorAgency,
            22 => Self::SiteOperatorContact,
            23 => Self::AntennaType,
            24 => Self::AntennaNumber,
            25 => Self::ReceiverType,
            26 => Self::ReceiverNumber,
            27 => Self::ReceiverFirmwareVersion,
            28 => Self::AntennaMount,
            29 => Self::AntennaEcef3D,
            30 => Self::AntennaGeo3D,
            31 => Self::AntennaOffset3D,
            32 => Self::AntennaRadomeType,
            33 => Self::AntennaRadomeNumber,
            34 => Self::Geocode,
            127 => Self::Extra,
            _ => Self::Unknown,
        }
    }
}

impl From<FieldID> for u32 {
    fn from(val: FieldID) -> u32 {
        match val {
            FieldID::Comments => 0,
            FieldID::SoftwareName => 1,
            FieldID::OperatorName => 2,
            FieldID::SiteLocation => 3,
            FieldID::SiteName => 4,
            FieldID::SiteNumber => 5,
            FieldID::MonumentName => 6,
            FieldID::MonumentNumber => 7,
            FieldID::MarkerName => 8,
            FieldID::MarkerNumber => 9,
            FieldID::ReferenceName => 10,
            FieldID::ReferenceNumber => 11,
            FieldID::ReferenceDate => 12,
            FieldID::Geophysical => 13,
            FieldID::Climatic => 14,
            FieldID::UserID => 15,
            FieldID::ProjectName => 16,
            FieldID::ObserverName => 17,
            FieldID::AgencyName => 18,
            FieldID::ObserverContact => 19,
            FieldID::SiteOperator => 20,
            FieldID::SiteOperatorAgency => 21,
            FieldID::SiteOperatorContact => 22,
            FieldID::AntennaType => 23,
            FieldID::AntennaNumber => 24,
            FieldID::ReceiverType => 25,
            FieldID::ReceiverNumber => 26,
            FieldID::ReceiverFirmwareVersion => 27,
            FieldID::AntennaMount => 28,
            FieldID::AntennaEcef3D => 29,
            FieldID::AntennaGeo3D => 30,
            FieldID::AntennaOffset3D => 31,
            FieldID::AntennaRadomeType => 32,
            FieldID::AntennaRadomeNumber => 33,
            FieldID::Geocode => 34,
            FieldID::Extra => 127,
            FieldID::Unknown => 0xffffffff,
        }
    }
}
