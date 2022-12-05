use super::prelude::*;

pub trait TimeScaling<T> {
    /// Copies self and converts all Epochs to desired
    /// [hifitime::TimeScale].
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::gnss_time::*;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx")
    ///     .unwrap();
    /// // default definition
    /// assert_eq!(rnx.timescale(), Some(TimeScale::UTC));
    /// ```
    fn with_timescale(&self, ts: TimeScale) -> Self;
    /// Converts converts all Epochs to desired
    /// [hifitime::TimeScale].
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::gnss_time::*;
    /// let mut rnx = Rinex::from_file("../test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx")
    ///     .unwrap();
    /// // default definition
    /// assert_eq!(rnx.timescale(), Some(TimeScale::UTC));
    /// ```
    fn convert_timescale(&mut self, ts: TimeScale);
}

/*

impl GnssTime {
    /// Corrects self to given reference using given correction parameters
    /// correction: correction to be applied
    /// reference: reference time (must match expected reference)
    /// TODO: refer to p39
    pub fn correct (&mut self, correction: &GnssTimeCorrection, reference: &GnssTime) -> Result<(), Error> {
        // check this is the expected reference time
        match correction.corr_type {
            TimeCorrectionType::GPUT => {
                // check time system matches the expected one
                match self.gnss {
                    constellation::Constellation::GPS => {},
                    _ => return Err(Error::CorrectionTimeSystemError),
                }
            },
            TimeCorrectionType::GAUT => {
                // check time system matches the expected one
                match self.gnss {
                    constellation::Constellation::Galileo => {},
                    _ => return Err(Error::CorrectionTimeSystemError),
                }
            },
            TimeCorrectionType::SBUT => {
                // check time system matches the expected one
                match self.gnss {
                    constellation::Constellation::Sbas => {},
                    _ => return Err(Error::CorrectionTimeSystemError),
                }
            },
            TimeCorrectionType::GLUT => {
                // check time system matches the expected one
                match self.gnss {
                    constellation::Constellation::Glonass => {},
                    _ => return Err(Error::CorrectionTimeSystemError),
                }
            },
            TimeCorrectionType::GPGA => {
                if reference.gnss != constellation::Constellation::Galileo {
                    return Err(Error::CorrectionTimeReferenceError)
                }
                // check time system matches the expected one
                match self.gnss {
                    constellation::Constellation::GPS => {},
                    _ => return Err(Error::CorrectionTimeSystemError),
                }
            },
            TimeCorrectionType::GLGP => {
                if reference.gnss != constellation::Constellation::GPS {
                    return Err(Error::CorrectionTimeReferenceError)
                }
                // check time system matches the expected one
                match self.gnss {
                    constellation::Constellation::Glonass => {},
                    _ => return Err(Error::CorrectionTimeSystemError),
                }
            },
            TimeCorrectionType::GZUT => {
                // check time system matches the expected one
                match self.gnss {
                    constellation::Constellation::QZSS => {},
                    _ => return Err(Error::CorrectionTimeSystemError),
                }
            },
        }
        Ok(())
    }
}

/// List of known correction types:
/// GPUT: GPS->UTC  (a0,a1)
/// GAUT: GAL->UTC  (a0,a1)
/// SBUT: SBAS->UTC (a0,a1)
/// GLUT: GLO->UTC  a0=τ(c) a1=0
/// GPGA: GPS->GAL  a0=a0g  a1=a1g
/// GLGP: GLO->GPS  a0=τ(gps) a1=zero
/// GZUT: QZS->UTC  a0,a1
pub enum TimeCorrectionType {
    GPUT,
    GAUT,
    SBUT,
    GLUT,
    GPGA,
    GLGP,
    GZUT,
}

impl std::str::FromStr for TimeCorrectionType {
    type Err = Error;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("GPUT") {
            Ok(TimeCorrectionType::GPUT)
        } else if s.eq("GAUT") {
            Ok(TimeCorrectionType::GAUT)
        } else if s.eq("SBUT") {
            Ok(TimeCorrectionType::SBUT)
        } else if s.eq("GLUT") {
            Ok(TimeCorrectionType::GLUT)
        } else if s.eq("GPGA") {
            Ok(TimeCorrectionType::GPGA)
        } else if s.eq("GLGP") {
            Ok(TimeCorrectionType::GLGP)
        } else if s.eq("GZUT") {
            Ok(TimeCorrectionType::GZUT)
        } else {
            Err(Error::InvalidTimeSystem(s.to_string()))
        }
    }
}

/// Describes known UTC providers
/// (laboratories)
pub enum UtcProvider {
    Unknown,
    NIST,
    USNO,
    SU,
    BIPM,
    EuropeLab,
    CRL,
}

impl std::str::FromStr for UtcProvider {
    type Err = Error;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("1") {
            Ok(UtcProvider::NIST)
        } else if s.eq("2") {
            Ok(UtcProvider::USNO)
        } else if s.eq("3") {
            Ok(UtcProvider::SU)
        } else if s.eq("4") {
            Ok(UtcProvider::BIPM)
        } else if s.eq("5") {
            Ok(UtcProvider::EuropeLab)
        } else if s.eq("6") {
            Ok(UtcProvider::CRL)
        } else {
            Ok(UtcProvider::Unknown)
        }
    }
}

/// Not documented ?
pub enum AugmentationSystem {
    EGNOS,
    WAAS,
    MSAS,
}

impl std::str::FromStr for AugmentationSystem {
    type Err = Error;
    /// Builds `AugmentationSystem` from string
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("EGNOS") {
            Ok(AugmentationSystem::EGNOS)
        } else if s.eq("WAAS") {
            Ok(AugmentationSystem::WAAS)
        } else if s.eq("MSAS") {
            Ok(AugmentationSystem::MSAS)
        } else {
            Err(Error::UnknownAugmentationSystem(s.to_string()))
        }
    }
}

/// `GnssTimeCorrection` describes
/// GNSS Time System corrections.
/// `system` : XXYY: XX corrected to YY
/// (a0, a1): correction params ((s), (s.s⁻¹))
/// delta_t: correction param
/// week: week number counter
/// `augmentation system`: (EGNOS,WAAS,MSAS)
/// utc_provider: provider identifier
#[allow(dead_code)]
pub struct GnssTimeCorrection {
    corr_type: TimeCorrectionType,
    params: (f64,f64),
    delta_t: u32,
    week: u32,
    augmentation: Option<AugmentationSystem>,
    utc_provider: Option<UtcProvider>,
}

impl Default for GnssTimeCorrection {
    fn default() -> GnssTimeCorrection {
        GnssTimeCorrection {
            corr_type: TimeCorrectionType::GPUT,
            params: (0.0_f64, 0.0_f64),
            delta_t: 0,
            week: 0,
            augmentation: None,
            utc_provider: None,
        }
    }
}

impl std::str::FromStr for GnssTimeCorrection {
    type Err = Error;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        Ok(GnssTimeCorrection::default())
        /*
SBUT  0.1331791282D-06 0.107469589D-12 552960 1025 EGNOS  5 TIME SYSTEM CORR
        let systype = TimeCorrectionType::from_str()?;
        let params ...
        let delta_t..
        let week..
        if not null:
            AugmentationSystem::from_str()?
        if not null:
            UtcProvider::from_str()?
        */
    }
}
*/
