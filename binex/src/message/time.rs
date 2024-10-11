//! Epoch encoding & decoding

use hifitime::{Epoch, Unit, BDT_REF_EPOCH, GPST_REF_EPOCH, GST_REF_EPOCH};

use crate::{utils::Utils, Error};

/// BINEX Time Resolution
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum TimeResolution {
    /// 250ms resolution on all [Epoch]s
    #[default]
    QuarterSecond = 0,
}

/// [Epoch] decoding attempt from buffered content.
pub(crate) fn decode_epoch(
    big_endian: bool,
    time_res: TimeResolution,
    buf: &[u8],
    t0: Epoch,
) -> Result<Epoch, Error> {
    let mut min = 0_u32;
    let mut qsec = 0_u8;
    if buf.len() < 5 {
        return Err(Error::NotEnoughBytes);
    }
    match time_res {
        TimeResolution::QuarterSecond => {
            min = Utils::decode_u32(big_endian, buf)?;
            qsec = buf[4];
        },
    }
    Ok(t0 + (min as f64) * Unit::Minute + (qsec as f64 / 4.0) * Unit::Second)
}

/// GPST [Epoch] decoding attempt from buffered content.
pub(crate) fn decode_gpst_epoch(
    big_endian: bool,
    time_res: TimeResolution,
    buf: &[u8],
) -> Result<Epoch, Error> {
    decode_epoch(big_endian, time_res, buf, GPST_REF_EPOCH)
}

/// GST [Epoch] decoding attempt from buffered content.
pub(crate) fn decode_gst_epoch(
    big_endian: bool,
    time_res: TimeResolution,
    buf: &[u8],
) -> Result<Epoch, Error> {
    decode_epoch(big_endian, time_res, buf, GST_REF_EPOCH)
}

/// BDT [Epoch] decoding attempt from buffered content.
pub(crate) fn decode_bdt_epoch(
    big_endian: bool,
    time_res: TimeResolution,
    buf: &[u8],
) -> Result<Epoch, Error> {
    decode_epoch(big_endian, time_res, buf, BDT_REF_EPOCH)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn epoch_fail() {
        let buf = [0];
        assert!(decode_gpst_epoch(true, TimeResolution::QuarterSecond, &buf).is_err());
        let buf = [0, 0, 0];
        assert!(decode_gpst_epoch(true, TimeResolution::QuarterSecond, &buf).is_err());
    }
    #[test]
    fn gpst_epoch_decoding() {
        // test QSEC
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
            assert_eq!(epoch.unwrap(), gpst_epoch);
        }
    }
}
