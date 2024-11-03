//! Epoch encoding & decoding
use crate::{utils::Utils, Error};
use hifitime::prelude::{Epoch, TimeScale, Unit};

/// BINEX Time Resolution
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TimeResolution {
    /// One Byte Second = 250ms resolution on [Epoch]s
    QuarterSecond = 0,
    /// 4 Byte Minutes and 2 byte millisecond
    MilliSecond = 1,
}

impl TimeResolution {
    pub fn encoding_size(&self) -> usize {
        match self {
            Self::QuarterSecond => 5,
            Self::MilliSecond => 6,
        }
    }
}

/// [Epoch] decoding attempt from buffered content.
/// If buffer does not contain enough data (5 bytes needed), this returns [Error::NotEnoughBytes].
pub fn decode_epoch(
    big_endian: bool,
    time_res: TimeResolution,
    buf: &[u8],
    ts: TimeScale,
) -> Result<Epoch, Error> {
    if buf.len() < time_res.encoding_size() {
        return Err(Error::NotEnoughBytes);
    }

    let min = Utils::decode_u32(big_endian, buf)?;
    match time_res {
        TimeResolution::QuarterSecond => {
            let qsec = buf[4];
            Ok(Epoch::from_duration(
                (min as f64) * Unit::Minute + (qsec as f64 / 4.0) * Unit::Second,
                ts,
            ))
        },
        TimeResolution::MilliSecond => {
            let ms = Utils::decode_u16(big_endian, &buf[4..])?;
            Ok(Epoch::from_duration(
                (min as f64) * Unit::Minute + (ms as f64) * Unit::Millisecond,
                ts,
            ))
        },
    }
}

/// [Epoch] encoding method.
/// If buffer is too small (needs 5 bytes), this returns [Error::NotEnoughBytes].
/// This is the exact [decode_epoch] mirror operation.
pub fn encode_epoch(
    t: Epoch,
    time_res: TimeResolution,
    big_endian: bool,
    buf: &mut [u8],
) -> Result<usize, Error> {
    let size = time_res.encoding_size();
    if buf.len() < size {
        return Err(Error::NotEnoughBytes);
    }

    let dt_s = t.duration.to_seconds();
    let total_mins = (dt_s / 60.0).round() as u32;
    let bytes = total_mins.to_be_bytes();

    if big_endian {
        buf[0] = bytes[0];
        buf[1] = bytes[1];
        buf[2] = bytes[2];
        buf[3] = bytes[3];
    } else {
        buf[0] = bytes[3];
        buf[1] = bytes[2];
        buf[2] = bytes[1];
        buf[3] = bytes[0];
    }

    match time_res {
        TimeResolution::MilliSecond => {
            let total_msec = (dt_s - (total_mins as f64) * 60.0) * 1.0E3;
            let bytes = ((total_msec as u16).min(59999)).to_be_bytes();

            if big_endian {
                buf[4] = bytes[0];
                buf[5] = bytes[1];
            } else {
                buf[4] = bytes[1];
                buf[5] = bytes[0];
            }
        },
        TimeResolution::QuarterSecond => {
            let total_qsec = (dt_s - (total_mins as f64) * 60.0) / 0.25;
            buf[4] = total_qsec as u8 & 0x7f; // 0xf0-0xff are excluded
        },
    }

    Ok(size)
}

/// GPST [Epoch] decoding attempt from buffered content.
pub(crate) fn decode_gpst_epoch(
    big_endian: bool,
    time_res: TimeResolution,
    buf: &[u8],
) -> Result<Epoch, Error> {
    decode_epoch(big_endian, time_res, buf, TimeScale::GPST)
}

// /// GST [Epoch] decoding attempt from buffered content.
// pub(crate) fn decode_gst_epoch(
//     big_endian: bool,
//     time_res: TimeResolution,
//     buf: &[u8],
// ) -> Result<Epoch, Error> {
//     decode_epoch(big_endian, time_res, buf, TimeScale::GST)
// }

// /// BDT [Epoch] decoding attempt from buffered content.
// pub(crate) fn decode_bdt_epoch(
//     big_endian: bool,
//     time_res: TimeResolution,
//     buf: &[u8],
// ) -> Result<Epoch, Error> {
//     decode_epoch(big_endian, time_res, buf, TimeScale::BDT)
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn epoch_fail() {
        let buf = [0];
        assert!(decode_gpst_epoch(true, TimeResolution::QuarterSecond, &buf).is_err());
        let buf = [0, 0, 0];
        assert!(decode_gpst_epoch(true, TimeResolution::QuarterSecond, &buf).is_err());
        let buf = [0, 0, 0, 0];
        assert!(decode_gpst_epoch(true, TimeResolution::MilliSecond, &buf).is_err());
    }

    #[test]
    fn gpst_qsec_sub_minute() {
        let big_endian = true;

        let mut buf = [0, 0, 0, 0, 0];
        let t = Epoch::from_gpst_seconds(10.0);
        let _ = encode_epoch(t, TimeResolution::QuarterSecond, big_endian, &mut buf).unwrap();
        assert_eq!(buf, [0, 0, 0, 0, 40]);

        let decoded = decode_epoch(
            big_endian,
            TimeResolution::QuarterSecond,
            &buf,
            TimeScale::GPST,
        )
        .unwrap();

        assert_eq!(decoded, t);

        let mut buf = [0, 0, 0, 0, 0];
        let t = Epoch::from_gpst_seconds(0.75);
        let _ = encode_epoch(t, TimeResolution::QuarterSecond, big_endian, &mut buf).unwrap();
        assert_eq!(buf, [0, 0, 0, 0, 3]);

        let decoded = decode_epoch(
            big_endian,
            TimeResolution::QuarterSecond,
            &buf,
            TimeScale::GPST,
        )
        .unwrap();

        assert_eq!(decoded, t);

        let mut buf = [0, 0, 0, 0, 0];
        let t = Epoch::from_gpst_seconds(10.75);
        let _ = encode_epoch(t, TimeResolution::QuarterSecond, big_endian, &mut buf).unwrap();
        assert_eq!(buf, [0, 0, 0, 0, 43]);

        let decoded = decode_epoch(
            big_endian,
            TimeResolution::QuarterSecond,
            &buf,
            TimeScale::GPST,
        )
        .unwrap();

        assert_eq!(decoded, t);
    }

    #[test]
    fn gpst_qsec_epoch_decoding() {
        for (buf, big_endian, gpst_epoch) in [
            ([0, 0, 0, 1, 0], true, Epoch::from_gpst_seconds(60.0)),
            ([0, 0, 0, 5, 0], true, Epoch::from_gpst_seconds(300.0)),
            (
                [0, 0, 1, 1, 1],
                true,
                Epoch::from_gpst_seconds(2.0_f64.powf(8.0) * 60.0 + 60.0 + 0.25),
            ),
            (
                [0, 0, 1, 1, 2],
                true,
                Epoch::from_gpst_seconds(2.0_f64.powf(8.0) * 60.0 + 60.0 + 0.5),
            ),
            (
                [0, 0, 1, 1, 0],
                true,
                Epoch::from_gpst_seconds(2.0_f64.powf(8.0) * 60.0 + 60.0),
            ),
            (
                [1, 1, 1, 0, 0],
                true,
                Epoch::from_gpst_seconds(
                    (2.0_f64.powf(24.0) + 2.0_f64.powf(16.0) + 2.0_f64.powf(8.0)) * 60.0,
                ),
            ),
        ] {
            let epoch = decode_gpst_epoch(big_endian, TimeResolution::QuarterSecond, &buf);
            assert!(epoch.is_ok(), "to parse valid gpst_epoch");
            let epoch = epoch.unwrap();
            assert_eq!(epoch, gpst_epoch);

            // test mirror op
            let mut test = [0, 0, 0, 0, 0];
            let _ =
                encode_epoch(epoch, TimeResolution::QuarterSecond, big_endian, &mut test).unwrap();
            assert_eq!(test, buf, "encode_epoch failed for {}", epoch);
        }
    }

    #[test]
    fn gpst_msec() {
        let t = Epoch::from_gpst_seconds(60.0);
        let mut buf = [0, 0, 0, 0, 0, 0];

        encode_epoch(t, TimeResolution::MilliSecond, true, &mut buf).unwrap();
        assert_eq!(buf, [0, 0, 0, 1, 0, 0]);

        let parsed = decode_gpst_epoch(true, TimeResolution::MilliSecond, &buf).unwrap();
        assert_eq!(parsed, t);

        let t = Epoch::from_gpst_seconds(61.0);
        let mut buf = [0, 0, 0, 0, 0, 0];

        encode_epoch(t, TimeResolution::MilliSecond, true, &mut buf).unwrap();
        assert_eq!(buf, [0, 0, 0, 1, 0x3, 0xe8]);

        let parsed = decode_gpst_epoch(true, TimeResolution::MilliSecond, &buf).unwrap();
        assert_eq!(parsed, t);

        let t = Epoch::from_gpst_seconds(71.0);
        let mut buf = [0, 0, 0, 0, 0, 0];

        encode_epoch(t, TimeResolution::MilliSecond, true, &mut buf).unwrap();
        assert_eq!(buf, [0, 0, 0, 1, 0x2a, 0xf8]);

        let parsed = decode_gpst_epoch(true, TimeResolution::MilliSecond, &buf).unwrap();
        assert_eq!(parsed, t);
    }
}
