use crate::{
    carrier::Carrier,
    epoch::{parse_in_timescale, ParsingError as EpochParsingError},
    prelude::{
        Epoch,
        TimeScale,
        //Duration,
    },
};
use bitflags::bitflags;
use std::str::FromStr;
use thiserror::Error;

use map_3d::deg2rad;
use std::f64::consts::PI;

/// Model parsing error
#[derive(Debug, Error)]
pub enum Error {
    #[error("ng model missing 1st line")]
    NgModelMissing1stLine,
    #[error("kb model missing 1st line")]
    KbModelMissing1stLine,
    #[error("bd model missing 1st line")]
    BdModelMissing1stLine,
    #[error("ng model missing 2nd line")]
    NgModelMissing2ndLine,
    #[error("kb model missing 2nd line")]
    KbModelMissing2ndLine,
    #[error("bd model missing 2nd line")]
    BdModelMissing2ndLine,
    #[error("kb model missing 3rd line")]
    KbModelMissing3rdLine,
    #[error("bd model missing 3rd line")]
    BdModelMissing3rdLine,
    #[error("missing data fields")]
    MissingData,
    #[error("failed to parse float data")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse epoch")]
    EpochParsingError(#[from] EpochParsingError),
}

/// Klobuchar Parameters region
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum KbRegionCode {
    /// Coefficients apply to wide area
    WideArea = 0,
    /// Japan Area coefficients
    JapanArea = 1,
}

impl Default for KbRegionCode {
    fn default() -> Self {
        Self::WideArea
    }
}

/// Klobuchar model payload,
/// we don't know how to parse the possible extra Region Code yet
#[derive(Default, Debug, Copy, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct KbModel {
    /// Alpha coefficients
    /// ((sec), (sec.semi-circle⁻¹), (sec.semi-circle⁻²), (sec.semi-circle⁻³))
    pub alpha: (f64, f64, f64, f64),
    /// Beta coefficients
    /// ((sec), (sec.semi-circle⁻¹), (sec.semi-circle⁻²), (sec.semi-circle⁻³))
    pub beta: (f64, f64, f64, f64),
    /// Region flag
    pub region: KbRegionCode,
}

impl KbModel {
    /*
     * Parse self from line groupings
     */
    pub(crate) fn parse(
        mut lines: std::str::Lines<'_>,
        ts: TimeScale,
    ) -> Result<(Epoch, Self), Error> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::NgModelMissing1stLine),
        };
        let (epoch, rem) = line.split_at(23);
        let (a0, rem) = rem.split_at(19);
        let (a1, a2) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::KbModelMissing2ndLine),
        };
        let (a3, rem) = line.split_at(23);
        let (b0, rem) = rem.split_at(19);
        let (b1, b2) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::KbModelMissing3rdLine),
        };
        let (b3, region) = line.split_at(23);

        let region: KbRegionCode = match region.trim().len() {
            0 => KbRegionCode::WideArea,
            _ => {
                if let Ok(f) = f64::from_str(region.trim()) {
                    let code = f as u8;
                    if code == 1 {
                        KbRegionCode::JapanArea
                    } else {
                        KbRegionCode::WideArea
                    }
                } else {
                    KbRegionCode::WideArea
                }
            },
        };

        let (epoch, _) = parse_in_timescale(epoch.trim(), ts)?;
        let alpha = (
            f64::from_str(a0.trim()).unwrap_or(0.0_f64),
            f64::from_str(a1.trim()).unwrap_or(0.0_f64),
            f64::from_str(a2.trim()).unwrap_or(0.0_f64),
            f64::from_str(a3.trim()).unwrap_or(0.0_f64),
        );
        let beta = (
            f64::from_str(b0.trim()).unwrap_or(0.0_f64),
            f64::from_str(b1.trim()).unwrap_or(0.0_f64),
            f64::from_str(b2.trim()).unwrap_or(0.0_f64),
            f64::from_str(b3.trim()).unwrap_or(0.0_f64),
        );

        Ok((
            epoch,
            Self {
                alpha,
                beta,
                region,
            },
        ))
    }
    /* converts self to meters of delay */
    pub(crate) fn meters_delay(
        &self,
        t: Epoch,
        e: f64,
        a: f64,
        h_km: f64,
        user_lat_ddeg: f64,
        user_lon_ddeg: f64,
        carrier: Carrier,
    ) -> f64 {
        const PHI_P: f64 = 78.3;
        const R_EARTH: f64 = 6378.0;
        const LAMBDA_P: f64 = 291.0;
        const L1_F: f64 = 1575.42E6;

        let fract = R_EARTH / (R_EARTH + h_km);
        let phi_u = deg2rad(user_lat_ddeg);
        let lambda_u = deg2rad(user_lon_ddeg);

        let t_gps = t.to_duration_in_time_scale(TimeScale::GPST).to_seconds();
        let psi = PI / 2.0 - e - (fract * e.cos()).asin();
        let phi_i = (phi_u.sin() * psi.cos() + phi_u.cos() * psi.sin() * a.cos()).asin();
        let lambda_i = lambda_u + a.sin() * psi / phi_i.cos();
        let phi_m = (phi_i.sin() * PHI_P.sin()
            + phi_i.cos() * PHI_P.cos() * (lambda_i - LAMBDA_P).cos())
        .asin();

        let mut t_s = 43.2E3 * lambda_i / PI + t_gps;
        if t_s > 86.4E3 {
            t_s -= 86.4E3;
        } else if t_s < 0.0 {
            t_s += 86.4E3;
        }

        let mut a_i = self.alpha.0 * (phi_m / PI).powi(0)
            + self.alpha.1 * (phi_m / PI).powi(1)
            + self.alpha.2 * (phi_m / PI).powi(2)
            + self.alpha.3 * (phi_m / PI).powi(3);
        if a_i < 0.0 {
            a_i = 0.0_f64;
        }
        let mut p_i = self.beta.0 * (phi_m / PI).powi(0)
            + self.beta.1 * (phi_m / PI).powi(1)
            + self.beta.2 * (phi_m / PI).powi(2)
            + self.beta.3 * (phi_m / PI).powi(3);
        if p_i < 72.0E3 {
            p_i = 72.0E3;
        }

        let x_i = 2.0 * PI * (t_s - 50400.0) / p_i;
        let f = 1.0 / ((1.0 - fract * e.cos()).powi(2)).sqrt();
        let i_1 = match x_i < PI / 2.0 {
            true => 5.0 * 10E-9 + a_i * x_i.cos(),
            false => f * 5.0 * 10E-9,
        };

        if carrier == Carrier::L1 {
            i_1
        } else {
            i_1 * (L1_F / carrier.frequency()).powi(2)
        }
    }
}

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
    /*
     * Parse self from line groupings
     */
    pub(crate) fn parse(
        mut lines: std::str::Lines<'_>,
        ts: TimeScale,
    ) -> Result<(Epoch, Self), Error> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::NgModelMissing1stLine),
        };
        let (epoch, rem) = line.split_at(23);
        let (a0, rem) = rem.split_at(19);
        let (a1, rem) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::NgModelMissing2ndLine),
        };

        let (epoch, _) = parse_in_timescale(epoch.trim(), ts)?;
        let a = (
            f64::from_str(a0.trim())?,
            f64::from_str(a1.trim())?,
            f64::from_str(rem.trim())?,
        );
        let f = f64::from_str(line.trim())?;
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

/// BDGIM Model payload
#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct BdModel {
    /// Alpha coefficients in TEC unit
    pub alpha: (f64, f64, f64, f64, f64, f64, f64, f64, f64),
}

impl BdModel {
    /*
     * Parse Self from line groupings
     */
    pub(crate) fn parse(
        mut lines: std::str::Lines<'_>,
        ts: TimeScale,
    ) -> Result<(Epoch, Self), Error> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::BdModelMissing1stLine),
        };
        let (epoch, rem) = line.split_at(23);
        let (a0, rem) = rem.split_at(19);
        let (a1, a2) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::KbModelMissing2ndLine),
        };
        let (a3, rem) = line.split_at(23);
        let (a4, rem) = rem.split_at(19);
        let (a5, a6) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::KbModelMissing3rdLine),
        };
        let (a7, a8) = line.split_at(23);

        let (epoch, _) = parse_in_timescale(epoch.trim(), ts)?;
        let alpha = (
            f64::from_str(a0.trim()).unwrap_or(0.0_f64),
            f64::from_str(a1.trim()).unwrap_or(0.0_f64),
            f64::from_str(a2.trim()).unwrap_or(0.0_f64),
            f64::from_str(a3.trim()).unwrap_or(0.0_f64),
            f64::from_str(a4.trim()).unwrap_or(0.0_f64),
            f64::from_str(a5.trim()).unwrap_or(0.0_f64),
            f64::from_str(a6.trim()).unwrap_or(0.0_f64),
            f64::from_str(a7.trim()).unwrap_or(0.0_f64),
            f64::from_str(a8.trim()).unwrap_or(0.0_f64),
        );
        Ok((epoch, Self { alpha }))
    }
    // /* converts self to meters of delay */
    // pub(crate) fn meters_delay(&self, freq: f64) -> f64 {
    //     0.0_f64
    // }
}

/// IonMessage: wraps several ionospheric models
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum IonMessage {
    /// Klobuchar Model
    KlobucharModel(KbModel),
    /// Nequick-G Model
    NequickGModel(NgModel),
    /// BDGIM Model
    BdgimModel(BdModel),
}

impl Default for IonMessage {
    fn default() -> Self {
        Self::KlobucharModel(KbModel::default())
    }
}

impl IonMessage {
    // /* converts self to meters of delay */
    // pub(crate) fn meters_delay(
    //     &self,
    //     t: Epoch,
    //     e: f64,
    //     a: f64,
    //     h_km: f64,
    //     user_lat_ddeg: f64,
    //     user_lon_ddeg: f64,
    //     carrier: Carrier,
    // ) -> Option<f64> {
    //     if let Some(kb) = self.as_klobuchar() {
    //         Some(kb.meters_delay(t, e, a, h_km, user_lat_ddeg, user_lon_ddeg, carrier))
    //     } else if let Some(ng) = self.as_nequick_g() {
    //         None
    //     } else if let Some(bd) = self.as_bdgim() {
    //         None
    //     } else {
    //         None
    //     }
    // }
    /// Unwraps self as Klobuchar Model
    pub fn as_klobuchar(&self) -> Option<&KbModel> {
        match self {
            Self::KlobucharModel(model) => Some(model),
            _ => None,
        }
    }
    /// Unwraps self as Nequick-G Model
    pub fn as_nequick_g(&self) -> Option<&NgModel> {
        match self {
            Self::NequickGModel(model) => Some(model),
            _ => None,
        }
    }
    /// Unwraps self as BDGIM Model
    pub fn as_bdgim(&self) -> Option<&BdModel> {
        match self {
            Self::BdgimModel(model) => Some(model),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_kb() {
        assert_eq!(KbRegionCode::default(), KbRegionCode::WideArea);
        let content =
            "    2022 06 08 09 59 48 1.024454832077E-08 2.235174179077E-08-5.960464477539E-08
    -1.192092895508E-07 9.625600000000E+04 1.310720000000E+05-6.553600000000E+04
    -5.898240000000E+05 0.000000000000E+00";
        let content = content.lines();
        let parsed = KbModel::parse(content, TimeScale::UTC);
        assert!(parsed.is_ok());
        let (epoch, message) = parsed.unwrap();
        assert_eq!(
            epoch,
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 48, 00)
        );
        assert_eq!(
            message,
            KbModel {
                alpha: (
                    1.024454832077E-08,
                    2.235174179077E-08,
                    -5.960464477539E-08,
                    -1.192092895508E-07
                ),
                beta: (
                    9.625600000000E+04,
                    1.310720000000E+05,
                    -6.553600000000E+04,
                    -5.898240000000E+05
                ),
                region: KbRegionCode::WideArea,
            },
        );
    }
    #[test]
    fn test_ng() {
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
    fn test_ionmessage() {
        let msg = IonMessage::KlobucharModel(KbModel::default());
        assert!(msg.as_klobuchar().is_some());
        assert!(msg.as_nequick_g().is_none());
        assert!(msg.as_bdgim().is_none());
    }
}
