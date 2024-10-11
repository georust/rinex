//! Epoch encoding & decoding

use hifitime::{
    prelude::{Epoch, TimeScale, Unit},
    BDT_REF_EPOCH, GPST_REF_EPOCH, GST_REF_EPOCH,
};

use crate::{utils::Utils, Error};

/// BINEX Time Resolution
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum TimeResolution {
    /// 250ms resolution on all [Epoch]s
    #[default]
    QuarterSecond = 0,
}

/// [Epoch] decoding attempt from buffered content.
/// If buffer does not contain enough data (5 bytes needed), this returns [Error::NotEnoughBytes].
pub(crate) fn decode_epoch(
    big_endian: bool,
    time_res: TimeResolution,
    buf: &[u8],
    ts: TimeScale,
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
    Ok(Epoch::from_duration(
        (min as f64) * Unit::Minute + (qsec as f64 / 4.0) * Unit::Second,
        ts,
    ))
}

/// [Epoch] encoding method.
/// If buffer is too small (needs 5 bytes), this returns [Error::NotEnoughBytes].
/// This is the exact [decode_epoch] mirror operation.
/// We only support GPST, GST and BDT at the moment,
/// it will return [Error::NonSupportedTimescale] for any other timescale.
pub(crate) fn encode_epoch(
    t: Epoch,
    big_endian: bool,
    time_res: TimeResolution,
    buf: &mut [u8],
) -> Result<usize, Error> {
    if buf.len() < 5 {
        return Err(Error::NotEnoughBytes);
    }
    let t0 = match t.time_scale {
        TimeScale::GPST => GPST_REF_EPOCH,
        TimeScale::GST => GST_REF_EPOCH,
        TimeScale::BDT => BDT_REF_EPOCH,
        _ => {
            return Err(Error::NonSupportedTimescale);
        },
    };
    let dt = (t - t0).to_seconds();
    let total_mins = (dt / 60.0).round() as u32;
    let total_qsec = ((dt / 0.25) * 10.0).round() as u8;
    let bytes = total_mins.to_be_bytes();
    buf[0] = bytes[0];
    buf[1] = bytes[1];
    buf[2] = bytes[2];
    buf[3] = bytes[3];
    buf[4] = total_qsec;
    Ok(5)
}

/// GPST [Epoch] decoding attempt from buffered content.
pub(crate) fn decode_gpst_epoch(
    big_endian: bool,
    time_res: TimeResolution,
    buf: &[u8],
) -> Result<Epoch, Error> {
    decode_epoch(big_endian, time_res, buf, TimeScale::GPST)
}

/// GST [Epoch] decoding attempt from buffered content.
pub(crate) fn decode_gst_epoch(
    big_endian: bool,
    time_res: TimeResolution,
    buf: &[u8],
) -> Result<Epoch, Error> {
    decode_epoch(big_endian, time_res, buf, TimeScale::GST)
}

/// BDT [Epoch] decoding attempt from buffered content.
pub(crate) fn decode_bdt_epoch(
    big_endian: bool,
    time_res: TimeResolution,
    buf: &[u8],
) -> Result<Epoch, Error> {
    decode_epoch(big_endian, time_res, buf, TimeScale::BDT)
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
            let epoch = epoch.unwrap();
            assert_eq!(epoch, gpst_epoch);

            // test mirror op
            let mut test = [0, 0, 0, 0, 0];
            let _ =
                encode_epoch(epoch, big_endian, TimeResolution::QuarterSecond, &mut test).unwrap();
            assert_eq!(test, buf);
        }
    }
}
