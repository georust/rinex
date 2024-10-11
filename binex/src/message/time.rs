//! Epoch encoding & decoding

use hifitime::{Epoch, Unit, GPST_REF_EPOCH};

use crate::{utils::Utils, Error};

/// BINEX Time Resolution
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum TimeResolution {
    /// 250ms resolution on all [Epoch]s
    #[default]
    QuarterSecond = 0,
}

/// [Epoch] decoding attempt from buffered content.
pub(crate) fn decode_gpst_epoch(time_res: TimeResolution, buf: &[u8]) -> Result<Epoch, Error> {
    let mut min = 0_u32;
    let mut qsec = 0_u8;
    match time_res {
        TimeResolution::QuarterSecond => {
            min = Utils::decode_u32(buf)?;
            qsec = Utils::decode_u8(buf)?;
        },
    }
    Ok(GPST_REF_EPOCH + (min as f64) * Unit::Minute + (qsec as f64 / 4.0) * Unit::Second)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn epoch_fail() {
        let buf = [0, 0, 0, 0, 0];
        assert!(decode_gpst_epoch(TimeResolution::QuarterSecond, buf).is_err());
        let buf = [0, 0, 0, 0, 0, 0];
        assert!(decode_gpst_epoch(TimeResolution::QuarterSecond, buf).is_err());
    }
    #[test]
    fn gpst_epoch_decoding() {
        // test QSEC
        for (buf, gpst_epoch) in [([0, 0, 0, 0, 0, 0, 0], Epoch::from_gpst_seconds(100.0))] {
            let epoch = decode_gpst_epoch(TimeResolution::QuarterSecond, buf);
            assert!(epoch.is_ok(), "failed to parse valid gpst_epoch");
            assert_eq!(epoch.unwrap(), gpst_epoch);
        }
    }
}
