//! `Navigation` new EOP Earth Orientation messages
use crate::epoch;
use thiserror::Error;
use std::str::FromStr;
use crate::prelude::*;

/// EopMessage Parsing error 
#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse epoch")]
    EpochError(#[from] epoch::Error),
    #[error("eop message missing 1st line")]
    EopMissing1stLine,
    #[error("eop message missing 2nd line")]
    EopMissing2ndLine,
    #[error("eop message missing 3rd line")]
    EopMissing3rdLine,
}

/// Earth Orientation Message 
/// ```
/// use rinex::prelude::*;
/// use rinex::navigation::*;
/// let rnx = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
///     .unwrap();
/// let record = rnx.record.as_nav()
///     .unwrap();
/// for (epoch, classes) in record {
///     for (class, frames) in classes {
///         // epochs may contain other frame classes
///         if *class == FrameClass::EarthOrientation {
///             for fr in frames {
///                 let (msg_type, sv, eop) = fr.as_eop()
///                     .unwrap(); // you're fine at this point
///                 let (x, dxdt, ddxdt) = eop.x;
///                 let (y, dydt, ddydt) = eop.y;
///                 let t_tm = eop.t_tm; 
///                 let (u, dudt, ddudt) = eop.delta_ut1; 
///             }
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
#[derive(Default)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct EopMessage {
    /// ([arc-sec], [arc-sec.day⁻¹], [arc-sec.day⁻²])
    pub x: (f64,f64,f64),
    /// ([arc-sec], [arc-sec.day⁻¹], [arc-sec.day⁻²])
    pub y: (f64,f64,f64),
    /// Message transmmission time [s] of GNSS week
    pub t_tm: u32,
    /// Delta UT1 ([sec], [sec.day⁻¹], [-sec.day⁻²])
    pub delta_ut1: (f64,f64,f64),
}

impl EopMessage {
    pub (crate)fn parse(mut lines: std::str::Lines<'_>) -> Result<(Epoch, Self), Error> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::EopMissing1stLine)
        };
        let (epoch, rem) = line.split_at(23);
        let (xp, rem) = rem.split_at(19);
        let (dxp, ddxp) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::EopMissing2ndLine)
        };
        let (_, rem) = line.split_at(23);
        let (yp, rem) = rem.split_at(19);
        let (dyp, ddyp) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::EopMissing3rdLine)
        };
        let (t_tm, rem) = line.split_at(23);
        let (dut, rem) = rem.split_at(19);
        let (ddut, dddut) = rem.split_at(19);

        let (epoch, _) = epoch::parse(epoch.trim())?;
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
            epoch, Self {
                x,
                y,
                t_tm: t_tm as u32,
                delta_ut1,
            }))
    }
}
