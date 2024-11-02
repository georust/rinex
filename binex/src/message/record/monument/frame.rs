//! Monument Geodetic marker specific frames

use crate::{
    message::{record::monument::FieldID, Message},
    Error,
};

// use log::error;

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
