use crate::{
    carrier::Carrier,
    epoch::parse_in_timescale as parse_epoch_in_timescale,
    prelude::{Epoch, ParsingError, TimeScale},
};

use bitflags::bitflags;

use std::{f64::consts::PI, str::FromStr};

bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct NgRegionFlags: u16 {
        const REGION5 = 0x01;
        const REGION4 = 0x02;
        const REGION3 = 0x04;
        const REGION2 = 0x08;
        const REGION1 = 0x10;
    }
}

/// Nequick-G Model payload
#[derive(Debug, Clone, Default, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct NgModel {
    /// a_i coefficients
    /// (sfu, (sfu.semi-circle⁻¹), (sfu.semi-circle⁻²))
    pub a: (f64, f64, f64),
    /// Region flags
    pub region: NgRegionFlags,
}

impl NgModel {
    /// Parses [NgModel] from Lines Iter
    pub(crate) fn parse(
        mut lines: std::str::Lines<'_>,
        ts: TimeScale,
    ) -> Result<(Epoch, Self), ParsingError> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(ParsingError::EmptyEpoch),
        };
        let (epoch, rem) = line.split_at(23);
        let (a0, rem) = rem.split_at(19);
        let (a1, rem) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(ParsingError::EmptyEpoch),
        };

        let epoch = parse_epoch_in_timescale(epoch.trim(), ts)?;
        let a = (
            f64::from_str(a0.trim()).map_err(|_| ParsingError::NequickGData)?,
            f64::from_str(a1.trim()).map_err(|_| ParsingError::NequickGData)?,
            f64::from_str(rem.trim()).map_err(|_| ParsingError::NequickGData)?,
        );
        let f = f64::from_str(line.trim()).map_err(|_| ParsingError::NequickGData)?;
        Ok((
            epoch,
            Self {
                a,
                region: NgRegionFlags::from_bits(f as u16).unwrap_or(NgRegionFlags::empty()),
            },
        ))
    }
    // /* converts self to meters of delay */
    // pub(crate) fn meters_delay(&self, freq: f64) -> f64 {
    //     0.0_f64
    // }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nequick_g_model() {
        let content =
            "    2022 06 08 09 59 57 7.850000000000E+01 5.390625000000E-01 2.713012695312E-02
     0.000000000000E+00";
        let content = content.lines();
        let parsed = NgModel::parse(content, TimeScale::UTC);
        assert!(parsed.is_ok());
        let (epoch, message) = parsed.unwrap();
        assert_eq!(
            epoch,
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 57, 00)
        );
        assert_eq!(
            message,
            NgModel {
                a: (7.850000000000E+01, 5.390625000000E-01, 2.713012695312E-02),
                region: NgRegionFlags::empty(),
            },
        );
    }
    
    #[test]
    fn rinex3_ng_header_parsing() {
        let ng = IonMessage::from_rinex3_header(
            "GAL    6.6250e+01 -1.6406e-01 -2.4719e-03  0.0000e+00       ",
        );
        assert!(ng.is_ok(), "failed to parse GAL iono correction header");
        let ng = ng.unwrap();
        assert_eq!(
            ng,
            IonMessage::NequickGModel(NgModel {
                a: (6.6250e+01, -1.6406e-01, -2.4719e-03),
                region: NgRegionFlags::empty(),
            })
        );
    }
    
}
