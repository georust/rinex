//! Monument Geodetic marker specific frames

use crate::{
    message::{record::monument::FieldID, Message},
    Error,
};

// use log::error;

#[derive(Debug, Clone, PartialEq)]
pub enum MonumentGeoFrame {
    /// Comment
    Comment(String),
    /// Software (Program) name
    SoftwareName(String),
    /// Agency Name
    AgencyName(String),
    /// Name of person or entity operating [MonumentGeoFrame::SoftwareName]
    /// employed by [MonumentGeoFrame::AgencyName].
    OperatorName(String),
    /// Site Location
    SiteLocation(String),
    /// Site Number
    SiteNumber(String),
    /// Site name
    SiteName(String),
    /// Site Operator
    SiteOperator(String),
    /// Site Operator Contact
    SiteOperatorContact(String),
    /// Site Operator Agency
    SiteOperatorAgency(String),
    /// Observer Name
    ObserverName(String),
    /// Observer Contact
    ObserverContact(String),
    /// Geodetic Marker Name
    MarkerName(String),
    /// Geodetic Monument Name
    MonumentName(String),
    /// Geodetic Monument Number
    MonumentNumber(String),
    /// Geodetic Marker Number (DOMES)
    MarkerNumber(String),
    /// Project Name
    ProjectName(String),
    /// Reference Name
    ReferenceName(String),
    /// Reference Date
    ReferenceDate(String),
    /// Reference Number
    ReferenceNumber(String),
    /// Local meteorological model/information at site location
    Climatic(String),
    /// Geophysical information at site location (like tectonic plate)
    Geophysical(String),
    /// Antenna Type
    AntennaType(String),
    /// Antenna Radome Type
    AntennaRadomeType(String),
    /// Antenna Mount information
    AntennaMount(String),
    /// Antenna Number
    AntennaNumber(String),
    /// Antenna Radome Number
    AntennaRadomeNumber(String),
    /// Receiver Firmware Version
    ReceiverFirmwareVersion(String),
    /// Receiver Type
    ReceiverType(String),
    /// Receiver (Serial) Number
    ReceiverNumber(String),
    /// User defined ID
    UserID(String),
    /// Extra information about production site
    Extra(String),
}

impl MonumentGeoFrame {
    /// Returns total length (bytewise) required to fully encode [Self].
    /// Use this to fulfill [Self::encode] requirements.
    pub(crate) fn encoding_size(&self) -> usize {
        match self {
            Self::Comment(s)
            | Self::ReferenceDate(s)
            | Self::ReferenceName(s)
            | Self::ReferenceNumber(s)
            | Self::SiteNumber(s)
            | Self::SiteOperator(s)
            | Self::SiteOperatorAgency(s)
            | Self::SiteOperatorContact(s)
            | Self::Extra(s)
            | Self::SiteLocation(s)
            | Self::SiteName(s)
            | Self::ReceiverFirmwareVersion(s)
            | Self::ReceiverNumber(s)
            | Self::ReceiverType(s)
            | Self::ObserverContact(s)
            | Self::ObserverName(s)
            | Self::MonumentName(s)
            | Self::MonumentNumber(s)
            | Self::ProjectName(s)
            | Self::MarkerName(s)
            | Self::MarkerNumber(s)
            | Self::SoftwareName(s)
            | Self::Geophysical(s)
            | Self::Climatic(s)
            | Self::AntennaType(s)
            | Self::AntennaMount(s)
            | Self::AntennaRadomeType(s)
            | Self::AntennaRadomeNumber(s)
            | Self::AntennaNumber(s)
            | Self::OperatorName(s)
            | Self::UserID(s)
            | Self::AgencyName(s) => {
                let s_len = s.len();
                s_len + 1 + Message::bnxi_encoding_size(s_len as u32) // FID
            },
        }
    }

    /// Returns expected [FieldID] for [Self]
    pub(crate) fn to_field_id(&self) -> FieldID {
        match self {
            Self::Comment(_) => FieldID::Comment,
            Self::OperatorName(_) => FieldID::OperatorName,
            Self::SiteLocation(_) => FieldID::SiteLocation,
            Self::SiteOperator(_) => FieldID::SiteOperator,
            Self::SiteOperatorAgency(_) => FieldID::SiteOperatorAgency,
            Self::SiteOperatorContact(_) => FieldID::SiteOperatorContact,
            Self::SiteName(_) => FieldID::SiteName,
            Self::MonumentName(_) => FieldID::MonumentName,
            Self::MonumentNumber(_) => FieldID::MonumentNumber,
            Self::ProjectName(_) => FieldID::ProjectName,
            Self::MarkerName(_) => FieldID::MarkerName,
            Self::MarkerNumber(_) => FieldID::MarkerNumber,
            Self::ObserverContact(_) => FieldID::ObserverContact,
            Self::ObserverName(_) => FieldID::ObserverName,
            Self::Extra(_) => FieldID::Extra,
            Self::UserID(_) => FieldID::UserID,
            Self::Climatic(_) => FieldID::Climatic,
            Self::Geophysical(_) => FieldID::Geophysical,
            Self::SoftwareName(_) => FieldID::SoftwareName,
            Self::AgencyName(_) => FieldID::AgencyName,
            Self::AntennaType(_) => FieldID::AntennaType,
            Self::AntennaMount(_) => FieldID::AntennaMount,
            Self::AntennaNumber(_) => FieldID::AntennaNumber,
            Self::AntennaRadomeType(_) => FieldID::AntennaRadomeType,
            Self::AntennaRadomeNumber(_) => FieldID::AntennaRadomeNumber,
            Self::ReceiverFirmwareVersion(_) => FieldID::ReceiverFirmwareVersion,
            Self::ReceiverNumber(_) => FieldID::ReceiverNumber,
            Self::ReceiverType(_) => FieldID::ReceiverType,
            Self::SiteNumber(_) => FieldID::SiteNumber,
            Self::ReferenceDate(_) => FieldID::ReferenceDate,
            Self::ReferenceName(_) => FieldID::ReferenceName,
            Self::ReferenceNumber(_) => FieldID::ReferenceNumber,
        }
    }

    /// [MonumentGeoFrame] decoding attempt from given [FieldID]
    pub(crate) fn decode(big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        if buf.len() < 2 {
            // smallest size
            return Err(Error::NotEnoughBytes);
        }

        // decode FID
        let (fid, mut ptr) = Message::decode_bnxi(&buf, big_endian);
        let fid = FieldID::from(fid);

        match fid {
            FieldID::Comment
            | FieldID::AntennaNumber
            | FieldID::AntennaType
            | FieldID::AntennaMount
            | FieldID::AntennaRadomeNumber
            | FieldID::AntennaRadomeType
            | FieldID::AgencyName
            | FieldID::Climatic
            | FieldID::Geophysical
            | FieldID::MonumentName
            | FieldID::MonumentNumber
            | FieldID::MarkerName
            | FieldID::MarkerNumber
            | FieldID::ObserverContact
            | FieldID::ObserverName
            | FieldID::ProjectName
            | FieldID::SiteLocation
            | FieldID::ReceiverFirmwareVersion
            | FieldID::ReceiverType
            | FieldID::ReceiverNumber
            | FieldID::Extra => {
                // can't decode 1-4b
                if buf.len() < 1 + ptr {
                    return Err(Error::NotEnoughBytes);
                }

                // decode slen
                let (s_len, size) = Message::decode_bnxi(&buf[ptr..], big_endian);
                let s_len = s_len as usize;
                ptr += size;

                if buf.len() - ptr < s_len {
                    return Err(Error::NotEnoughBytes); // can't parse entire string
                }

                match std::str::from_utf8(&buf[ptr..ptr + s_len]) {
                    Ok(s) => match fid {
                        FieldID::Comment => Ok(Self::Comment(s.to_string())),
                        FieldID::MonumentName => Ok(Self::MonumentName(s.to_string())),
                        FieldID::MonumentNumber => Ok(Self::MonumentNumber(s.to_string())),
                        FieldID::ProjectName => Ok(Self::ProjectName(s.to_string())),
                        FieldID::ObserverName => Ok(Self::ObserverName(s.to_string())),
                        FieldID::ObserverContact => Ok(Self::ObserverContact(s.to_string())),
                        FieldID::SoftwareName => Ok(Self::SoftwareName(s.to_string())),
                        FieldID::MarkerName => Ok(Self::MarkerName(s.to_string())),
                        FieldID::MarkerNumber => Ok(Self::MarkerNumber(s.to_string())),
                        FieldID::Extra => Ok(Self::Extra(s.to_string())),
                        FieldID::Climatic => Ok(Self::Climatic(s.to_string())),
                        FieldID::Geophysical => Ok(Self::Geophysical(s.to_string())),
                        FieldID::AgencyName => Ok(Self::AgencyName(s.to_string())),
                        FieldID::AntennaType => Ok(Self::AntennaType(s.to_string())),
                        FieldID::AntennaMount => Ok(Self::AntennaMount(s.to_string())),
                        FieldID::AntennaNumber => Ok(Self::AntennaNumber(s.to_string())),
                        FieldID::AntennaRadomeType => Ok(Self::AntennaRadomeType(s.to_string())),
                        FieldID::AntennaRadomeNumber => {
                            Ok(Self::AntennaRadomeNumber(s.to_string()))
                        },
                        FieldID::ReceiverFirmwareVersion => {
                            Ok(Self::ReceiverFirmwareVersion(s.to_string()))
                        },
                        FieldID::ReceiverNumber => Ok(Self::ReceiverNumber(s.to_string())),
                        FieldID::ReceiverType => Ok(Self::ReceiverType(s.to_string())),
                        FieldID::OperatorName => Ok(Self::OperatorName(s.to_string())),
                        FieldID::SiteLocation => Ok(Self::SiteLocation(s.to_string())),
                        FieldID::SiteName => Ok(Self::SiteName(s.to_string())),
                        FieldID::SiteNumber => Ok(Self::SiteNumber(s.to_string())),
                        FieldID::ReferenceDate => Ok(Self::ReferenceDate(s.to_string())),
                        FieldID::ReferenceName => Ok(Self::ReferenceName(s.to_string())),
                        FieldID::ReferenceNumber => Ok(Self::ReferenceNumber(s.to_string())),
                        FieldID::UserID => Ok(Self::UserID(s.to_string())),
                        FieldID::SiteOperator => Ok(Self::SiteOperator(s.to_string())),
                        FieldID::SiteOperatorAgency => Ok(Self::SiteOperatorAgency(s.to_string())),
                        FieldID::SiteOperatorContact => {
                            Ok(Self::SiteOperatorContact(s.to_string()))
                        },
                        // TODO
                        FieldID::AntennaEcef3D
                        | FieldID::Geocode
                        | FieldID::AntennaOffset3D
                        | FieldID::AntennaGeo3D
                        | FieldID::Unknown => Err(Error::UnknownMessage),
                    },
                    Err(e) => {
                        println!("bnx00-str: utf8 error {}", e);
                        Err(Error::Utf8Error)
                    },
                }
            },
            _ => Err(Error::UnknownRecordFieldId),
        }
    }

    /// Encodes [Self] into buffer, returns encoded size (total bytes).
    /// [Self] must fit in preallocated buffer.
    pub fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.encoding_size();
        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        // encode FID
        let fid = self.to_field_id() as u32;
        let mut ptr = Message::encode_bnxi(fid, big_endian, buf)?;

        match self {
            Self::Comment(s)
            | Self::UserID(s)
            | Self::SiteOperator(s)
            | Self::SiteOperatorAgency(s)
            | Self::OperatorName(s)
            | Self::SiteLocation(s)
            | Self::SiteOperatorContact(s)
            | Self::SiteNumber(s)
            | Self::ObserverName(s)
            | Self::ProjectName(s)
            | Self::ReferenceName(s)
            | Self::MonumentNumber(s)
            | Self::ReferenceDate(s)
            | Self::ReferenceNumber(s)
            | Self::ObserverContact(s)
            | Self::MonumentName(s)
            | Self::SiteName(s)
            | Self::Extra(s)
            | Self::SoftwareName(s)
            | Self::Climatic(s)
            | Self::Geophysical(s)
            | Self::AgencyName(s)
            | Self::MarkerName(s)
            | Self::MarkerNumber(s)
            | Self::ReceiverFirmwareVersion(s)
            | Self::ReceiverNumber(s)
            | Self::ReceiverType(s)
            | Self::AntennaType(s)
            | Self::AntennaNumber(s)
            | Self::AntennaMount(s)
            | Self::AntennaRadomeType(s)
            | Self::AntennaRadomeNumber(s) => {
                // encode strlen
                let s_len = s.len();
                let size = Message::encode_bnxi(s_len as u32, big_endian, &mut buf[ptr..])?;
                ptr += size;

                buf[ptr..ptr + s_len].clone_from_slice(s.as_bytes()); // utf8 encoding
            },
        }

        Ok(size)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn geo_comments() {
        let frame = MonumentGeoFrame::Comment("Hello".to_string());
        assert_eq!(frame.encoding_size(), 5 + 2);

        let big_endian = true;
        let mut buf = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let size = frame.encode(big_endian, &mut buf).unwrap();

        assert_eq!(size, frame.encoding_size());
        assert_eq!(
            buf,
            [0, 5, 'H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0, 0, 0, 0, 0, 0]
        );

        let decoded = MonumentGeoFrame::decode(big_endian, &buf).unwrap();

        assert_eq!(decoded, frame);
    }
}
