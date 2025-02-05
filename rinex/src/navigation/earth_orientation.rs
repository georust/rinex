//! Earth Orientation NAV frames

use crate::{
    epoch::parse_in_timescale as parse_epoch_in_timescale,
    prelude::{Epoch, ParsingError, TimeScale},
};

use std::str::FromStr;

/// Earth Orientation Message
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct EarthOrientation {
    /// ((arc-sec), (arc-sec.day⁻¹), (arc-sec.day⁻²))
    pub x: (f64, f64, f64),
    /// ((arc-sec), (arc-sec.day⁻¹), (arc-sec.day⁻²))
    pub y: (f64, f64, f64),
    /// Message transmmission time in seconds of GNSS week
    pub t_tm: u32,
    /// Delta UT1 ((sec), (sec.day⁻¹), (sec.day⁻²))
    pub delta_ut1: (f64, f64, f64),
}

impl EarthOrientation {
    pub(crate) fn parse(
        mut lines: std::str::Lines<'_>,
        ts: TimeScale,
    ) -> Result<(Epoch, Self), ParsingError> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(ParsingError::EopMissingData),
        };

        let (epoch, rem) = line.split_at(23);
        let (xp, rem) = rem.split_at(19);
        let (dxp, ddxp) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(ParsingError::EopMissingData),
        };

        let (_, rem) = line.split_at(23);
        let (yp, rem) = rem.split_at(19);
        let (dyp, ddyp) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(ParsingError::EopMissingData),
        };

        let (t_tm, rem) = line.split_at(23);
        let (dut, rem) = rem.split_at(19);
        let (ddut, dddut) = rem.split_at(19);

        let epoch = parse_epoch_in_timescale(epoch.trim(), ts)?;
        let x = (
            f64::from_str(xp.trim()).unwrap_or(0.0_f64),
            f64::from_str(dxp.trim()).unwrap_or(0.0_f64),
            f64::from_str(ddxp.trim()).unwrap_or(0.0_f64),
        );
        let y = (
            f64::from_str(yp.trim()).unwrap_or(0.0_f64),
            f64::from_str(dyp.trim()).unwrap_or(0.0_f64),
            f64::from_str(ddyp.trim()).unwrap_or(0.0_f64),
        );
        let t_tm = f64::from_str(t_tm.trim()).unwrap_or(0.0_f64);
        let delta_ut1 = (
            f64::from_str(dut.trim()).unwrap_or(0.0_f64),
            f64::from_str(ddut.trim()).unwrap_or(0.0_f64),
            f64::from_str(dddut.trim()).unwrap_or(0.0_f64),
        );

        Ok((
            epoch,
            Self {
                x,
                y,
                t_tm: t_tm as u32,
                delta_ut1,
            },
        ))
    }
}
