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

// private
use fid::FieldID;

// public
pub use src::MonumentGeoMetadata;

#[derive(Debug, Clone, PartialEq)]
/// [GeoStringFrame] helps us encode / decode
/// readable [MonumentGeoRecord] entries, which makes
/// the vast majority of supported frames
struct GeoStringFrame {
    /// [FieldID] frame identifier
    pub fid: FieldID,
    /// readable string
    pub string: String,
}

impl GeoStringFrame {
    pub fn new(fid: FieldID, s: &str) -> Self {
        Self {
            fid,
            string: s.to_string(),
        }
    }
    pub fn encoding_size(&self) -> usize {
        let mut size = 2; // FID + FID_1_4 (will never exceed 1)
        let s_len = self.string.len();
        size += Message::bnxi_encoding_size(s_len as u32);
        size += s_len;
        size
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MonumentGeoRecord {
    /// [Epoch]
    pub epoch: Epoch,
    /// Source of this information
    pub meta: MonumentGeoMetadata,
    /// Readable comments (if any), repeat as needed.
    pub comments: Vec<String>,
    /// Readable frames that we expose with high level methods
    frames: Vec<GeoStringFrame>,
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
    /// };
    ///     
    /// let t = Epoch::from_gpst_seconds(60.0 + 0.75);
    ///
    /// let record = MonumentGeoRecord::new(
    ///     t,
    ///     MonumentGeoMetadata::RNX2BIN,
    ///     "Fancy GNSS receiver",
    ///     "Magnificent antenna",
    ///     "SITE34", // Site name
    ///     "SITE34", // Site DOMES number
    /// );
    ///
    /// // customize as you need
    /// let record = record.
    ///     with_comment("you can add")
    ///     .with_comment("as many as you need")
    ///     .with_extra_info("Experiment or setup context")
    ///     .with_geophysical_info("Eurasian plate")
    ///     .with_climatic_info("Climatic Model XXXX");
    ///
    /// // define your preference,
    /// // which really impacts the decoder's end
    /// let big_endian = true;
    ///
    /// // buffer is too small!
    /// // you should always use .encoding_size()
    /// // to double check the size you need
    /// let mut buf = [0; 8];
    /// assert!(record.encode(big_endian, &mut buf).is_err());
    ///
    /// let mut buf = [0; 256];
    /// record.encode(true, &mut buf)
    ///     .unwrap();
    /// ```
    ///
    /// Another option is to use the Default constructor.
    /// But in this case you must pay attention to at least add
    /// one custom field (like one comments) otherwise the resulting
    /// frame would not be valid
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
    ///
    /// let mut buf = [0; 64];
    /// record.encode(true, &mut buf)
    ///     .unwrap();
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
    ///    - time_res: [TimeResolution]
    ///    - big_endian: endianness
    ///    - buf: buffered content
    /// ## Outputs
    ///    - Ok: [Self]
    ///    - Err: [Error]
    pub fn decode(
        mlen: usize,
        time_res: TimeResolution,
        big_endian: bool,
        buf: &[u8],
    ) -> Result<Self, Error> {
        let mut ret = Self::default();

        if mlen < Self::MIN_SIZE {
            // does not look good
            return Err(Error::NotEnoughBytes);
        }

        // decode timestamp
        ret.epoch = decode_gpst_epoch(big_endian, time_res, &buf)?;

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
                        ret.frames.push(GeoStringFrame::new(fid, s));
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
    pub fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.encoding_size();
        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        // encode tstamp
        let mut ptr = encode_epoch(self.epoch.to_time_scale(TimeScale::GPST), big_endian, buf)?;

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

    /// Define receiver model.
    pub fn with_receiver_model(&self, model: &str) -> Self {
        let mut s = self.clone();
        s.frames
            .push(GeoStringFrame::new(FieldID::ReceiverType, model));
        s
    }
    /// Define receiver serial number (if known).
    pub fn with_receiver_serial_number(&self, sn: &str) -> Self {
        let mut s = self.clone();
        s.frames
            .push(GeoStringFrame::new(FieldID::ReceiverNumber, sn));
        s
    }
    /// Define receiver firmware version (if known).
    pub fn with_receiver_firm_version(&self, version: &str) -> Self {
        let mut s = self.clone();
        s.frames.push(GeoStringFrame::new(
            FieldID::ReceiverFirmwareVersion,
            version,
        ));
        s
    }

    /// Define name of observer
    pub fn with_observer(&self, observer: &str) -> Self {
        let mut s = self.clone();
        s.frames
            .push(GeoStringFrame::new(FieldID::ObserverName, observer));
        s
    }

    /// Define observer's contact (email address)
    pub fn with_observer_contact(&self, contact: &str) -> Self {
        let mut s = self.clone();
        s.frames
            .push(GeoStringFrame::new(FieldID::ObserverContact, contact));
        s
    }
    /// Define Geodetic marker name
    pub fn with_geodetic_marker_name(&self, name: &str) -> Self {
        let mut s = self.clone();
        s.frames
            .push(GeoStringFrame::new(FieldID::MarkerName, name));
        s
    }

    /// Define Geodetic marker number (DOMES)
    pub fn with_geodetic_marker_number(&self, domes: &str) -> Self {
        let mut s = self.clone();
        s.frames
            .push(GeoStringFrame::new(FieldID::MarkerNumber, domes));
        s
    }

    /// Define Locatio of this geodetic site
    pub fn with_site_location(&self, location: &str) -> Self {
        let mut s = self.clone();
        s.frames
            .push(GeoStringFrame::new(FieldID::SiteLocation, location));
        s
    }

    /// Define Agency (Organization)
    pub fn with_agency(&self, agency: &str) -> Self {
        let mut s = self.clone();
        s.frames
            .push(GeoStringFrame::new(FieldID::AgencyName, agency));
        s
    }

    /// Define Antenna model (if known)
    pub fn with_antenna_model(&self, model: &str) -> Self {
        let mut s = self.clone();
        s.frames
            .push(GeoStringFrame::new(FieldID::AntennaType, model));
        s
    }

    /// Define Antenna serial number (if known)
    pub fn with_antenna_serial_number(&self, sn: &str) -> Self {
        let mut s = self.clone();
        s.frames
            .push(GeoStringFrame::new(FieldID::AntennaNumber, sn));
        s
    }

    /// Define name of Geodetic site
    pub fn with_site_name(&self, name: &str) -> Self {
        let mut s = self.clone();
        s.frames.push(GeoStringFrame::new(FieldID::SiteName, name));
        s
    }

    /// Attach readable Geophysical information (like local tectonic plate)
    /// to this [MonumentGeoRecord]. You can only add one per dataset
    /// otherwise, the message will not respect the standard definitions.
    pub fn with_geophysical_info(&self, info: &str) -> Self {
        let mut s = self.clone();
        s.frames
            .push(GeoStringFrame::new(FieldID::Geophysical, info));
        s
    }

    /// Provide Climatic or Meteorological information (local to reference site).
    /// You can only add one per dataset otherwise,
    /// the message will not respect the standard definitions.
    pub fn with_climatic_info(&self, info: &str) -> Self {
        let mut s = self.clone();
        s.frames.push(GeoStringFrame::new(FieldID::Climatic, info));
        s
    }

    /// Define a readable UserID to attach to this [MonumentGeoRecord] dataset.
    /// You can only add one per dataset otherwise,
    /// the message will not respect the standard definitions.
    pub fn with_user_id(&self, userid: &str) -> Self {
        let mut s = self.clone();
        s.frames.push(GeoStringFrame::new(FieldID::UserID, userid));
        s
    }

    /// Provide the name of this Geodetic project (if any).
    pub fn with_project_name(&self, name: &str) -> Self {
        let mut s = self.clone();
        s.frames
            .push(GeoStringFrame::new(FieldID::ProjectName, name));
        s
    }

    /// Add one extra note (like concise description of context
    /// or experiment)
    pub fn with_extra_info(&self, extra: &str) -> Self {
        let mut s = self.clone();
        s.frames.push(GeoStringFrame::new(FieldID::Extra, extra));
        s
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn monument_marker_bnx00_error() {
        let buf = [0, 0, 0, 0];
        let time_res = TimeResolution::QuarterSecond;
        let monument = MonumentGeoRecord::decode(4, time_res, true, &buf);
        assert!(monument.is_err());
    }

    #[test]
    fn monument_geo_comments_decoding() {
        let mlen = 17;
        let big_endian = true;
        let time_res = TimeResolution::QuarterSecond;

        let buf = [
            0, 0, 1, 1, 41, 2, 0, 9, 'H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8,
            ' ' as u8, 'G' as u8, 'E' as u8, 'O' as u8,
        ];

        let geo = MonumentGeoRecord::decode(mlen, time_res, big_endian, &buf).unwrap();

        assert_eq!(
            geo.epoch,
            Epoch::from_gpst_seconds(256.0 * 60.0 + 60.0 + 10.25)
        );

        assert_eq!(geo.meta, MonumentGeoMetadata::IGS);

        assert_eq!(geo.comments.len(), 1);

        let comments = geo.comments.get(0).unwrap();
        assert_eq!(comments, "Hello GEO");

        // ts +meta + FID_1_4 +strlen_1_4 +strlen
        assert_eq!(geo.encoding_size(), 5 + 1 + 1 + 1 + 9);

        // test mirror op
        let mut encoded = [0; 12];
        assert!(geo.encode(big_endian, &mut encoded).is_err());

        let mut encoded = [0; 18];
        let size = geo.encode(big_endian, &mut encoded).unwrap();
        assert_eq!(size, 5 + 1 + 1 + 1 + 9);
    }

    #[test]
    fn monument_geo_double_comments_decoding() {
        let mut geo: MonumentGeoRecord = MonumentGeoRecord::default()
            .with_comment("A B C")
            .with_comment("D E F");

        geo.epoch = Epoch::from_gpst_seconds(60.0 + 0.75);
        geo.meta = MonumentGeoMetadata::IGS;

        // ts + meta + 2*(FID_1_4 +STR_1_4 +STR)
        assert_eq!(geo.encoding_size(), 5 + 1 + 2 * (1 + 1 + 5));

        let mut buf = [0; 16];
        assert!(geo.encode(true, &mut buf).is_err());

        let mut buf = [0; 22];
        let size = geo.encode(true, &mut buf).unwrap();

        // ts + meta + 2*(FID_1_4 +strlen_1_4 +strlen)
        assert_eq!(geo.encoding_size(), 5 + 1 + 2 * (1 + 1 + 5));
        assert_eq!(size, 5 + 1 + 2 * (1 + 1 + 5));

        let mut geo = MonumentGeoRecord::default()
            .with_comment("Hello")
            .with_climatic_info("Clim");

        geo.epoch = Epoch::from_gpst_seconds(60.0 + 0.75);
        geo.meta = MonumentGeoMetadata::IGS;

        let mut buf = [0; 19];
        let size = geo.encode(true, &mut buf).unwrap();
        assert_eq!(size, 5 + 1 + 1 + 1 + 5 + 1 + 1 + 4);

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
                'H' as u8,
                'e' as u8,
                'l' as u8,
                'l' as u8,
                'o' as u8,
                14,
                4,
                'C' as u8,
                'l' as u8,
                'i' as u8,
                'm' as u8,
            ]
        );
    }
}
