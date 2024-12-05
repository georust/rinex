//! Monument / Geodetic marker frames

use crate::{
    message::{
        time::{decode_gpst_epoch, encode_epoch, TimeResolution},
        Message,
    },
    Error,
};

use hifitime::{Epoch, TimeScale};
use std::str::from_utf8 as str_from_utf8;

mod fid;
mod src;

use fid::FieldID;

// public
pub use src::MonumentGeoMetadata;

#[derive(Debug, Clone, PartialEq)]
/// [GeoStringFrame] helps us encode / decode
/// readable [MonumentGeoRecord] entries, which makes
/// the vast majority of supported frames
pub struct GeoStringFrame {
    /// [FieldID] frame identifier
    pub(crate) fid: FieldID,
    /// Readable string
    pub string: String,
}

impl GeoStringFrame {
    pub fn new(fid: FieldID, s: &str) -> Self {
        Self {
            fid,
            string: s.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonumentGeoRecord {
    /// [Epoch]
    pub epoch: Epoch,
    /// Source of this information
    pub meta: MonumentGeoMetadata,
    /// Readable comments (if any), repeat as needed.
    pub comments: Vec<String>,
    /// Readable frames that we expose with high level methods
    pub frames: Vec<GeoStringFrame>,
}

impl Default for MonumentGeoRecord {
    fn default() -> Self {
        Self {
            epoch: Epoch::from_gpst_seconds(0.0),
            meta: MonumentGeoMetadata::RNX2BIN,
            comments: Default::default(),
            frames: Default::default(),
        }
    }
}

impl MonumentGeoRecord {
    /// 4 byte date uint4       }
    /// 1 byte qsec             } epoch
    /// 1 byte MonumentGeoMetadata
    /// 1 byte FID
    ///     if FID corresponds to a character string
    ///     the next 1-4 BNXI byte represent the number of bytes in the caracter string
    /// follows: FID dependent sequence. See [FieldID].
    const MIN_SIZE: usize = 5 + 1 + 1;

    /// Creates new [MonumentGeoRecord] with basic setup information.
    /// ```
    /// use binex::prelude::{
    ///     Epoch,
    ///     Error,
    ///     MonumentGeoRecord,
    ///     MonumentGeoMetadata,
    ///     Message,
    ///     Meta,
    ///     Record,
    /// };
    ///     
    /// // forge geodetic monument message
    /// let t = Epoch::from_gpst_seconds(60.0);
    ///
    /// let geo = MonumentGeoRecord::new(
    ///     t,
    ///     MonumentGeoMetadata::RNX2BIN,
    ///     "Fancy GNSS receiver",
    ///     "Magnificent antenna",
    ///     "SITE34", // Site name
    ///     "SITE34", // Site DOMES number
    /// );
    ///
    /// // customize as you need
    /// let geo = geo
    ///     .with_comment("you can add")
    ///     .with_comment("as many as you need")
    ///     .with_extra_info("Experiment or setup context")
    ///     .with_geophysical_info("Eurasian plate")
    ///     .with_climatic_info("Climatic Model XXXX");
    ///
    /// // Build Monument Geodetic message
    /// let geo = Record::new_monument_geo(geo);
    ///
    /// let meta = Meta {
    ///     reversed: false,
    ///     enhanced_crc: false,
    ///     big_endian: true,
    /// };
    ///
    /// let msg = Message::new(meta, geo);
    ///
    /// // encode
    /// let mut encoded = [0; 256];
    /// msg.encode(&mut encoded, msg.encoding_size()).unwrap();
    ///
    /// // decode
    /// let decoded = Message::decode(&encoded)
    ///     .unwrap();
    ///
    /// assert_eq!(decoded, msg);
    /// ```
    ///
    /// Another option is to use the Default constructor, but you have to be careful:
    /// - Epoch is set to t0 GPST, which is probably not what you intend
    /// - Message body is empty, so not ready to encode into a valid message.
    /// ```
    /// use binex::prelude::{
    ///     Epoch,
    ///     Error,
    ///     MonumentGeoRecord,
    ///     MonumentGeoMetadata,
    /// };
    ///     
    /// let t = Epoch::from_gpst_seconds(60.0 + 0.75);
    ///
    /// // pay attention that using defaults:
    /// //  - Epoch is GPST(t=0)
    /// //  - Default context setup!
    /// let mut record = MonumentGeoRecord::default()
    ///     .with_comment("Unknown context");
    ///
    /// record.epoch = t;
    /// record.meta = MonumentGeoMetadata::RNX2BIN;
    /// ```
    pub fn new(
        epoch: Epoch,
        meta: MonumentGeoMetadata,
        receiver_model: &str,
        antenna_model: &str,
        geodetic_marker_name: &str,
        geodetic_marker_number: &str,
    ) -> Self {
        let mut s = Self::default();
        s.epoch = epoch;
        s.meta = meta;
        s.frames
            .push(GeoStringFrame::new(FieldID::ReceiverType, receiver_model));
        s.frames
            .push(GeoStringFrame::new(FieldID::AntennaType, antenna_model));
        s.frames.push(GeoStringFrame::new(
            FieldID::MarkerName,
            geodetic_marker_name,
        ));
        s.frames.push(GeoStringFrame::new(
            FieldID::MarkerNumber,
            geodetic_marker_number,
        ));
        s
    }

    /// IGS data production special macro.
    /// Refer to [Self::new] for more information.
    pub fn new_igs(
        epoch: Epoch,
        receiver_model: &str,
        antenna_model: &str,
        geodetic_marker_name: &str,
        geodetic_marker_number: &str,
        site_location: &str,
        site_name: &str,
    ) -> Self {
        Self::new(
            epoch,
            MonumentGeoMetadata::IGS,
            receiver_model,
            antenna_model,
            geodetic_marker_name,
            geodetic_marker_number,
        )
        .with_agency("IGS")
        .with_site_location(site_location)
        .with_site_name(site_name)
    }

    /// New [MonumentGeoRecord] with emphasis that this
    /// results of a RINEX conversion (special context).
    /// Refer to [Self::new] for more information.
    pub fn new_rinex2bin(
        epoch: Epoch,
        receiver_model: &str,
        antenna_model: &str,
        geodetic_marker_name: &str,
        geodetic_marker_number: &str,
        agency: &str,
        site_location: &str,
        site_name: &str,
    ) -> Self {
        Self::new(
            epoch,
            MonumentGeoMetadata::RNX2BIN,
            receiver_model,
            antenna_model,
            geodetic_marker_name,
            geodetic_marker_number,
        )
        .with_agency(agency)
        .with_site_location(site_location)
        .with_site_name(site_name)
    }

    /// [Self] decoding attempt from buffered content.
    /// ## Inputs
    ///    - mlen: message length in bytes
    ///    - big_endian: endianness
    ///    - buf: buffered content
    /// ## Outputs
    ///    - Ok: [Self]
    ///    - Err: [Error]
    pub(crate) fn decode(mlen: usize, big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        let mut ret = Self::default();

        if mlen < Self::MIN_SIZE {
            // does not look good
            return Err(Error::NotEnoughBytes);
        }

        // decode timestamp
        ret.epoch = decode_gpst_epoch(big_endian, TimeResolution::QuarterSecond, buf)?;

        // decode source meta
        ret.meta = MonumentGeoMetadata::from(buf[5]);

        // parse inner frames
        let mut ptr = 6;

        while ptr < mlen {
            // decode Frame ID
            let (fid, size) = Message::decode_bnxi(&buf[ptr..], big_endian);
            let fid = FieldID::from(fid);
            ptr += size;

            if mlen < ptr + 1 {
                // cant decode 1-4b
                break;
            }

            // decode strlen
            let (s_len, size) = Message::decode_bnxi(&buf[ptr..], big_endian);
            let s_len = s_len as usize;
            ptr += size;

            // decode str: tolerance to bad utf8
            if let Ok(s) = str_from_utf8(&buf[ptr..ptr + s_len]) {
                // append geo frame
                match fid {
                    FieldID::Comments => {
                        ret.comments.push(s.to_string());
                    },
                    FieldID::AntennaEcef3D | FieldID::AntennaGeo3D | FieldID::AntennaOffset3D => {
                        // TODO: unhandled yet !
                    },
                    FieldID::Unknown => {
                        // bad ID: debug trace ?
                    },
                    _ => {
                        // reflect uniqueness of this information
                        // if stream badly encodes many of these, we only lacth the latest one
                        if let Some(fr) = ret
                            .frames
                            .iter_mut()
                            .filter(|fr| fr.fid == fid)
                            .reduce(|k, _| k)
                        {
                            fr.string = s.to_string(); // overwrite with latest
                        } else {
                            // store new readable element
                            ret.frames.push(GeoStringFrame::new(fid, s));
                        }
                    },
                }
            }
            ptr += s_len;
        }

        Ok(ret)
    }

    /// Encodes [MonumentGeoRecord] into buffer, returns encoded size (total bytes).
    /// [MonumentGeoRecord] must fit in preallocated buffer.
    // TODO: missing following fields
    //  - AntennaECEF3D
    //  - AntennaGeo3D
    //  - AntennaOffset3D
    pub(crate) fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.encoding_size();
        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        // encode tstamp
        let t = self.epoch.to_time_scale(TimeScale::GPST);
        let mut ptr = encode_epoch(t, TimeResolution::QuarterSecond, big_endian, buf)?;

        // encode source meta
        buf[ptr] = self.meta.into();
        ptr += 1;

        // encode all comments
        for comments in self.comments.iter() {
            // encode FID
            let fid = FieldID::Comments;
            let size = Message::encode_bnxi(fid as u32, big_endian, &mut buf[ptr..])?;
            ptr += size;

            // encode strlen
            let strlen = comments.len();
            let size = Message::encode_bnxi(strlen as u32, big_endian, &mut buf[ptr..])?;
            ptr += size;

            // encode str
            buf[ptr..ptr + strlen].clone_from_slice(comments.as_bytes());
            ptr += strlen;
        }

        // encode all geo string frames
        for fr in self.frames.iter() {
            // encode FID
            let size = Message::encode_bnxi(fr.fid as u32, big_endian, &mut buf[ptr..])?;
            ptr += size;

            // encode strlen
            let strlen = fr.string.len();
            let size = Message::encode_bnxi(strlen as u32, big_endian, &mut buf[ptr..])?;
            ptr += size;

            // encode str
            buf[ptr..ptr + strlen].clone_from_slice(fr.string.as_bytes());
            ptr += strlen;
        }

        Ok(size)
    }

    /// Returns total length (bytewise) required to fully encode [Self].
    /// Use this to fulfill [Self::encode] requirements.
    // TODO: missing following fields
    //  - AntennaECEF3D
    //  - AntennaGeo3D
    //  - AntennaOffset3D
    pub(crate) fn encoding_size(&self) -> usize {
        let mut size = 6; // tstamp + meta

        // for all comments:
        // 1-4 FID
        // 1-4 strlen
        // strlen
        for comment in self.comments.iter() {
            size += 1; // FID_1_4
            let s_len = comment.len();
            size += Message::bnxi_encoding_size(s_len as u32);
            size += s_len;
        }

        // for all encoded string:
        // 1-4 FID
        // 1-4 strlen
        // strlen
        for fr in self.frames.iter() {
            size += 1; // FID_1_4
            let s_len = fr.string.len();
            size += Message::bnxi_encoding_size(s_len as u32);
            size += s_len;
        }

        size
    }

    /// Add one readable comment to this [MonumentGeoRecord].
    /// You can add as many as you need.
    pub fn with_comment(&self, comment: &str) -> Self {
        let mut s = self.clone();
        s.comments.push(comment.to_string());
        s
    }

    // reflect uniqueness of this information
    // if stream badly encodes many of these, we only lacth the latest one
    fn push_or_update(&mut self, fid: FieldID, value: &str) {
        if let Some(fr) = self
            .frames
            .iter_mut()
            .filter(|fr| fr.fid == fid)
            .reduce(|k, _| k)
        {
            fr.string = value.to_string(); // overwrite / update
        } else {
            // store new readable element
            self.frames.push(GeoStringFrame::new(fid, value));
        }
    }

    /// Define software name
    pub fn with_software_name(&self, name: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::SoftwareName, name);
        s
    }

    /// Define receiver model.
    pub fn with_receiver_model(&self, model: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::ReceiverType, model);
        s
    }

    /// Define receiver serial number (if known).
    pub fn with_receiver_serial_number(&self, sn: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::ReceiverNumber, sn);
        s
    }

    /// Define receiver firmware version (if known).
    pub fn with_receiver_firmware_version(&self, version: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::ReceiverFirmwareVersion, version);
        s
    }

    /// Define name of observer
    pub fn with_observer(&self, observer: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::ObserverName, observer);
        s
    }

    /// Define observer's contact (email address)
    pub fn with_observer_contact(&self, contact: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::ObserverContact, contact);
        s
    }

    /// Define Geodetic marker name
    pub fn with_geodetic_marker_name(&self, name: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::MarkerName, name);
        s
    }

    /// Define Geodetic marker number (DOMES)
    pub fn with_geodetic_marker_number(&self, domes: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::MarkerNumber, domes);
        s
    }

    /// Define Locatio of this geodetic site
    pub fn with_site_location(&self, location: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::SiteLocation, location);
        s
    }

    /// Define Agency (Organization)
    pub fn with_agency(&self, agency: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::AgencyName, agency);
        s
    }

    /// Define Antenna model (if known)
    pub fn with_antenna_model(&self, model: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::AntennaType, model);
        s
    }

    /// Define Antenna serial number (if known)
    pub fn with_antenna_serial_number(&self, sn: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::AntennaNumber, sn);
        s
    }

    /// Define name of Geodetic site
    pub fn with_site_name(&self, name: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::SiteName, name);
        s
    }

    /// Attach readable Geophysical information (like local tectonic plate)
    /// to this [MonumentGeoRecord]. You can only add one per dataset
    /// otherwise, the message will not respect the standard definitions.
    pub fn with_geophysical_info(&self, info: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::Geophysical, info);
        s
    }

    /// Provide Climatic or Meteorological information (local to reference site).
    /// You can only add one per dataset otherwise,
    /// the message will not respect the standard definitions.
    pub fn with_climatic_info(&self, info: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::Climatic, info);
        s
    }

    /// Define a readable UserID to attach to this [MonumentGeoRecord] dataset.
    /// You can only add one per dataset otherwise,
    /// the message will not respect the standard definitions.
    pub fn with_user_id(&self, userid: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::UserID, userid);
        s
    }

    /// Provide the name of this Geodetic project (if any).
    pub fn with_project_name(&self, name: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::ProjectName, name);
        s
    }

    /// Add one extra note (like concise description of context
    /// or experiment)
    pub fn with_extra_info(&self, extra: &str) -> Self {
        let mut s = self.clone();
        s.push_or_update(FieldID::Extra, extra);
        s
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn monument_marker_bnx00_error() {
        let buf = [0, 0, 0, 0];
        let monument = MonumentGeoRecord::decode(4, true, &buf);
        assert!(monument.is_err());
    }

    #[test]
    fn monument_geo_comments_decoding() {
        let mlen = 17;
        for big_endian in [true, false] {
            let mut geo = MonumentGeoRecord::default();
            geo.epoch = Epoch::from_gpst_seconds(256.0 * 60.0 + 60.0 + 10.25);
            geo = geo.with_comment("Hello World");
            geo.meta = MonumentGeoMetadata::IGS;

            let mut encoded = [0; 64];
            geo.encode(big_endian, &mut encoded).unwrap();
            let decoded = MonumentGeoRecord::decode(mlen, big_endian, &encoded).unwrap();
            assert_eq!(decoded, geo);
        }
    }

    #[test]
    fn monument_geo_double_comments_decoding() {
        for big_endian in [true, false] {
            let mut geo: MonumentGeoRecord = MonumentGeoRecord::default()
                .with_comment("A B C")
                .with_comment("D E F");

            geo.epoch = Epoch::from_gpst_seconds(60.0 + 0.75);
            geo.meta = MonumentGeoMetadata::IGS;

            // ts + meta + 2*(FID_1_4 +STR_1_4 +STR)
            assert_eq!(geo.encoding_size(), 5 + 1 + 2 * (1 + 1 + 5));

            let mut buf = [0; 16];
            assert!(geo.encode(big_endian, &mut buf).is_err());

            let mut buf = [0; 22];
            let size = geo.encode(big_endian, &mut buf).unwrap();

            // ts + meta + 2*(FID_1_4 +strlen_1_4 +strlen)
            assert_eq!(geo.encoding_size(), 5 + 1 + 2 * (1 + 1 + 5));
            assert_eq!(size, 5 + 1 + 2 * (1 + 1 + 5));

            let mut geo = MonumentGeoRecord::default()
                .with_comment("Hello")
                .with_climatic_info("Clim");

            geo.epoch = Epoch::from_gpst_seconds(60.0 + 0.75);
            geo.meta = MonumentGeoMetadata::IGS;

            let mut buf = [0; 19];
            let size = geo.encode(big_endian, &mut buf).unwrap();
            assert_eq!(size, 5 + 1 + 1 + 1 + 5 + 1 + 1 + 4);

            if big_endian {
                assert_eq!(
                    buf,
                    [
                        0,
                        0,
                        0,
                        1,
                        3,
                        MonumentGeoMetadata::IGS as u8,
                        0,
                        5,
                        b'H',
                        b'e',
                        b'l',
                        b'l',
                        b'o',
                        14,
                        4,
                        b'C',
                        b'l',
                        b'i',
                        b'm',
                    ]
                );
            }
        }
    }
}
