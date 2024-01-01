/*
 * RINEX File Production attributes.
 * Attached to RINEX files that follow standard naming conventions.
 * Also used in customized RINEX file production API.
 */
use super::Error;
use crate::marker::GeodeticMarker;
use hifitime::{Duration, Epoch, TimeScale, Unit};
use std::str::FromStr;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum DataSource {
    /// Source of data is hardware (radio) receiver.
    /// It can also represent a sensor in case of meteo observations.
    Receiver,
    /// Other stream source, like RTCM
    Stream,
    /// Unknown data source
    #[default]
    Unknown,
}

impl std::str::FromStr for DataSource {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        if content.eq("R") {
            Ok(Self::Receiver)
        } else if content.eq("S") {
            Ok(Self::Stream)
        } else {
            Ok(Self::Unknown)
        }
    }
}

impl std::fmt::Display for DataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Receiver => write!(f, "{}", 'R'),
            Self::Stream => write!(f, "{}", 'S'),
            Self::Unknown => write!(f, "{}", 'U'),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// PPU Gives information on file periodicity.
pub enum PPU {
    /// A Daily file is the standard and will contain 24h of data
    #[default]
    Daily,
    /// Contains 15' of data
    QuarterHour,
    /// Contains 1h of data
    Hourly,
    /// Contains 1 year of data
    Yearly,
    /// Unspecified
    Unspecified,
}

impl PPU {
    /// Returns this file periodicity as a [Duration]
    pub fn duration(&self) -> Option<Duration> {
        match self {
            Self::QuarterHour => Some(15 * Unit::Minute),
            Self::Hourly => Some(1 * Unit::Hour),
            Self::Daily => Some(1 * Unit::Day),
            Self::Yearly => Some(365 * Unit::Day),
            _ => None,
        }
    }
}

impl std::fmt::Display for PPU {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::QuarterHour => write!(f, "15M"),
            Self::Hourly => write!(f, "01H"),
            Self::Daily => write!(f, "01D"),
            Self::Yearly => write!(f, "O1Y"),
            Self::Unspecified => write!(f, "00U"),
        }
    }
}

impl std::str::FromStr for PPU {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "15M" => Ok(Self::QuarterHour),
            "01H" => Ok(Self::Hourly),
            "01D" => Ok(Self::Daily),
            "01Y" => Ok(Self::Yearly),
            _ => Ok(Self::Unspecified),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
// FFU gives information on the file sample rate.
// You should initialize your FFU from a [hifitime::Duration].
struct FFU {
    /// Sample rate
    pub val: u32,
    /// Period unit
    pub unit: Unit,
}

impl Default for FFU {
    fn default() -> Self {
        Self {
            val: 30,
            unit: Unit::Second,
        }
    }
}

impl std::fmt::Display for FFU {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.unit {
            Unit::Minute => write!(f, "{:02}M", self.val),
            Unit::Hour => write!(f, "{:02}H", self.val),
            Unit::Day => write!(f, "{:02}D", self.val),
            Unit::Second | _ => write!(f, "{:02}S", self.val),
        }
    }
}

impl From<Duration> for FFU {
    fn from(dur: Duration) -> Self {
        let secs = dur.to_seconds().round() as u32;
        if secs < 60 {
            Self {
                val: secs,
                unit: Unit::Second,
            }
        } else if secs < 3_600 {
            Self {
                val: secs / 60,
                unit: Unit::Minute,
            }
        } else if secs < 86_400 {
            Self {
                val: secs / 3_600,
                unit: Unit::Hour,
            }
        } else {
            Self {
                val: secs / 86_400,
                unit: Unit::Day,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RinexProductionAttributes {
    /// (Complete) Station name. Usually named after a geodetic marker.
    pub station: String,
    /// Data source
    pub data_src: DataSource,
    /// Epoch of file production
    pub production_epoch: Epoch,
    /// 3 letter country code
    pub country: String,
    /// PPU field gives information on the file periodicity
    pub ppu: PPU,
}

/*
 * Default values, that would still follow the specs.
 * This is useful to provide an infaillble API.
 */
impl Default for RinexProductionAttributes {
    fn default() -> Self {
        Self {
            station: "XXXX".to_string(),
            data_src: DataSource::default(),
            production_epoch: Epoch::from_str("2000-01-01T00:00:00 GPST").unwrap(),
            country: "CCC".to_string(),
            ppu: PPU::default(),
        }
    }
}

impl std::str::FromStr for RinexProductionAttributes {
    type Err = Error;
    fn from_str(fname: &str) -> Result<Self, Self::Err> {
        if fname.len() < 34 {
            return Err(Error::NonStandardFilename);
        }
        if fname.chars().nth(9) != Some('_') {
            return Err(Error::NonStandardFilename);
        }
        if fname.chars().nth(11) != Some('_') {
            return Err(Error::NonStandardFilename);
        }
        if fname.chars().nth(23) != Some('_') {
            return Err(Error::NonStandardFilename);
        }
        Ok(Self {
            station: fname[..6].trim().to_string(),
            country: fname[6..9].trim().to_string(),
            // data_src parsing is infaillible
            data_src: DataSource::from_str(&fname[10..11]).unwrap(),
            production_epoch: Epoch::default(), // FIXME
            ppu: PPU::from_str(&fname[24..27]).unwrap(),
        })
    }
}

impl RinexProductionAttributes {
    /*
    * This is used to generate a V3+ compliant filename
      Modern format is "XXXXMRCCC_R_YYYYDDDHHMM_PPU_FFU_ZZ."*
    */
    pub(crate) fn obs_filename(
        marker: Option<&GeodeticMarker>,
        r: u8, // receiver number
        country_code: &str,
        src: DataSource,
        epoch: Option<Epoch>,
        ppu: PPU,
        sample_rate: Duration,
        is_crinex: bool,
    ) -> String {
        let xxxx = match marker {
            Some(marker) => marker.name[..4].to_string(),
            None => "XXXX".to_string(),
        };
        let m = match marker {
            Some(marker) => {
                if let Some(number) = marker.number() {
                    let offset = number.find('M').unwrap() + 1;
                    number.chars().nth(offset).unwrap()
                } else {
                    '0'
                }
            },
            None => '0',
        };
        let (yyyy, ddd, hh, mm) = match epoch {
            Some(epoch) => {
                //FIXME: waiting on new hifitime release
                let mut ddd = Epoch::from_duration(epoch.to_utc_duration(), TimeScale::UTC)
                    .day_of_year()
                    .round() as u16;
                ddd %= 365; //FIXME: waiting on new hifitime release
                if ddd == 0 {
                    ddd += 1; //FIXME: waiting on new hifitime release
                }

                let (yyyy, _, _, hh, mm, _, _) = epoch.to_gregorian_utc();
                (
                    format!("{:04}", yyyy),
                    format!("{:03}", ddd),
                    format!("{:02}", hh),
                    format!("{:02}", mm),
                )
            },
            None => (
                "YYYY".to_string(),
                "DDD".to_string(),
                "HH".to_string(),
                "MM".to_string(),
            ),
        };
        let ffu = FFU::from(sample_rate);
        let ext = if is_crinex { "crx" } else { "rnx" };
        format!(
            "{}{}{}{}_{}_{:04}{}{}{}_{}_{}_MO.{}",
            xxxx, m, r, country_code, src, yyyy, ddd, hh, mm, ppu, ffu, ext
        )
    }
}

#[cfg(test)]
mod test {
    use super::DataSource;
    use super::RinexProductionAttributes;
    use super::PPU;
    use crate::marker::GeodeticMarker;
    use hifitime::{Epoch, Unit};
    use std::str::FromStr;
    // #[test]
    // fn ffu_from_duration() {
    //     for (dur, expected) in [
    //         (30 * Unit::Second, "30S"),
    //     ] {
    //         assert_eq!(
    //             FFU::from(dur).to_string(),
    //             expected,
    //         );
    //     }
    // }
    #[test]
    fn ppu() {
        /* test parser */
        for (c, expected, dur) in [
            ("15M", PPU::QuarterHour, Some(15 * Unit::Minute)),
            ("01H", PPU::Hourly, Some(1 * Unit::Hour)),
            ("01D", PPU::Daily, Some(1 * Unit::Day)),
            ("01Y", PPU::Yearly, Some(365 * Unit::Day)),
            ("XX", PPU::Unspecified, None),
            ("01U", PPU::Unspecified, None),
        ] {
            let ppu = PPU::from_str(c).unwrap();
            assert_eq!(ppu, expected);
            assert_eq!(ppu.duration(), dur);
        }
    }
    #[test]
    fn rinex_prod_attributes() {
        for (filename, station, country, data_src) in [
            (
                "ACOR00ESP_R_20213550000_01D_30S_MO",
                "ACOR00",
                "ESP",
                DataSource::Receiver,
            ),
            (
                "ESBC00DNK_R_20201770000_01D_30S_MO",
                "ESBC00",
                "DNK",
                DataSource::Receiver,
            ),
        ] {
            let attrs = RinexProductionAttributes::from_str(filename);
            assert!(attrs.is_ok());
            let attrs = attrs.unwrap();
            assert_eq!(attrs.country, country);
            assert_eq!(attrs.station, station);
            assert_eq!(attrs.data_src, data_src);
        }
    }
    #[test]
    fn rinex_prod_obsfile_gen() {
        assert_eq!(
            RinexProductionAttributes::obs_filename(
                Some(
                    GeodeticMarker::default()
                        .with_name("ESBC00DNK")
                        .with_number("10118M001")
                )
                .as_ref(),
                0,
                "DNK",
                DataSource::Receiver,
                None,
                PPU::Daily,
                30 * Unit::Second,
                false,
            ),
            "ESBC00DNK_R_YYYYDDDHHMM_01D_30S_MO.rnx" //FIXME: hifitime DOY
        );
        assert_eq!(
            RinexProductionAttributes::obs_filename(
                Some(
                    GeodeticMarker::default()
                        .with_name("ESBC00DNK")
                        .with_number("10118M001")
                )
                .as_ref(),
                0,
                "DNK",
                DataSource::Receiver,
                Some(Epoch::from_str("2020-06-25T00:00:00 GPST").unwrap()),
                PPU::Daily,
                30 * Unit::Second,
                false,
            ),
            "ESBC00DNK_R_20201762359_01D_30S_MO.rnx" //FIXME: hifitime DOY
        );
    }
}
