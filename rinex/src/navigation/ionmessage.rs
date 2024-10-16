use crate::{
    carrier::Carrier,
    epoch::parse_in_timescale as parse_epoch_in_timescale,
    prelude::{Epoch, ParsingError, TimeScale},
};

use bitflags::bitflags;

use std::{f64::consts::PI, str::FromStr};

/// Klobuchar Parameters region
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum KbRegionCode {
    /// Worlwide (GPS) Orbits.
    WideArea = 0,
    /// QZSS Japanese special Orbital plan.
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
    /// Parse [KbModel] from lines Iter
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
        let (a1, a2) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(ParsingError::EmptyEpoch),
        };
        let (a3, rem) = line.split_at(23);
        let (b0, rem) = rem.split_at(19);
        let (b1, b2) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(ParsingError::EmptyEpoch),
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

        let epoch = parse_epoch_in_timescale(epoch.trim(), ts)?;
        let alpha = (
            f64::from_str(a0.trim()).map_err(|_| ParsingError::KlobucharData)?,
            f64::from_str(a1.trim()).map_err(|_| ParsingError::KlobucharData)?,
            f64::from_str(a2.trim()).map_err(|_| ParsingError::KlobucharData)?,
            f64::from_str(a3.trim()).map_err(|_| ParsingError::KlobucharData)?,
        );
        let beta = (
            f64::from_str(b0.trim()).map_err(|_| ParsingError::KlobucharData)?,
            f64::from_str(b1.trim()).map_err(|_| ParsingError::KlobucharData)?,
            f64::from_str(b2.trim()).map_err(|_| ParsingError::KlobucharData)?,
            f64::from_str(b3.trim()).map_err(|_| ParsingError::KlobucharData)?,
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

    // Converts [KbModel] to meters of delay
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
        let phi_u = user_lat_ddeg.to_radians();
        let lambda_u = user_lon_ddeg.to_radians();

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

/// BDGIM Model payload
#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct BdModel {
    /// Alpha coefficients in TEC unit
    pub alpha: (f64, f64, f64, f64, f64, f64, f64, f64, f64),
}

impl BdModel {
    /// Parses [BdModel] from Lines Iter
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
        let (a1, a2) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(ParsingError::EmptyEpoch),
        };
        let (a3, rem) = line.split_at(23);
        let (a4, rem) = rem.split_at(19);
        let (a5, a6) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(ParsingError::EmptyEpoch),
        };
        let (a7, a8) = line.split_at(23);

        let epoch = parse_epoch_in_timescale(epoch.trim(), ts)?;

        let alpha = (
            f64::from_str(a0.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a1.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a2.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a3.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a4.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a5.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a6.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a7.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a8.trim()).map_err(|_| ParsingError::BdgimData)?,
        );

        Ok((epoch, Self { alpha }))
    }
    // /* converts self to meters of delay */
    // pub(crate) fn meters_delay(&self, freq: f64) -> f64 {
    //     0.0_f64
    // }
}

/// IonMessage wraps all known Ionosphere models
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
    /// Parses [IonMessage] from old (RINEX3) Ionospheric Correction as IonMessage.
    /// The IonMessage is shared by RINEX3 and newest revision
    /// to represent the Ionospheric correction model.
    /// It's just unique and applies to the entire daycourse in RINEX3.
    /// The API remains totally coherent.
    pub(crate) fn from_rinex3_header(header: &str) -> Result<Self, ParsingError> {
        let (system, rem) = header.split_at(5);
        match system.trim() {
            /*
             * Models that only needs 3 fields
             */
            "GAL" => {
                let (a0, rem) = rem.split_at(12);
                let (a1, rem) = rem.split_at(12);
                let (a2, _) = rem.split_at(12);
                let a0 = f64::from_str(a0.trim()).map_err(|_| ParsingError::NequickGData)?;
                let a1 = f64::from_str(a1.trim()).map_err(|_| ParsingError::NequickGData)?;
                let a2 = f64::from_str(a2.trim()).map_err(|_| ParsingError::NequickGData)?;
                Ok(Self::NequickGModel(NgModel {
                    a: (a0, a1, a2),
                    // TODO: is this not the 4th field? double check
                    region: NgRegionFlags::default(),
                }))
            },
            system => {
                /*
                 * Model has 4 fields
                 */
                let (a0, rem) = rem.split_at(12);
                let (a1, rem) = rem.split_at(12);
                let (a2, rem) = rem.split_at(12);
                let (a3, _) = rem.split_at(12);
                // World or QZSS special orbital plan
                let region = match system.contains("QZS") {
                    true => KbRegionCode::JapanArea,
                    false => KbRegionCode::WideArea,
                };
                /* determine which field we're dealing with */
                if system.ends_with('A') {
                    let a0 = f64::from_str(a0.trim()).map_err(|_| ParsingError::KlobucharData)?;
                    let a1 = f64::from_str(a1.trim()).map_err(|_| ParsingError::KlobucharData)?;
                    let a2 = f64::from_str(a2.trim()).map_err(|_| ParsingError::KlobucharData)?;
                    let a3 = f64::from_str(a3.trim()).map_err(|_| ParsingError::KlobucharData)?;

                    Ok(Self::KlobucharModel(KbModel {
                        alpha: (a0, a1, a2, a3),
                        beta: (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64),
                        region,
                    }))
                } else {
                    let b0 = f64::from_str(a0.trim()).map_err(|_| ParsingError::KlobucharData)?;
                    let b1 = f64::from_str(a1.trim()).map_err(|_| ParsingError::KlobucharData)?;
                    let b2 = f64::from_str(a2.trim()).map_err(|_| ParsingError::KlobucharData)?;
                    let b3 = f64::from_str(a3.trim()).map_err(|_| ParsingError::KlobucharData)?;
                    Ok(Self::KlobucharModel(KbModel {
                        alpha: (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64),
                        beta: (b0, b1, b2, b3),
                        region,
                    }))
                }
            },
        }
    }

    pub(crate) fn from_rinex2_header(header: &str, marker: &str) -> Result<Self, ParsingError> {
        let header = header.replace("d", "e").replace("D", "E");
        let (_, rem) = header.split_at(2);
        let (a0, rem) = rem.split_at(12);
        let (a1, rem) = rem.split_at(12);
        let (a2, rem) = rem.split_at(12);
        let (a3, _) = rem.split_at(12);

        if marker.contains("ALPHA") {
            let a0 = f64::from_str(a0.trim()).map_err(|_| ParsingError::KlobucharData)?;
            let a1 = f64::from_str(a1.trim()).map_err(|_| ParsingError::KlobucharData)?;
            let a2 = f64::from_str(a2.trim()).map_err(|_| ParsingError::KlobucharData)?;
            let a3 = f64::from_str(a3.trim()).map_err(|_| ParsingError::KlobucharData)?;

            Ok(Self::KlobucharModel(KbModel {
                alpha: (a0, a1, a2, a3),
                beta: (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64),
                region: KbRegionCode::WideArea,
            }))
        } else {
            // Assume marker.contains("BETA")
            let b0 = f64::from_str(a0.trim()).map_err(|_| ParsingError::KlobucharData)?;
            let b1 = f64::from_str(a1.trim()).map_err(|_| ParsingError::KlobucharData)?;
            let b2 = f64::from_str(a2.trim()).map_err(|_| ParsingError::KlobucharData)?;
            let b3 = f64::from_str(a3.trim()).map_err(|_| ParsingError::KlobucharData)?;

            Ok(Self::KlobucharModel(KbModel {
                alpha: (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64),
                beta: (b0, b1, b2, b3),
                region: KbRegionCode::WideArea,
            }))
        }
    }

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
    /// Returns reference to Klobuchar Model
    pub fn as_klobuchar(&self) -> Option<&KbModel> {
        match self {
            Self::KlobucharModel(model) => Some(model),
            _ => None,
        }
    }
    /// Returns mutable reference to Klobuchar Model
    pub fn as_klobuchar_mut(&mut self) -> Option<&mut KbModel> {
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
    #[test]
    fn rinex3_kb_header_parsing() {
        let kb = IonMessage::from_rinex3_header(
            "GPSA   7.4506e-09 -1.4901e-08 -5.9605e-08  1.1921e-07       ",
        );
        assert!(kb.is_ok(), "failed to parse GPSA iono correction header");
        let kb = kb.unwrap();
        assert_eq!(
            kb,
            IonMessage::KlobucharModel(KbModel {
                alpha: (7.4506E-9, -1.4901E-8, -5.9605E-8, 1.1921E-7),
                beta: (0.0, 0.0, 0.0, 0.0),
                region: KbRegionCode::WideArea,
            })
        );

        let kb = IonMessage::from_rinex3_header(
            "GPSB   9.0112e+04 -6.5536e+04 -1.3107e+05  4.5875e+05       ",
        );
        assert!(kb.is_ok(), "failed to parse GPSB iono correction header");
        let kb = kb.unwrap();
        assert_eq!(
            kb,
            IonMessage::KlobucharModel(KbModel {
                alpha: (0.0, 0.0, 0.0, 0.0),
                beta: (9.0112E4, -6.5536E4, -1.3107E5, 4.5875E5),
                region: KbRegionCode::WideArea,
            })
        );

        let kb = IonMessage::from_rinex3_header(
            "BDSA   1.1176e-08  2.9802e-08 -4.1723e-07  6.5565e-07       ",
        );
        assert!(kb.is_ok(), "failed to parse BDSA iono correction header");
        let kb = kb.unwrap();
        assert_eq!(
            kb,
            IonMessage::KlobucharModel(KbModel {
                alpha: (1.1176E-8, 2.9802E-8, -4.1723E-7, 6.5565E-7),
                beta: (0.0, 0.0, 0.0, 0.0),
                region: KbRegionCode::WideArea,
            })
        );

        let kb = IonMessage::from_rinex3_header(
            "BDSB   1.4131e+05 -5.2429e+05  1.6384e+06 -4.5875e+05   3   ",
        );
        assert!(kb.is_ok(), "failed to parse BDSB iono correction header");
        let kb = kb.unwrap();
        assert_eq!(
            kb,
            IonMessage::KlobucharModel(KbModel {
                alpha: (0.0, 0.0, 0.0, 0.0),
                beta: (1.4131E5, -5.2429E5, 1.6384E6, -4.5875E5),
                region: KbRegionCode::WideArea,
            })
        );

        /*
         * Test japanese (QZSS) orbital plan
         */
        let kb = IonMessage::from_rinex3_header(
            "QZSA   7.4506e-09 -1.4901e-08 -5.9605e-08  1.1921e-07       ",
        );
        assert!(kb.is_ok(), "failed to parse QZSA iono correction header");
        let kb = kb.unwrap();
        let kb = kb.as_klobuchar().unwrap();
        assert_eq!(
            kb.region,
            KbRegionCode::JapanArea,
            "QZSA ionospheric corr badly interprated as worldwide correction"
        );

        let kb = IonMessage::from_rinex3_header(
            "QZSB   9.0112e+04 -6.5536e+04 -1.3107e+05  4.5875e+05       ",
        );
        assert!(kb.is_ok(), "failed to parse QZSB iono correction header");
        let kb = kb.unwrap();
        let kb = kb.as_klobuchar().unwrap();
        assert_eq!(
            kb.region,
            KbRegionCode::JapanArea,
            "QZSB ionospheric corr badly interprated as worldwide correction"
        );
    }

    #[test]
    fn rinex2_kb_header_parsing() {
        for (line, alpha) in [
            // RINEX v2.10 and v2.11 standards (same string in both)
            // https://files.igs.org/pub/data/format/rinex211.txt
            (
                "     .1676D-07   .2235D-07  -.1192D-06  -.1192D-06          ",
                (0.1676E-07, 0.2235E-07, -0.1192E-06, -0.1192E-06),
            ),
            // ftp://igs.ign.fr/pub/igs/data/2024/119/zimm1190.24n.gz
            (
                "     .2515D-07   .1490D-07  -.1192D-06  -.5960D-07          ",
                (0.2515E-07, 0.1490E-07, -0.1192E-06, -0.5960E-07),
            ),
            // https://github.com/osqzss/gps-sdr-sim/blob/654a9888c54218766909e15fe8139e4bc8e83ecc/brdc0010.22n
            (
                "    0.1211D-07 -0.7451D-08 -0.5960D-07  0.1192D-06          ",
                (0.1211E-07, -0.7451E-08, -0.5960E-07, 0.1192E-06),
            ),
        ] {
            let kb = IonMessage::from_rinex2_header(line, "ION ALPHA           ");
            assert!(kb.is_ok(), "failed to parse ION ALPHA header");
            let kb = kb.unwrap();
            assert_eq!(
                kb,
                IonMessage::KlobucharModel(KbModel {
                    alpha,
                    beta: (0.0, 0.0, 0.0, 0.0),
                    region: KbRegionCode::WideArea,
                })
            );
        }

        for (line, beta) in [
            // RINEX v2.10 and v2.11 standards (same string in both)
            // https://files.igs.org/pub/data/format/rinex211.txt
            (
                "     .1208D+06   .1310D+06  -.1310D+06  -.1966D+06          ",
                (0.1208E+06, 0.1310E+06, -0.1310E+06, -0.1966E+06),
            ),
            // ftp://igs.ign.fr/pub/igs/data/2024/119/zimm1190.24n.gz
            (
                "     .1290D+06   .4915D+05  -.2621D+06   .1966D+06          ",
                (0.1290E+06, 0.4915E+05, -0.2621E+06, 0.1966E+06),
            ),
            // https://github.com/osqzss/gps-sdr-sim/blob/654a9888c54218766909e15fe8139e4bc8e83ecc/brdc0010.22n
            (
                "    0.1167D+06 -0.2458D+06 -0.6554D+05  0.1114D+07          ",
                (0.1167E+06, -0.2458E+06, -0.6554E+05, 0.1114E+07),
            ),
        ] {
            let kb = IonMessage::from_rinex2_header(line, "ION BETA            ");
            assert!(kb.is_ok(), "failed to parse ION BETA header");
            let kb = kb.unwrap();
            assert_eq!(
                kb,
                IonMessage::KlobucharModel(KbModel {
                    alpha: (0.0, 0.0, 0.0, 0.0),
                    beta,
                    region: KbRegionCode::WideArea,
                })
            );
        }
    }
}
