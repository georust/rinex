//! Monument Geodetic marker specific frames
use crate::{
    message::{record::monument::FieldID, Message},
    Error,
};

use core::str::from_utf8;

// use log::error;

#[derive(Debug, Clone, PartialEq)]
pub enum MonumentGeoFrame<'a> {
    /// Comment
    Comment(&'a str),
    /// Software (Program) name
    SoftwareName(&'a str),
    /// Agency Name
    AgencyName(&'a str),
    /// Name of person or entity operating [MonumentGeoFrame::SoftwareName]
    /// employed by [MonumentGeoFrame::AgencyName].
    OperatorName(&'a str),
    /// Site Location
    SiteLocation(&'a str),
    /// Site Number
    SiteNumber(&'a str),
    /// Site name
    SiteName(&'a str),
    /// Site Operator
    SiteOperator(&'a str),
    /// Site Operator Contact
    SiteOperatorContact(&'a str),
    /// Site Operator Agency
    SiteOperatorAgency(&'a str),
    /// Observer Name
    ObserverName(&'a str),
    /// Observer Contact
    ObserverContact(&'a str),
    /// Geodetic Marker Name
    MarkerName(&'a str),
    /// Geodetic Monument Name
    MonumentName(&'a str),
    /// Geodetic Monument Number
    MonumentNumber(&'a str),
    /// Geodetic Marker Number (DOMES)
    MarkerNumber(&'a str),
    /// Project Name
    ProjectName(&'a str),
    /// Reference Name
    ReferenceName(&'a str),
    /// Reference Date
    ReferenceDate(&'a str),
    /// Reference Number
    ReferenceNumber(&'a str),
    /// Local meteorological model/information at site location
    Climatic(&'a str),
    /// Geophysical information at site location (like tectonic plate)
    Geophysical(&'a str),
    /// Antenna Type
    AntennaType(&'a str),
    /// Antenna Radome Type
    AntennaRadomeType(&'a str),
    /// Antenna Mount information
    AntennaMount(&'a str),
    /// Antenna Number
    AntennaNumber(&'a str),
    /// Antenna Radome Number
    AntennaRadomeNumber(&'a str),
    /// Receiver Firmware Version
    ReceiverFirmwareVersion(&'a str),
    /// Receiver Type
    ReceiverType(&'a str),
    /// Receiver (Serial) Number
    ReceiverNumber(&'a str),
    /// User defined ID
    UserID(&'a str),
    /// Extra information about production site
    Extra(&'a str),
}

impl<'a> MonumentGeoFrame<'a> {
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

                match from_utf8(&buf[ptr..ptr + s_len]) {
                    Ok(s) => match fid {
                        FieldID::Comment => Ok(Self::Comment(s)),
                        FieldID::MonumentName => Ok(Self::MonumentName(s)),
                        FieldID::MonumentNumber => Ok(Self::MonumentNumber(s)),
                        FieldID::ProjectName => Ok(Self::ProjectName(s)),
                        FieldID::ObserverName => Ok(Self::ObserverName(s)),
                        FieldID::ObserverContact => Ok(Self::ObserverContact(s)),
                        FieldID::SoftwareName => Ok(Self::SoftwareName(s)),
                        FieldID::MarkerName => Ok(Self::MarkerName(s)),
                        FieldID::MarkerNumber => Ok(Self::MarkerNumber(s)),
                        FieldID::Extra => Ok(Self::Extra(s)),
                        FieldID::Climatic => Ok(Self::Climatic(s)),
                        FieldID::Geophysical => Ok(Self::Geophysical(s)),
                        FieldID::AgencyName => Ok(Self::AgencyName(s)),
                        FieldID::AntennaType => Ok(Self::AntennaType(s)),
                        FieldID::AntennaMount => Ok(Self::AntennaMount(s)),
                        FieldID::AntennaNumber => Ok(Self::AntennaNumber(s)),
                        FieldID::AntennaRadomeType => Ok(Self::AntennaRadomeType(s)),
                        FieldID::AntennaRadomeNumber => Ok(Self::AntennaRadomeNumber(s)),
                        FieldID::ReceiverFirmwareVersion => Ok(Self::ReceiverFirmwareVersion(s)),
                        FieldID::ReceiverNumber => Ok(Self::ReceiverNumber(s)),
                        FieldID::ReceiverType => Ok(Self::ReceiverType(s)),
                        FieldID::OperatorName => Ok(Self::OperatorName(s)),
                        FieldID::SiteLocation => Ok(Self::SiteLocation(s)),
                        FieldID::SiteName => Ok(Self::SiteName(s)),
                        FieldID::SiteNumber => Ok(Self::SiteNumber(s)),
                        FieldID::ReferenceDate => Ok(Self::ReferenceDate(s)),
                        FieldID::ReferenceName => Ok(Self::ReferenceName(s)),
                        FieldID::ReferenceNumber => Ok(Self::ReferenceNumber(s)),
                        FieldID::UserID => Ok(Self::UserID(s)),
                        FieldID::SiteOperator => Ok(Self::SiteOperator(s)),
                        FieldID::SiteOperatorAgency => Ok(Self::SiteOperatorAgency(s)),
                        FieldID::SiteOperatorContact => Ok(Self::SiteOperatorContact(s)),
                        // TODO
                        FieldID::AntennaEcef3D
                        | FieldID::Geocode
                        | FieldID::AntennaOffset3D
                        | FieldID::AntennaGeo3D
                        | FieldID::Unknown => Err(Error::NonSupportedMesssage(24)),
                    },
                    Err(_) => {
                        // println!("bnx00-str: utf8 error {}", e);
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
    #[test]
    fn geo_climatic() {
        let frame = MonumentGeoFrame::Climatic("ABC".to_string());
        assert_eq!(frame.encoding_size(), 3 + 2);

        let big_endian = true;
        let mut buf = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let size = frame.encode(big_endian, &mut buf).unwrap();

        assert_eq!(size, frame.encoding_size());
        assert_eq!(
            buf,
            [14, 3, 'A' as u8, 'B' as u8, 'C' as u8, 0, 0, 0, 0, 0, 0]
        );

        let decoded = MonumentGeoFrame::decode(big_endian, &buf).unwrap();

        assert_eq!(decoded, frame);
    }
}
