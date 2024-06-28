#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docrs, feature(doc_cfg))]
#![allow(clippy::type_complexity)]

extern crate gnss_rs as gnss;

#[cfg(feature = "qc")]
extern crate rinex_qc_traits as qc_traits;

pub mod antex;
pub mod carrier;
pub mod clock;
pub mod doris;
pub mod epoch;
pub mod gnss_time;
pub mod hardware;
pub mod hatanaka;
pub mod header;
pub mod ionex;
pub mod marker;
pub mod merge;
pub mod meteo;
pub mod navigation;
pub mod observation;
pub mod record;
pub mod split;
pub mod types;
pub mod version;

mod bibliography;
mod constants;
mod ground_position;
mod leap; // leap second
mod linspace; // grid and linear spacing
mod observable;
mod production; // RINEX production infrastructure // physical observations

#[cfg(test)]
mod tests;

#[macro_use]
mod macros;

extern crate num;

#[macro_use]
extern crate num_derive;

#[macro_use]
extern crate lazy_static;

pub mod reader;
use reader::BufferedReader;

pub mod writer;
use writer::BufferedWriter;

use std::collections::{BTreeMap, HashMap};
use std::io::Write; //, Read};
use std::path::Path;
use std::str::FromStr;

use itertools::Itertools;
use thiserror::Error;

use antex::{Antenna, AntennaSpecific, FrequencyDependentData};
use doris::record::ObservationData as DorisObservationData;
use epoch::epoch_decompose;
use ionex::TECPlane;
use navigation::NavFrame;
use observable::Observable;
use observation::{Crinex, ObservationData};
use version::Version;

use production::{DataSource, DetailedProductionAttributes, ProductionAttributes, FFU, PPU};

use hifitime::Unit;
//use hifitime::{efmt::Format as EpochFormat, efmt::Formatter as EpochFormatter, Duration, Unit};

/// Package to include all basic structures
pub mod prelude {
    #[cfg(feature = "antex")]
    pub use crate::antex::AntennaMatcher;
    #[cfg(feature = "clock")]
    pub use crate::clock::{ClockKey, ClockProfile, ClockProfileType, ClockType, WorkClock};
    #[cfg(feature = "doris")]
    pub use crate::doris::Station;
    pub use crate::ground_position::GroundPosition;
    pub use crate::header::Header;
    pub use crate::observable::Observable;
    pub use crate::observation::EpochFlag;
    pub use crate::types::Type as RinexType;
    pub use crate::{Error, Rinex};
    pub use gnss::prelude::{Constellation, DOMESTrackingPoint, COSPAR, DOMES, SV};
    pub use hifitime::{Duration, Epoch, TimeScale, TimeSeries};
}

/// Package dedicated to file production.
pub mod prod {
    pub use crate::production::{
        DataSource, DetailedProductionAttributes, ProductionAttributes, FFU, PPU,
    };
}

#[cfg(feature = "processing")]
use qc_traits::processing::{Decimate, DecimationFilter, MaskFilter, Masking, Preprocessing};

#[cfg(feature = "processing")]
use crate::{
    clock::record::{clock_decim_mut, clock_mask_mut},
    doris::record::{doris_decim_mut, doris_mask_mut},
    header::header_mask_mut,
    ionex::record::{ionex_decim_mut, ionex_mask_mut},
    meteo::record::{meteo_decim_mut, meteo_mask_mut},
    navigation::record::{navigation_decim_mut, navigation_mask_mut},
    observation::record::{observation_decim_mut, observation_mask_mut},
};

use carrier::Carrier;
use prelude::*;

pub use merge::Merge;
pub use split::Split;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[cfg(docrs)]
pub use bibliography::Bibliography;

/*
 * returns true if given line is a comment
 */
pub(crate) fn is_rinex_comment(content: &str) -> bool {
    content.len() > 60 && content.trim_end().ends_with("COMMENT")
}

/*
 * macro to format one header line or a comment
 */
pub(crate) fn fmt_rinex(content: &str, marker: &str) -> String {
    if content.len() < 60 {
        format!("{:<padding$}{}", content, marker, padding = 60)
    } else {
        let mut string = String::new();
        let nb_lines = num_integer::div_ceil(content.len(), 60);
        for i in 0..nb_lines {
            let start_off = i * 60;
            let end_off = std::cmp::min(start_off + 60, content.len());
            let chunk = &content[start_off..end_off];
            string.push_str(&format!("{:<padding$}{}", chunk, marker, padding = 60));
            if i < nb_lines - 1 {
                string.push('\n');
            }
        }
        string
    }
}

/*
 * macro to generate comments with standardized formatting
 */
pub(crate) fn fmt_comment(content: &str) -> String {
    fmt_rinex(content, "COMMENT")
}

#[derive(Clone, Default, Debug, PartialEq)]
/// `Rinex` describes a `RINEX` file, it comprises a [Header] section,
/// and a [record::Record] file body.   
/// This parser can also store comments encountered while parsing the file body,
/// stored as [record::Comments], without much application other than presenting
/// all encountered data at the moment.   
/// Following is an example of high level usage (mainly header fields).  
/// For each RINEX type you get a method named after that type, which exposes
/// the whole dataset, for example [`Self::meteo`] for Meteo RINEX.
/// Other (high level information, calculations) are type dependent and
/// contained in a specific crate feature.
/// ```
/// use rinex::prelude::*;
/// let rnx = Rinex::from_file("../test_resources/OBS/V2/delf0010.21o")
///     .unwrap();
/// // header contains high level information
/// // like file standard revision:
/// assert_eq!(rnx.header.version.major, 2);
/// assert_eq!(rnx.header.version.minor, 11);
/// // general informations
/// assert_eq!(rnx.header.program, "teqc  2019Feb25");
/// assert_eq!(rnx.header.run_by, "Unknown"); // field was empty
/// // File creation date, temporarily stored as a String
/// // value, but that will soon change
/// assert_eq!(rnx.header.date, "20210102 00:01:40UTC");
/// assert_eq!(rnx.header.observer, "H. VAN DER MAREL");
///
/// let marker = rnx.header.geodetic_marker
///         .as_ref()
///         .unwrap();
/// assert_eq!(marker.number(), Some("13502M004".to_string()));
///
/// // Constellation describes which kind of vehicles
/// // are to be encountered in the record, or which
/// // GNSS constellation the data will be referred to.
/// // Mixed constellation, means a combination of vehicles or
/// // GNSS constellations is expected
/// assert_eq!(rnx.header.constellation, Some(Constellation::Mixed));
/// // Some information on the hardware being used might be stored
/// println!("{:#?}", rnx.header.rcvr);
/// // WGS84 receiver approximate position
/// println!("{:#?}", rnx.header.ground_position);
/// // comments encountered in the Header section
/// println!("{:#?}", rnx.header.comments);
/// // sampling interval was set
/// assert_eq!(rnx.header.sampling_interval, Some(Duration::from_seconds(30.0))); // 30s sample rate
/// // record content is RINEX format dependent.
/// // This one is Observation RINEX.
/// // Refer to [record::Record] definitions, to understand
/// // how to browse all RINEX records.
/// let record = rnx.record.as_obs()
///     .unwrap();
/// for (epoch, (clk_offset, observations)) in record {
///     // Do something
/// }
/// // comments encountered in file body
/// // are currently stored like this and indexed by epoch of "appearance"
/// // they are currently not really exploited
/// for (epoch, comment) in rnx.comments {
///     println!("{:?}: \"{:?}\"", epoch, comment);
/// }
/// ```
pub struct Rinex {
    /// `header` field contains general information
    pub header: Header,
    /// `comments` : list of extra readable information,   
    /// found in `record` section exclusively.    
    /// Comments extracted from `header` sections are exposed in `header.comments`
    pub comments: record::Comments,
    /// `record` contains `RINEX` file body
    /// and is type and constellation dependent
    pub record: record::Record,
    /*
     * File Production attributes, attached to Self
     * parsed from files that follow stadard naming conventions
     */
    prod_attr: Option<ProductionAttributes>,
}

#[derive(Error, Debug)]
/// `RINEX` Parsing related errors
pub enum Error {
    #[error("header parsing error")]
    HeaderParsingError(#[from] header::ParsingError),
    #[error("record parsing error")]
    RecordError(#[from] record::Error),
    #[error("file i/o error")]
    IoError(#[from] std::io::Error),
}

impl Rinex {
    /// Builds a new `RINEX` struct from given header & body sections.
    pub fn new(header: Header, record: record::Record) -> Rinex {
        Rinex {
            header,
            record,
            comments: record::Comments::new(),
            prod_attr: None,
        }
    }
    /// Returns a copy of self with given header attributes.
    pub fn with_header(&self, header: Header) -> Self {
        Self {
            header,
            record: self.record.clone(),
            comments: self.comments.clone(),
            prod_attr: self.prod_attr.clone(),
        }
    }
    /// Replaces header section.
    pub fn replace_header(&mut self, header: Header) {
        self.header = header.clone();
    }
    /// Returns a copy of self with given internal record.
    pub fn with_record(&self, record: record::Record) -> Self {
        Rinex {
            header: self.header.clone(),
            comments: self.comments.clone(),
            record,
            prod_attr: self.prod_attr.clone(),
        }
    }
    /// Replaces internal record.
    pub fn replace_record(&mut self, record: record::Record) {
        self.record = record.clone();
    }
    /// Converts self to CRINEX (compressed RINEX) format.
    /// If current revision is < 3 then file gets converted to CRINEX1
    /// format, otherwise, modern Observations are converted to CRINEX3.
    /// This has no effect if self is not an Observation RINEX.
    ///
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    ///
    /// // convert to CRINEX
    /// let crinex = rinex.rnx2crnx();
    /// assert!(crinex.to_file("test.crx").is_ok());
    /// ```
    pub fn rnx2crnx(&self) -> Self {
        let mut s = self.clone();
        s.rnx2crnx_mut();
        s
    }
    /// [`Self::rnx2crnx`] mutable implementation
    pub fn rnx2crnx_mut(&mut self) {
        if self.is_observation_rinex() {
            let mut crinex = Crinex::default();
            crinex.version.major = match self.header.version.major {
                1 | 2 => 1,
                _ => 3,
            };
            self.header = self.header.with_crinex(crinex);
        }
    }

    /// Converts self to CRINEX1 compressed format,
    /// whatever the RINEX revision might be.  
    /// This can be used to "force" compression of a RINEX1 into CRINEX3
    pub fn rnx2crnx1(&self) -> Self {
        let mut s = self.clone();
        s.rnx2crnx1_mut();
        s
    }

    /// [`Self::rnx2crnx1`] mutable implementation.
    pub fn rnx2crnx1_mut(&mut self) {
        if self.is_observation_rinex() {
            self.header = self.header.with_crinex(Crinex {
                version: Version { major: 1, minor: 0 },
                date: epoch::now(),
                prog: format!("rust-rinex-{}", env!("CARGO_PKG_VERSION")),
            });
        }
    }

    /// Converts self to CRINEX3 compressed format,
    /// whatever the RINEX revision might be.
    /// This can be used to "force" compression of a RINEX1 into CRINEX3
    pub fn rnx2crnx3(&self) -> Self {
        let mut s = self.clone();
        s.rnx2crnx1_mut();
        s
    }

    /// [`Self::rnx2crnx3`] mutable implementation.
    pub fn rnx2crnx3_mut(&mut self) {
        if self.is_observation_rinex() {
            self.header = self.header.with_crinex(Crinex {
                date: epoch::now(),
                version: Version { major: 3, minor: 0 },
                prog: "rust-crinex".to_string(),
            });
        }
    }

    /// Converts a CRINEX (compressed RINEX) into readable RINEX.
    /// This has no effect if self is not an Observation RINEX.
    pub fn crnx2rnx(&self) -> Self {
        let mut s = self.clone();
        s.crnx2rnx_mut();
        s
    }

    /// [Rinex::crnx2rnx] mutable implementation
    pub fn crnx2rnx_mut(&mut self) {
        if self.is_observation_rinex() {
            let params = self.header.obs.as_ref().unwrap();
            self.header = self
                .header
                .with_observation_fields(observation::HeaderFields {
                    crinex: None,
                    codes: params.codes.clone(),
                    clock_offset_applied: params.clock_offset_applied,
                    scaling: params.scaling.clone(),
                    time_of_first_obs: params.time_of_first_obs,
                    time_of_last_obs: params.time_of_last_obs,
                });
        }
    }
    /// Returns a filename that would describe Self according to standard naming conventions.
    /// For this information to be 100% complete, Self must come from a file
    /// that follows these conventions itself.
    /// Otherwise you must provide [ProductionAttributes] yourself with "custom".
    /// In any case, this method is infaillible. You will just lack more or
    /// less information, depending on current context.
    /// If you're working with Observation, Navigation or Meteo data,
    /// and prefered shorter filenames (V2 like format): force short to "true".
    /// Otherwse, we will prefer modern V3 like formats.
    /// Use "suffix" to append a custom suffix like ".gz" for example.
    /// NB this will only output uppercase filenames (as per standard specs).
    /// ```
    /// use rinex::prelude::*;
    /// // Parse a File that follows standard naming conventions
    /// // and verify we generate something correct
    /// ```
    pub fn standard_filename(
        &self,
        short: bool,
        suffix: Option<&str>,
        custom: Option<ProductionAttributes>,
    ) -> String {
        let header = &self.header;
        let rinextype = header.rinex_type;
        let is_crinex = header.is_crinex();
        let constellation = header.constellation;

        let mut filename = match rinextype {
            RinexType::IonosphereMaps => {
                let name = match custom {
                    Some(ref custom) => {
                        custom.name[..std::cmp::min(3, custom.name.len())].to_string()
                    },
                    None => {
                        if let Some(attr) = &self.prod_attr {
                            attr.name.clone()
                        } else {
                            "XXX".to_string()
                        }
                    },
                };
                let region = match &custom {
                    Some(ref custom) => custom.region.unwrap_or('G'),
                    None => {
                        if let Some(attr) = &self.prod_attr {
                            attr.region.unwrap_or('G')
                        } else {
                            'G'
                        }
                    },
                };
                let ddd = match &custom {
                    Some(ref custom) => format!("{:03}", custom.doy),
                    None => {
                        if let Some(epoch) = self.first_epoch() {
                            let ddd = epoch.day_of_year().round() as u32;
                            format!("{:03}", ddd)
                        } else {
                            "DDD".to_string()
                        }
                    },
                };
                let yy = match &custom {
                    Some(ref custom) => format!("{:02}", custom.year - 2_000),
                    None => {
                        if let Some(epoch) = self.first_epoch() {
                            let yy = epoch_decompose(epoch).0;
                            format!("{:02}", yy - 2_000)
                        } else {
                            "YY".to_string()
                        }
                    },
                };
                ProductionAttributes::ionex_format(&name, region, &ddd, &yy)
            },
            RinexType::ObservationData | RinexType::MeteoData | RinexType::NavigationData => {
                let name = match custom {
                    Some(ref custom) => custom.name.clone(),
                    None => {
                        if let Some(attr) = &self.prod_attr {
                            attr.name.clone()
                        } else {
                            "XXXX".to_string()
                        }
                    },
                };
                let ddd = match &custom {
                    Some(ref custom) => format!("{:03}", custom.doy),
                    None => {
                        if let Some(epoch) = self.first_epoch() {
                            let ddd = epoch.day_of_year().round() as u32;
                            format!("{:03}", ddd)
                        } else {
                            "DDD".to_string()
                        }
                    },
                };
                if short {
                    let yy = match &custom {
                        Some(ref custom) => format!("{:02}", custom.year - 2_000),
                        None => {
                            if let Some(epoch) = self.first_epoch() {
                                let yy = epoch_decompose(epoch).0;
                                format!("{:02}", yy - 2_000)
                            } else {
                                "YY".to_string()
                            }
                        },
                    };
                    let ext = match rinextype {
                        RinexType::ObservationData => {
                            if is_crinex {
                                'D'
                            } else {
                                'O'
                            }
                        },
                        RinexType::MeteoData => 'M',
                        RinexType::NavigationData => match constellation {
                            Some(Constellation::Glonass) => 'G',
                            _ => 'N',
                        },
                        _ => unreachable!("unreachable"),
                    };
                    ProductionAttributes::rinex_short_format(&name, &ddd, &yy, ext)
                } else {
                    /* long /V3 like format */
                    let batch = match &custom {
                        Some(ref custom) => {
                            if let Some(details) = &custom.details {
                                details.batch
                            } else {
                                0
                            }
                        },
                        None => {
                            if let Some(attr) = &self.prod_attr {
                                if let Some(details) = &attr.details {
                                    details.batch
                                } else {
                                    0
                                }
                            } else {
                                0
                            }
                        },
                    };
                    let country = match &custom {
                        Some(ref custom) => {
                            if let Some(details) = &custom.details {
                                details.country.to_string()
                            } else {
                                "CCC".to_string()
                            }
                        },
                        None => {
                            if let Some(attr) = &self.prod_attr {
                                if let Some(details) = &attr.details {
                                    details.country.to_string()
                                } else {
                                    "CCC".to_string()
                                }
                            } else {
                                "CCC".to_string()
                            }
                        },
                    };
                    let src = match &header.rcvr {
                        Some(_) => 'R', // means GNSS rcvr
                        None => {
                            if let Some(attr) = &self.prod_attr {
                                if let Some(details) = &attr.details {
                                    details.data_src.to_char()
                                } else {
                                    'U' // means unspecified
                                }
                            } else {
                                'U' // means unspecified
                            }
                        },
                    };
                    let yyyy = match &custom {
                        Some(ref custom) => format!("{:04}", custom.year),
                        None => {
                            if let Some(epoch) = self.first_epoch() {
                                let yy = epoch_decompose(epoch).0;
                                format!("{:04}", yy)
                            } else {
                                "YYYY".to_string()
                            }
                        },
                    };
                    let (hh, mm) = match &custom {
                        Some(ref custom) => {
                            if let Some(details) = &custom.details {
                                (format!("{:02}", details.hh), format!("{:02}", details.mm))
                            } else {
                                ("HH".to_string(), "MM".to_string())
                            }
                        },
                        None => {
                            if let Some(epoch) = self.first_epoch() {
                                let (_, _, _, hh, mm, _, _) = epoch_decompose(epoch);
                                (format!("{:02}", hh), format!("{:02}", mm))
                            } else {
                                ("HH".to_string(), "MM".to_string())
                            }
                        },
                    };
                    // FFU sampling rate
                    let ffu = match self.dominant_sample_rate() {
                        Some(duration) => FFU::from(duration).to_string(),
                        None => {
                            if let Some(ref custom) = custom {
                                if let Some(details) = &custom.details {
                                    if let Some(ffu) = details.ffu {
                                        ffu.to_string()
                                    } else {
                                        "XXX".to_string()
                                    }
                                } else {
                                    "XXX".to_string()
                                }
                            } else {
                                "XXX".to_string()
                            }
                        },
                    };
                    // ffu only in OBS file names
                    let ffu = match rinextype {
                        RinexType::ObservationData => Some(ffu),
                        _ => None,
                    };
                    // PPU periodicity
                    let ppu = if let Some(ref custom) = custom {
                        if let Some(details) = &custom.details {
                            details.ppu
                        } else {
                            PPU::Unspecified
                        }
                    } else if let Some(ref attr) = self.prod_attr {
                        if let Some(details) = &attr.details {
                            details.ppu
                        } else {
                            PPU::Unspecified
                        }
                    } else {
                        PPU::Unspecified
                    };
                    let fmt = match rinextype {
                        RinexType::ObservationData => "MO".to_string(),
                        RinexType::MeteoData => "MM".to_string(),
                        RinexType::NavigationData => match constellation {
                            Some(Constellation::Mixed) | None => "MN".to_string(),
                            Some(constell) => format!("M{:x}", constell),
                        },
                        _ => unreachable!("unreachable fmt"),
                    };
                    let ext = if is_crinex { "crx" } else { "rnx" };
                    ProductionAttributes::rinex_long_format(
                        &name,
                        batch,
                        &country,
                        src,
                        &yyyy,
                        &ddd,
                        &hh,
                        &mm,
                        &ppu.to_string(),
                        ffu.as_deref(),
                        &fmt,
                        ext,
                    )
                }
            },
            rinex => unimplemented!("{} format", rinex),
        };
        if let Some(suffix) = suffix {
            filename.push_str(suffix);
        }
        filename
    }

    /// Guesses File [ProductionAttributes] from the actual Record content.
    /// This is particularly useful when working with datasets we are confident about,
    /// yet that do not follow standard naming conventions.
    /// Here is an example of such use case:
    /// ```
    /// use rinex::prelude::*;
    ///
    /// // Parse file that does not follow naming conventions
    /// let rinex = Rinex::from_file("../test_resources/MET/V4/example1.txt");
    /// assert!(rinex.is_ok()); // As previously stated, we totally accept that
    /// let rinex = rinex.unwrap();
    ///
    /// // The standard filename generator has no means to generate something correct.
    /// let standard_name = rinex.standard_filename(true, None, None);
    /// assert_eq!(standard_name, "XXXX0070.21M");
    ///
    /// // Now use the smart attributes detector as custom attributes
    /// let guessed = rinex.guess_production_attributes();
    /// let standard_name = rinex.standard_filename(true, None, Some(guessed.clone()));
    ///
    /// // Short name are always correctly determined
    /// assert_eq!(standard_name, "bako0070.21M");
    ///
    /// // Modern (lengthy) names have fields like the Country code that cannot be recovered
    /// // if the original file did not follow standard conventions itself.
    /// let standard_name = rinex.standard_filename(false, None, Some(guessed.clone()));
    /// assert_eq!(standard_name, "bako00XXX_U_20210070000_00U_MM.rnx");
    /// ```
    pub fn guess_production_attributes(&self) -> ProductionAttributes {
        // start from content identified from the filename
        let mut attributes = self.prod_attr.clone().unwrap_or_default();

        let first_epoch = self.first_epoch();
        let last_epoch = self.last_epoch();
        let first_epoch_gregorian = first_epoch.map(|t0| t0.to_gregorian_utc());

        match first_epoch_gregorian {
            Some((y, _, _, _, _, _, _)) => attributes.year = y as u32,
            _ => {},
        }
        match first_epoch {
            Some(t0) => attributes.doy = t0.day_of_year().round() as u32,
            _ => {},
        }
        // notes on attribute."name"
        // - Non detailed OBS RINEX: this is usually the station name
        //   which can be named after a geodetic marker
        // - Non detailed NAV RINEX: station name
        // - CLK RINEX: name of the local clock
        // - IONEX: agency
        match self.header.rinex_type {
            RinexType::ClockData => match &self.header.clock {
                Some(clk) => match &clk.ref_clock {
                    Some(refclock) => attributes.name = refclock.to_string(),
                    _ => {
                        if let Some(site) = &clk.site {
                            attributes.name = site.to_string();
                        } else {
                            attributes.name = self.header.agency.to_string();
                        }
                    },
                },
                _ => attributes.name = self.header.agency.to_string(),
            },
            RinexType::IonosphereMaps => {
                attributes.name = self.header.agency.to_string();
            },
            _ => match &self.header.geodetic_marker {
                Some(marker) => attributes.name = marker.name.to_string(),
                _ => attributes.name = self.header.agency.to_string(),
            },
        }
        if let Some(ref mut details) = attributes.details {
            if let Some((_, _, _, hh, mm, _, _)) = first_epoch_gregorian {
                details.hh = hh;
                details.mm = mm;
            }
            if let Some(first_epoch) = first_epoch {
                if let Some(last_epoch) = last_epoch {
                    let total_dt = last_epoch - first_epoch;
                    details.ppu = PPU::from(total_dt);
                }
            }
        } else {
            attributes.details = Some(DetailedProductionAttributes {
                batch: 0,                      // see notes down below
                country: "XXX".to_string(),    // see notes down below
                data_src: DataSource::Unknown, // see notes down below
                ppu: match (first_epoch, last_epoch) {
                    (Some(first), Some(last)) => {
                        let total_dt = last - first;
                        PPU::from(total_dt)
                    },
                    _ => PPU::Unspecified,
                },
                ffu: self.dominant_sample_rate().map(FFU::from),
                hh: match first_epoch_gregorian {
                    Some((_, _, _, hh, _, _, _)) => hh,
                    _ => 0,
                },
                mm: match first_epoch_gregorian {
                    Some((_, _, _, _, mm, _, _)) => mm,
                    _ => 0,
                },
            });
        }
        /*
         * Several fields cannot be deduced from the actual
         * Record content. If provided filename did not describe them,
         * we have no means to recover them.
         * Example of such fields would be:
         *    + Country Code: would require a worldwide country database
         *    + Data source: is only defined in the filename
         */
        attributes
    }

    /// Builds a `RINEX` from given file fullpath.
    /// Header section must respect labelization standards,
    /// some are mandatory.   
    /// Parses record (file body) for supported `RINEX` types.
    pub fn from_file(fullpath: &str) -> Result<Rinex, Error> {
        Self::from_path(Path::new(fullpath))
    }

    /// See [Self::from_file]
    pub fn from_path(path: &Path) -> Result<Rinex, Error> {
        let fullpath = path.to_string_lossy().to_string();

        // create buffered reader
        let mut reader = BufferedReader::new(&fullpath)?;

        // Parse header fields
        let mut header = Header::new(&mut reader)?;

        // Parse file body (record content)
        // Comments might serve some fileops like "splice".
        let (record, comments) = record::parse_record(&mut reader, &mut header)?;

        // Parse / identify production attributes
        // that only exist in the filename.
        let prod_attr = match path.file_name() {
            Some(filename) => {
                let filename = filename.to_string_lossy().to_string();
                if let Ok(attrs) = ProductionAttributes::from_str(&filename) {
                    Some(attrs)
                } else {
                    None
                }
            },
            _ => None,
        };

        Ok(Rinex {
            header,
            record,
            comments,
            prod_attr,
        })
    }

    /// Returns true if this is an ATX RINEX
    pub fn is_antex(&self) -> bool {
        self.header.rinex_type == types::Type::AntennaData
    }

    /// Returns true if this is a CLOCK RINEX
    pub fn is_clock_rinex(&self) -> bool {
        self.header.rinex_type == types::Type::ClockData
    }

    /// Returns true if Self is a IONEX
    pub fn is_ionex(&self) -> bool {
        self.header.rinex_type == types::Type::IonosphereMaps
    }

    /// Returns true if Self is a 3D IONEX.  
    /// In this case, you can have TEC values at different altitudes, for a given Epoch.
    pub fn is_ionex_3d(&self) -> bool {
        if let Some(ionex) = &self.header.ionex {
            ionex.map_dimension == 3
        } else {
            false
        }
    }

    /// Returns true if Self is a 2D IONEX.
    /// In this case, all TEC values are presented at the same altitude points.
    pub fn is_ionex_2d(&self) -> bool {
        if let Some(ionex) = &self.header.ionex {
            ionex.map_dimension == 2
        } else {
            false
        }
    }

    /// Returns true if this is a METEO RINEX
    pub fn is_meteo_rinex(&self) -> bool {
        self.header.rinex_type == types::Type::MeteoData
    }

    /// Retruns true if this is a NAV RINEX
    pub fn is_navigation_rinex(&self) -> bool {
        self.header.rinex_type == types::Type::NavigationData
    }

    /// Retruns true if this is an OBS RINEX
    pub fn is_observation_rinex(&self) -> bool {
        self.header.rinex_type == types::Type::ObservationData
    }

    /// Generates a new RINEX = Self(=RINEX(A)) - RHS(=RINEX(B)).
    /// Therefore RHS is considered reference.
    /// This operation is typically used to compare two GNSS receivers.
    /// Both RINEX formats must match otherwise this will panic.
    /// This is only available to Observation RINEX files.
    pub fn substract(&self, rhs: &Self) -> Self {
        let mut record = observation::Record::default();
        let lhs_rec = self
            .record
            .as_obs()
            .expect("can only substract observation data");

        let rhs_rec = rhs
            .record
            .as_obs()
            .expect("can only substract observation data");

        for ((epoch, flag), (clk, svnn)) in lhs_rec {
            if let Some((ref_clk, ref_svnn)) = rhs_rec.get(&(*epoch, *flag)) {
                for (sv, observables) in svnn {
                    if let Some(ref_observables) = ref_svnn.get(sv) {
                        for (observable, observation) in observables {
                            if let Some(ref_observation) = ref_observables.get(observable) {
                                if let Some((_, c_svnn)) = record.get_mut(&(*epoch, *flag)) {
                                    if let Some(c_observables) = c_svnn.get_mut(sv) {
                                        c_observables.insert(
                                            observable.clone(),
                                            ObservationData {
                                                obs: observation.obs - ref_observation.obs,
                                                lli: None,
                                                snr: None,
                                            },
                                        );
                                    } else {
                                        // new observable
                                        let mut inner =
                                            HashMap::<Observable, ObservationData>::new();
                                        let observation = ObservationData {
                                            obs: observation.obs - ref_observation.obs,
                                            lli: None,
                                            snr: None,
                                        };
                                        inner.insert(observable.clone(), observation);
                                        c_svnn.insert(*sv, inner);
                                    }
                                } else {
                                    // new epoch
                                    let mut map = HashMap::<Observable, ObservationData>::new();
                                    let observation = ObservationData {
                                        obs: observation.obs - ref_observation.obs,
                                        lli: None,
                                        snr: None,
                                    };
                                    map.insert(observable.clone(), observation);
                                    let mut inner =
                                        BTreeMap::<SV, HashMap<Observable, ObservationData>>::new();
                                    inner.insert(*sv, map);
                                    if let Some(clk) = clk {
                                        if let Some(refclk) = ref_clk {
                                            record.insert(
                                                (*epoch, *flag),
                                                (Some(clk - refclk), inner),
                                            );
                                        } else {
                                            record.insert((*epoch, *flag), (None, inner));
                                        }
                                    } else {
                                        record.insert((*epoch, *flag), (None, inner));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Rinex::new(self.header.clone(), record::Record::ObsRecord(record))
    }

    /// Returns true if Differential Code Biases (DCBs)
    /// are compensated for, in this file, for this GNSS constellation.
    /// DCBs are biases due to tiny frequency differences,
    /// in both the SV embedded code generator, and receiver PLL.
    /// If this is true, that means all code signals received in from
    /// all SV within that constellation, have intrinsinc DCB compensation.
    /// In very high precision and specific applications, you then do not have
    /// to deal with their compensation yourself.
    pub fn dcb_compensation(&self, constellation: Constellation) -> bool {
        self.header
            .dcb_compensations
            .iter()
            .filter(|dcb| dcb.constellation == constellation)
            .count()
            > 0
    }
    /// Returns true if Antenna Phase Center variations are compensated
    /// for in this file. Useful for high precision application.
    pub fn pcv_compensation(&self, constellation: Constellation) -> bool {
        self.header
            .pcv_compensations
            .iter()
            .filter(|pcv| pcv.constellation == constellation)
            .count()
            > 0
    }
    /// Returns `true` if self is a `merged` RINEX file,   
    /// meaning, this file is the combination of two RINEX files merged together.  
    /// This is determined by the presence of a custom yet somewhat standardized `FILE MERGE` comments
    pub fn is_merged(&self) -> bool {
        let special_comment = String::from("FILE MERGE");
        for comment in self.header.comments.iter() {
            if comment.contains(&special_comment) {
                return true;
            }
        }
        false
    }

    /// Removes all observations where receiver phase lock was lost.   
    /// This is only relevant on OBS RINEX.
    pub fn lock_loss_filter_mut(&mut self) {
        self.lli_and_mask_mut(observation::LliFlags::LOCK_LOSS)
    }

    /// Applies given AND mask in place, to all observations.
    /// This has no effect on non observation records.
    /// This also drops observations that did not come with an LLI flag.  
    /// Only relevant on OBS RINEX.
    pub fn lli_and_mask_mut(&mut self, mask: observation::LliFlags) {
        if !self.is_observation_rinex() {
            return; // nothing to browse
        }
        let record = self.record.as_mut_obs().unwrap();
        for (_e, (_clk, sv)) in record.iter_mut() {
            for (_sv, obs) in sv.iter_mut() {
                obs.retain(|_, data| {
                    if let Some(lli) = data.lli {
                        lli.intersects(mask)
                    } else {
                        false // drops data with no LLI attached
                    }
                })
            }
        }
    }

    /// [`Rinex::lli_and_mask`] immutable implementation.   
    /// Only relevant on OBS RINEX.
    pub fn lli_and_mask(&self, mask: observation::LliFlags) -> Self {
        let mut c = self.clone();
        c.lli_and_mask_mut(mask);
        c
    }
    /// Aligns Phase observations at origin
    pub fn observation_phase_align_origin_mut(&mut self) {
        let mut init_phases: HashMap<SV, HashMap<Observable, f64>> = HashMap::new();
        if let Some(r) = self.record.as_mut_obs() {
            for (_, (_, vehicles)) in r.iter_mut() {
                for (sv, observations) in vehicles.iter_mut() {
                    for (observable, data) in observations.iter_mut() {
                        if observable.is_phase_observable() {
                            if let Some(init_phase) = init_phases.get_mut(sv) {
                                if init_phase.get(observable).is_none() {
                                    init_phase.insert(observable.clone(), data.obs);
                                }
                            } else {
                                let mut map: HashMap<Observable, f64> = HashMap::new();
                                map.insert(observable.clone(), data.obs);
                                init_phases.insert(*sv, map);
                            }
                            data.obs -= init_phases.get(sv).unwrap().get(observable).unwrap();
                        }
                    }
                }
            }
        }
    }
    /// Aligns Phase observations at origin,
    /// immutable implementation
    pub fn observation_phase_align_origin(&self) -> Self {
        let mut s = self.clone();
        s.observation_phase_align_origin_mut();
        s
    }
    /// Converts all Phase Data to Carrier Cycles by multiplying all phase points
    /// by the carrier signal wavelength.
    pub fn observation_phase_carrier_cycles_mut(&mut self) {
        if let Some(r) = self.record.as_mut_obs() {
            for (_, (_, vehicles)) in r.iter_mut() {
                for (sv, observations) in vehicles.iter_mut() {
                    for (observable, data) in observations.iter_mut() {
                        if observable.is_phase_observable() {
                            if let Ok(carrier) = observable.carrier(sv.constellation) {
                                data.obs *= carrier.wavelength();
                            }
                        }
                    }
                }
            }
        }
    }

    /// Converts all Phase Data to Carrier Cycles by multiplying all phase points
    /// by the carrier signal wavelength.
    pub fn observation_phase_carrier_cycles(&self) -> Self {
        let mut s = self.clone();
        s.observation_phase_carrier_cycles_mut();
        s
    }

    /// Writes self into given file.   
    /// Both header + record will strictly follow RINEX standards.   
    /// Record: refer to supported RINEX types.
    /// ```
    /// // Read a RINEX and dump it without any modifications
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///   .unwrap();
    /// assert!(rnx.to_file("test.rnx").is_ok());
    /// ```
    /// Other useful links are:
    ///   * [Self::standard_filename] to generate a standardized filename
    ///   * [Self::guess_production_attributes] helps generate standardized filenames for
    ///     files that do not follow naming conventions
    pub fn to_file(&self, path: &str) -> Result<(), Error> {
        let mut writer = BufferedWriter::new(path)?;
        write!(writer, "{}", self.header)?;
        self.record.to_file(&self.header, &mut writer)?;
        Ok(())
    }
}

/*
 * Sampling related methods
 */
impl Rinex {
    /// Returns first [`Epoch`] encountered in time
    pub fn first_epoch(&self) -> Option<Epoch> {
        self.epoch().next()
    }

    /// Returns last [`Epoch`] encountered in time
    pub fn last_epoch(&self) -> Option<Epoch> {
        self.epoch().last()
    }

    /// Returns Duration of (time spanned by) this RINEX
    pub fn duration(&self) -> Option<Duration> {
        let start = self.first_epoch()?;
        let end = self.last_epoch()?;
        Some(end - start)
    }

    /// Form a [`Timeseries`] iterator spanning [Self::duration]
    /// with [Self::dominant_sample_rate] spacing
    pub fn timeseries(&self) -> Option<TimeSeries> {
        let start = self.first_epoch()?;
        let end = self.last_epoch()?;
        let dt = self.dominant_sample_rate()?;
        Some(TimeSeries::inclusive(start, end, dt))
    }

    /// Returns sample rate used by the data receiver.
    pub fn sample_rate(&self) -> Option<Duration> {
        self.header.sampling_interval
    }

    /// Returns dominant sample rate
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// assert_eq!(
    ///     rnx.dominant_sample_rate(),
    ///     Some(Duration::from_seconds(60.0)));
    /// ```
    pub fn dominant_sample_rate(&self) -> Option<Duration> {
        self.sampling_histogram()
            .max_by(|(_, pop_i), (_, pop_j)| pop_i.cmp(pop_j))
            .map(|dominant| dominant.0)
    }
    /// Histogram analysis on Epoch interval. Although
    /// it is feasible on all types indexed by [Epoch],
    /// this operation only makes truly sense on Observation Data.
    /// ```
    /// use rinex::prelude::*;
    /// use itertools::Itertools;
    /// use std::collections::HashMap;
    /// let rinex = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    ///  assert!(
    ///     rinex.sampling_histogram().sorted().eq(vec![
    ///         (Duration::from_seconds(30.0), 1),
    ///     ]),
    ///     "sampling_histogram failed"
    /// );
    /// ```
    pub fn sampling_histogram(&self) -> Box<dyn Iterator<Item = (Duration, usize)> + '_> {
        // compute dt = |e_k+1 - e_k| : instantaneous epoch delta
        //              then compute an histogram on these intervals
        Box::new(
            self.epoch()
                .zip(self.epoch().skip(1))
                .map(|(ek, ekp1)| ekp1 - ek) // following step computes the histogram
                // and at the same time performs a .unique() like filter
                .fold(vec![], |mut list, dt| {
                    let mut found = false;
                    for (delta, pop) in list.iter_mut() {
                        if *delta == dt {
                            *pop += 1;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        list.push((dt, 1));
                    }
                    list
                })
                .into_iter(),
        )
    }
    /// Returns True if Self has a steady sampling, ie., all epoch interval
    /// are evenly spaced
    pub fn steady_sampling(&self) -> bool {
        self.sampling_histogram().count() == 1
    }
    /// Returns an iterator over unexpected data gaps,
    /// in the form ([`Epoch`], [`Duration`]), where
    /// epoch is the starting datetime, and its related duration.
    /// ```
    /// use std::str::FromStr;
    /// use rinex::prelude::{Rinex, Epoch, Duration};
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    ///
    /// // when tolerance is set to None,
    /// // the reference sample rate is [Self::dominant_sample_rate].
    /// let mut tolerance : Option<Duration> = None;
    /// let gaps : Vec<_> = rinex.data_gaps(tolerance).collect();
    /// assert!(
    ///     rinex.data_gaps(None).eq(
    ///         vec![
    ///             (Epoch::from_str("2015-01-01T00:09:00 UTC").unwrap(), Duration::from_seconds(8.0 * 3600.0 + 51.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T09:04:00 UTC").unwrap(), Duration::from_seconds(10.0 * 3600.0 + 21.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T19:54:00 UTC").unwrap(), Duration::from_seconds(3.0 * 3600.0 + 1.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T23:02:00 UTC").unwrap(), Duration::from_seconds(7.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T23:21:00 UTC").unwrap(), Duration::from_seconds(31.0 * 60.0)),
    ///         ]),
    ///     "data_gaps(tol=None) failed"
    /// );
    ///
    /// // with a tolerance, we tolerate the given gap duration
    /// tolerance = Some(Duration::from_seconds(3600.0));
    /// let gaps : Vec<_> = rinex.data_gaps(tolerance).collect();
    /// assert!(
    ///     rinex.data_gaps(Some(Duration::from_seconds(3.0 * 3600.0))).eq(
    ///         vec![
    ///             (Epoch::from_str("2015-01-01T00:09:00 UTC").unwrap(), Duration::from_seconds(8.0 * 3600.0 + 51.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T09:04:00 UTC").unwrap(), Duration::from_seconds(10.0 * 3600.0 + 21.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T19:54:00 UTC").unwrap(), Duration::from_seconds(3.0 * 3600.0 + 1.0 * 60.0)),
    ///         ]),
    ///     "data_gaps(tol=3h) failed"
    /// );
    /// ```
    pub fn data_gaps(
        &self,
        tolerance: Option<Duration>,
    ) -> Box<dyn Iterator<Item = (Epoch, Duration)> + '_> {
        let sample_rate: Duration = match tolerance {
            Some(dt) => dt, // user defined
            None => {
                match self.dominant_sample_rate() {
                    Some(dt) => dt,
                    None => {
                        match self.sample_rate() {
                            Some(dt) => dt,
                            None => {
                                // not enough information
                                // this is probably not an Epoch iterated RINEX
                                return Box::new(Vec::<(Epoch, Duration)>::new().into_iter());
                            },
                        }
                    },
                }
            },
        };
        Box::new(
            self.epoch()
                .zip(self.epoch().skip(1))
                .filter_map(move |(ek, ekp1)| {
                    let dt = ekp1 - ek; // gap
                    if dt > sample_rate {
                        // too large
                        Some((ek, dt)) // retain starting datetime and gap duration
                    } else {
                        None
                    }
                }),
        )
    }
}

/*
 * Methods that return an Iterator exclusively.
 * These methods are used to browse data easily and efficiently.
 */
impl Rinex {
    pub fn epoch(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        if let Some(r) = self.record.as_obs() {
            Box::new(r.iter().map(|((k, _), _)| *k))
        } else if let Some(r) = self.record.as_doris() {
            Box::new(r.iter().map(|((k, _), _)| *k))
        } else if let Some(r) = self.record.as_nav() {
            Box::new(r.iter().map(|(k, _)| *k))
        } else if let Some(r) = self.record.as_meteo() {
            Box::new(r.iter().map(|(k, _)| *k))
        } else if let Some(r) = self.record.as_clock() {
            Box::new(r.iter().map(|(k, _)| *k))
        } else if let Some(r) = self.record.as_ionex() {
            Box::new(r.iter().map(|((k, _), _)| *k))
        } else {
            panic!(
                "cannot get an epoch iterator for \"{:?}\" RINEX",
                self.header.rinex_type
            );
        }
    }

    /// Returns a unique [`SV`] iterator, to navigate
    /// all Satellite Vehicles encountered and identified.
    /// This will panic if invoked on ATX, Meteo or IONEX records.
    /// In case of Clock RINEX, the returns the list of vehicles
    /// used as reference.
    /// ```
    /// extern crate gnss_rs as gnss;
    /// use rinex::prelude::*;
    /// use gnss_rs::prelude::*;
    /// use gnss_rs::sv; // sv!
    /// use std::str::FromStr; // sv!
    ///
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/aopr0010.17o")
    ///     .unwrap();
    /// let mut vehicles : Vec<_> = rnx.sv().collect(); // to run comparison
    /// vehicles.sort(); // to run comparison
    ///
    /// assert_eq!(vehicles, vec![
    ///     sv!("G01"), sv!("G03"), sv!("G06"),
    ///     sv!("G07"), sv!("G08"), sv!("G09"),
    ///     sv!("G11"), sv!("G14"), sv!("G16"),
    ///     sv!("G17"), sv!("G19"), sv!("G22"),
    ///     sv!("G23"), sv!("G26"), sv!("G27"),
    ///     sv!("G28"), sv!("G30"), sv!("G31"),
    ///     sv!("G32")]);
    /// ```
    pub fn sv(&self) -> Box<dyn Iterator<Item = SV> + '_> {
        if let Some(record) = self.record.as_obs() {
            Box::new(
                // grab all vehicles identified through all Epochs
                // and fold them into a unique list
                record
                    .iter()
                    .map(|((_, _), (_clk, entries))| {
                        let sv: Vec<SV> = entries.keys().cloned().collect();
                        sv
                    })
                    .fold(vec![], |mut list, new_items| {
                        for new in new_items {
                            if !list.contains(&new) {
                                // create a unique list
                                list.push(new);
                            }
                        }
                        list
                    })
                    .into_iter(),
            )
        } else if let Some(record) = self.record.as_nav() {
            Box::new(
                // grab all vehicles through all epochs,
                // fold them into a unique list
                record
                    .iter()
                    .flat_map(|(_, frames)| {
                        frames
                            .iter()
                            .filter_map(|fr| {
                                if let Some((_, sv, _)) = fr.as_eph() {
                                    Some(sv)
                                } else if let Some((_, sv, _)) = fr.as_eop() {
                                    Some(sv)
                                } else if let Some((_, sv, _)) = fr.as_ion() {
                                    Some(sv)
                                } else if let Some((_, sv, _)) = fr.as_sto() {
                                    Some(sv)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                            .into_iter()
                    })
                    .unique(),
            )
        } else if let Some(record) = self.record.as_clock() {
            Box::new(
                // grab all embedded sv clocks
                record
                    .iter()
                    .flat_map(|(_, keys)| {
                        keys.iter()
                            .filter_map(|(key, _)| key.clock_type.as_sv())
                            .collect::<Vec<_>>()
                            .into_iter()
                    })
                    .unique(),
            )
        } else {
            panic!(
                ".sv() is not feasible on \"{:?}\" RINEX",
                self.header.rinex_type
            );
        }
    }

    /// List all [`SV`] per epoch of appearance.
    /// ```
    /// use rinex::prelude::*;
    /// use std::str::FromStr;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/aopr0010.17o")
    ///     .unwrap();
    ///
    /// let mut data = rnx.sv_epoch();
    ///
    /// if let Some((epoch, vehicles)) = data.nth(0) {
    ///     assert_eq!(epoch, Epoch::from_str("2017-01-01T00:00:00 GPST").unwrap());
    ///     let expected = vec![
    ///         SV::new(Constellation::GPS, 03),
    ///         SV::new(Constellation::GPS, 08),
    ///         SV::new(Constellation::GPS, 14),
    ///         SV::new(Constellation::GPS, 16),
    ///         SV::new(Constellation::GPS, 22),
    ///         SV::new(Constellation::GPS, 23),
    ///         SV::new(Constellation::GPS, 26),
    ///         SV::new(Constellation::GPS, 27),
    ///         SV::new(Constellation::GPS, 31),
    ///         SV::new(Constellation::GPS, 32),
    ///     ];
    ///     assert_eq!(*vehicles, expected);
    /// }
    /// ```
    pub fn sv_epoch(&self) -> Box<dyn Iterator<Item = (Epoch, Vec<SV>)> + '_> {
        if let Some(record) = self.record.as_obs() {
            Box::new(
                // grab all vehicles identified through all Epochs
                // and fold them into individual lists
                record.iter().map(|((epoch, _), (_clk, entries))| {
                    (*epoch, entries.keys().unique().cloned().collect())
                }),
            )
        } else if let Some(record) = self.record.as_nav() {
            Box::new(
                // grab all vehicles through all epochs,
                // fold them into individual lists
                record.iter().map(|(epoch, frames)| {
                    (
                        *epoch,
                        frames
                            .iter()
                            .filter_map(|fr| {
                                if let Some((_, sv, _)) = fr.as_eph() {
                                    Some(sv)
                                } else if let Some((_, sv, _)) = fr.as_eop() {
                                    Some(sv)
                                } else if let Some((_, sv, _)) = fr.as_ion() {
                                    Some(sv)
                                } else if let Some((_, sv, _)) = fr.as_sto() {
                                    Some(sv)
                                } else {
                                    None
                                }
                            })
                            .fold(vec![], |mut list, sv| {
                                if !list.contains(&sv) {
                                    list.push(sv);
                                }
                                list
                            }),
                    )
                }),
            )
        } else {
            panic!(
                ".sv_epoch() is not feasible on \"{:?}\" RINEX",
                self.header.rinex_type
            );
        }
    }
    /// Returns a (unique) Iterator over all identified [`Constellation`]s.
    /// ```
    /// use rinex::prelude::*;
    /// use itertools::Itertools; // .sorted()
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx")
    ///     .unwrap();
    ///
    /// assert!(
    ///     rnx.constellation().sorted().eq(
    ///         vec![
    ///             Constellation::GPS,
    ///             Constellation::Glonass,
    ///             Constellation::BeiDou,
    ///             Constellation::Galileo,
    ///         ]
    ///     ),
    ///     "parsed wrong GNSS context",
    /// );
    /// ```
    pub fn constellation(&self) -> Box<dyn Iterator<Item = Constellation> + '_> {
        // from .sv() (unique) iterator:
        //  create a unique list of Constellations
        Box::new(self.sv().map(|sv| sv.constellation).unique())
    }
    /// Returns an Iterator over Unique Constellations, per Epoch
    pub fn constellation_epoch(
        &self,
    ) -> Box<dyn Iterator<Item = (Epoch, Vec<Constellation>)> + '_> {
        Box::new(self.sv_epoch().map(|(epoch, svnn)| {
            (
                epoch,
                svnn.iter().map(|sv| sv.constellation).unique().collect(),
            )
        }))
    }
    /// Returns a (unique) Iterator over all identified [`Observable`]s.
    /// Applies to Observation RINEX:
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/CRNX/V1/AJAC3550.21D")
    ///     .unwrap();
    /// for observable in rinex.observable() {
    ///     if observable.is_phase_observable() {
    ///         // do something
    ///     }
    /// }
    /// ```
    /// Also applies to Meteo RINEX:
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for observable in rinex.observable() {
    ///     if *observable == Observable::Temperature {
    ///         // do something
    ///     }
    /// }
    /// ```
    /// Also applies to DORIS RINEX:
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/DOR/V3/cs2rx18164.gz")
    ///     .unwrap();
    /// for observable in rinex.observable() {
    ///     if observable.is_pseudorange_observable() {
    ///         // do something
    ///     }
    /// }
    /// ```
    pub fn observable(&self) -> Box<dyn Iterator<Item = &Observable> + '_> {
        if self.record.as_obs().is_some() {
            Box::new(
                self.observation()
                    .flat_map(|(_, (_, svnn))| {
                        svnn.iter()
                            .flat_map(|(_, observables)| observables.iter().map(|(k, _)| k))
                    })
                    .unique(),
            )
        } else if self.record.as_meteo().is_some() {
            Box::new(
                self.meteo()
                    .flat_map(|(_, observables)| observables.iter().map(|(k, _)| k))
                    .unique(),
            )
        } else if self.record.as_doris().is_some() {
            Box::new(
                self.doris()
                    .flat_map(|(_, stations)| {
                        stations
                            .iter()
                            .flat_map(|(_, observables)| observables.iter().map(|(k, _)| k))
                    })
                    .unique(),
            )
        } else {
            Box::new([].iter())
        }
    }
    /// Meteo RINEX record browsing method. Extracts data for this specific format.
    /// Data is sorted by [`Epoch`] then by [`Observable`].
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///    .unwrap();
    /// for (epoch, observables) in rnx.meteo() {
    ///     println!(" *** Epoch:  {} ****", epoch);
    ///     for (observable, data) in observables {
    ///         println!("{} : {}", observable, data);
    ///     }
    /// }
    /// ```
    pub fn meteo(&self) -> Box<dyn Iterator<Item = (&Epoch, &HashMap<Observable, f64>)> + '_> {
        Box::new(
            self.record
                .as_meteo()
                .into_iter()
                .flat_map(|record| record.iter()),
        )
    }
    /// Returns Observation record iterator. Unlike other records,
    /// an [`EpochFlag`] is attached to each individual [`Epoch`]
    /// to either validated or invalidate it.
    /// Clock receiver offset (in seconds), if present, are defined for each individual
    /// [`Epoch`].
    /// Phase data is exposed as raw / unscaled data: therefore incorrect
    /// values in case of High Precision RINEX. Prefer the dedicated
    /// [Self::carrier_phase] iterator. In any case, you should always
    /// prefer the iteration method of the type of data you're interested in.
    /// ```
    /// use rinex::prelude::*;
    /// use gnss_rs::prelude::SV;
    /// // macros
    /// use gnss_rs::sv;
    /// use rinex::observable;
    /// use std::str::FromStr; // observable!, sv!
    ///
    /// let rnx = Rinex::from_file("../test_resources/CRNX/V3/KUNZ00CZE.crx")
    ///    .unwrap();
    ///
    /// for ((epoch, flag), (clock_offset, vehicles)) in rnx.observation() {
    ///     assert!(flag.is_ok()); // no invalid epochs in this file
    ///     assert!(clock_offset.is_none()); // we don't have an example for this, at the moment
    ///     for (sv, observations) in vehicles {
    ///         if *sv == sv!("E01") {
    ///             for (observable, observation) in observations {
    ///                 if *observable == observable!("L1C") {
    ///                     if let Some(lli) = observation.lli {
    ///                         // A flag might be attached to each observation.
    ///                         // Implemented as `bitflag`, it supports bit masking operations
    ///                     }
    ///                     if let Some(snri) = observation.snr {
    ///                         // SNR indicator might exist too
    ///                     }
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn observation(
        &self,
    ) -> Box<
        dyn Iterator<
                Item = (
                    &(Epoch, EpochFlag),
                    &(
                        Option<f64>,
                        BTreeMap<SV, HashMap<Observable, ObservationData>>,
                    ),
                ),
            > + '_,
    > {
        Box::new(
            self.record
                .as_obs()
                .into_iter()
                .flat_map(|record| record.iter()),
        )
    }
    /// Returns Navigation Data interator (any type of message).
    /// NAV records may contain several different types of frames.
    /// You should prefer more precise methods, like [ephemeris] or
    /// [ionosphere_models] but those require the "nav" feature.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::navigation::NavMsgType;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V2/amel0010.21g")
    ///     .unwrap();
    /// for (epoch, nav_frames) in rinex.navigation() {
    ///     for frame in nav_frames {
    ///         // this record only contains ephemeris frames
    ///         assert!(frame.as_eph().is_some());
    ///         assert!(frame.as_ion().is_none());
    ///         assert!(frame.as_eop().is_none());
    ///         assert!(frame.as_sto().is_none());
    ///         if let Some((msg, sv, data)) = frame.as_eph() {
    ///             // this record only contains legacy frames
    ///             assert_eq!(msg, NavMsgType::LNAV);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn navigation(&self) -> Box<dyn Iterator<Item = (&Epoch, &Vec<NavFrame>)> + '_> {
        Box::new(
            self.record
                .as_nav()
                .into_iter()
                .flat_map(|record| record.iter()),
        )
    }
    /// DORIS special RINEX iterator
    pub fn doris(
        &self,
    ) -> Box<
        dyn Iterator<
                Item = (
                    &(Epoch, EpochFlag),
                    &BTreeMap<Station, HashMap<Observable, DorisObservationData>>,
                ),
            > + '_,
    > {
        Box::new(
            self.record
                .as_doris()
                .into_iter()
                .flat_map(|record| record.iter()),
        )
    }
    /// ANTEX antennas specifications browsing
    pub fn antennas(
        &self,
    ) -> Box<dyn Iterator<Item = &(Antenna, HashMap<Carrier, FrequencyDependentData>)> + '_> {
        Box::new(
            self.record
                .as_antex()
                .into_iter()
                .flat_map(|record| record.iter()),
        )
    }
}

// #[cfg(feature = "obs")]
// use std::str::FromStr;

#[cfg(feature = "obs")]
use crate::observation::{record::code_multipath, LliFlags, SNR};

/*
 * OBS RINEX specific methods: only available on crate feature.
 * Either specific Iterators, or meaningful data we can extract.
 */
#[cfg(feature = "obs")]
#[cfg_attr(docrs, doc(cfg(feature = "obs")))]
impl Rinex {
    /// Returns a Unique Iterator over identified [`Carrier`]s
    pub fn carrier(&self) -> Box<dyn Iterator<Item = Carrier> + '_> {
        Box::new(
            self.observation()
                .flat_map(|(_, (_, sv))| {
                    sv.iter().flat_map(|(sv, observations)| {
                        observations.keys().filter_map(|observable| {
                            Some(observable.carrier(sv.constellation).ok()?)
                        })
                    })
                })
                .unique(),
        )
    }
    /// Returns a Unique Iterator over signal Codes, like "1C" or "1P"
    /// for precision code.
    pub fn code(&self) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(
            self.observation()
                .flat_map(|(_, (_, sv))| {
                    sv.iter().flat_map(|(_, observations)| {
                        observations
                            .keys()
                            .filter_map(|observable| observable.code())
                    })
                })
                .unique(),
        )
    }
    /// Returns Unique Iterator over all feasible Pseudo range and Phase range combination,
    /// expressed as (lhs: Observable, rhs: Observable).
    /// Regardless which one is to consider as reference signal.
    /// Use [pseudo_range_combinations()] or [phase_range_combinations()]
    /// to reduce to specific physical observations.
    pub fn signal_combinations(&self) -> Box<dyn Iterator<Item = (&Observable, &Observable)> + '_> {
        Box::new(
            self.pseudo_range_combinations()
                .zip(self.phase_range_combinations()),
        )
    }
    /// See [signal_combinations()]
    pub fn pseudo_range_combinations(
        &self,
    ) -> Box<dyn Iterator<Item = (&Observable, &Observable)> + '_> {
        self.signal_combinations().filter_map(|(lhs, rhs)| {
            if lhs.is_pseudorange_observable() {
                Some((lhs, rhs))
            } else {
                None
            }
        })
    }
    /// See [signal_combinations()]
    pub fn phase_range_combinations(
        &self,
    ) -> Box<dyn Iterator<Item = (&Observable, &Observable)> + '_> {
        self.signal_combinations().filter_map(|(lhs, rhs)| {
            if lhs.is_phase_observable() {
                Some((lhs, rhs))
            } else {
                None
            }
        })
    }
    /// Returns ([`Epoch`] [`EpochFlag`]) iterator, where each {`EpochFlag`]
    /// validates or invalidates related [`Epoch`]
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// for (epoch, flag) in rnx.epoch_flag() {
    ///     assert!(flag.is_ok()); // no invalid epoch
    /// }
    /// ```
    pub fn epoch_flag(&self) -> Box<dyn Iterator<Item = (Epoch, EpochFlag)> + '_> {
        Box::new(self.observation().map(|(e, _)| *e))
    }
    /// Returns an Iterator over all abnormal [`Epoch`]s
    /// and reports given event nature.  
    /// Refer to [`epoch::EpochFlag`] for all possible events.  
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// ```
    pub fn epoch_anomalies(&self) -> Box<dyn Iterator<Item = (Epoch, EpochFlag)> + '_> {
        Box::new(self.epoch_flag().filter_map(
            |(e, f)| {
                if !f.is_ok() {
                    Some((e, f))
                } else {
                    None
                }
            },
        ))
    }
    /// Returns an iterator over all [`Epoch`]s that have
    /// an [`EpochFlag::Ok`] flag attached to them
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// ```
    pub fn epoch_ok(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        Box::new(
            self.epoch_flag()
                .filter_map(|(e, f)| if f.is_ok() { Some(e) } else { None }),
        )
    }
    /// Returns an iterator over all [`Epoch`]s where
    /// a Cycle Slip is declared by the receiver
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// ```
    pub fn epoch_cs(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        Box::new(self.epoch_flag().filter_map(|(e, f)| {
            if f == EpochFlag::CycleSlip {
                Some(e)
            } else {
                None
            }
        }))
    }
    /// Returns an iterator over receiver clock offsets, expressed in seconds.
    /// Such information is kind of rare (modern / dual frequency receivers?)
    /// and we don't have a compelling example yet.
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// for ((epoch, flag), clk) in rnx.recvr_clock() {
    ///     // epoch: [hifitime::Epoch]
    ///     // clk: receiver clock offset [s]
    /// }
    /// ```
    pub fn recvr_clock(&self) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), f64)> + '_> {
        Box::new(
            self.observation()
                .filter_map(|(e, (clk, _))| clk.as_ref().map(|clk| (*e, *clk))),
        )
    }
    /// Returns an iterator over phase data, expressed in (whole) carrier cycles.
    /// If Self is a High Precision RINEX (scaled RINEX), data is correctly scaled.
    /// High precision RINEX allows up to 100 pico carrier cycle precision.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observable;
    /// use std::str::FromStr;
    ///
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    /// // example: design a L1 signal iterator
    /// let phase_l1c = rnx.carrier_phase()
    ///     .filter_map(|(e, sv, obs, value)| {
    ///         if *obs == observable!("L1C") {
    ///             Some((e, sv, value))
    ///         } else {
    ///             None
    ///         }
    ///     });
    /// ```
    pub fn carrier_phase(
        &self,
    ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, f64)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations.iter().filter_map(|(observable, obsdata)| {
                    if observable.is_phase_observable() {
                        if let Some(header) = &self.header.obs {
                            // apply a scaling (if any), otherwise preserve data precision
                            if let Some(scaling) =
                                header.scaling(sv.constellation, observable.clone())
                            {
                                Some((*e, *sv, observable, obsdata.obs / *scaling as f64))
                            } else {
                                Some((*e, *sv, observable, obsdata.obs))
                            }
                        } else {
                            Some((*e, *sv, observable, obsdata.obs))
                        }
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Returns an iterator over pseudo range observations.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observable;
    /// use std::str::FromStr;
    ///
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    /// // example: design a C1 pseudo range iterator
    /// let c1 = rnx.pseudo_range()
    ///     .filter_map(|(e, sv, obs, value)| {
    ///         if *obs == observable!("C1") {
    ///             Some((e, sv, value))
    ///         } else {
    ///             None
    ///         }
    ///     });
    /// ```
    pub fn pseudo_range(
        &self,
    ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, f64)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations.iter().filter_map(|(obs, obsdata)| {
                    if obs.is_pseudorange_observable() {
                        Some((*e, *sv, obs, obsdata.obs))
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Returns an Iterator over pseudo range observations in valid
    /// Epochs, with valid LLI flags
    pub fn pseudo_range_ok(&self) -> Box<dyn Iterator<Item = (Epoch, SV, &Observable, f64)> + '_> {
        Box::new(self.observation().flat_map(|((e, flag), (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations.iter().filter_map(|(obs, obsdata)| {
                    if obs.is_pseudorange_observable() {
                        if flag.is_ok() {
                            Some((*e, *sv, obs, obsdata.obs))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
        }))
    }

    /// Returns an Iterator over fractional pseudo range observations
    pub fn pseudo_range_fract(
        &self,
    ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, f64)> + '_> {
        Box::new(self.pseudo_range().filter_map(|(e, sv, observable, pr)| {
            if let Some(t) = observable.code_length(sv.constellation) {
                let c = 299792458_f64; // speed of light
                Some((e, sv, observable, pr / c / t))
            } else {
                None
            }
        }))
    }
    /// Returns an iterator over doppler shifts. A positive doppler
    /// means SV is moving towards receiver.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observable;
    /// use std::str::FromStr;
    ///
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    /// // example: design a L1 signal doppler iterator
    /// let doppler_l1 = rnx.doppler()
    ///     .filter_map(|(e, sv, obs, value)| {
    ///         if *obs == observable!("D1") {
    ///             Some((e, sv, value))
    ///         } else {
    ///             None
    ///         }
    ///     });
    /// ```
    pub fn doppler(
        &self,
    ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, f64)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations.iter().filter_map(|(obs, obsdata)| {
                    if obs.is_doppler_observable() {
                        Some((*e, *sv, obs, obsdata.obs))
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Returns an iterator over signal strength observations.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observable;
    /// use std::str::FromStr;
    ///
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    /// // example: design a S1: L1 strength iterator
    /// let ssi_l1 = rnx.ssi()
    ///     .filter_map(|(e, sv, obs, value)| {
    ///         if *obs == observable!("S1") {
    ///             Some((e, sv, value))
    ///         } else {
    ///             None
    ///         }
    ///     });
    /// ```
    pub fn ssi(&self) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, f64)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations.iter().filter_map(|(obs, obsdata)| {
                    if obs.is_ssi_observable() {
                        Some((*e, *sv, obs, obsdata.obs))
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Returns an Iterator over signal SNR indications.
    /// All observation that did not come with such indication are filtered out.
    /// ```
    /// use rinex::*;
    /// let rinex =
    ///     Rinex::from_file("../test_resources/OBS/V3/ALAC00ESP_R_20220090000_01D_30S_MO.rnx")
    ///         .unwrap();
    /// for ((e, flag), sv, observable, snr) in rinex.snr() {
    ///     // See RINEX specs or [SNR] documentation
    ///     if snr.weak() {
    ///     } else if snr.strong() {
    ///     } else if snr.excellent() {
    ///     }
    ///     // you can directly compare to dBHz
    ///     if snr < 29.0.into() {
    ///         // considered weak signal
    ///     } else if snr >= 30.0.into() {
    ///         // considered strong signal
    ///     }
    /// }
    /// ```
    pub fn snr(&self) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, SNR)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations
                    .iter()
                    .filter_map(|(obs, obsdata)| obsdata.snr.map(|snr| (*e, *sv, obs, snr)))
            })
        }))
    }
    /// Returns an Iterator over LLI flags that might be associated to an Observation.
    /// ```
    /// use rinex::*;
    /// use rinex::observation::LliFlags;
    /// let rinex =
    ///     Rinex::from_file("../test_resources/OBS/V3/ALAC00ESP_R_20220090000_01D_30S_MO.rnx")
    ///         .unwrap();
    /// let custom_mask
    ///     = LliFlags::OK_OR_UNKNOWN | LliFlags::UNDER_ANTI_SPOOFING;
    /// for ((e, flag), sv, observable, lli) in rinex.lli() {
    ///     // See RINEX specs or [LliFlags] documentation
    ///     if lli.intersects(custom_mask) {
    ///         // sane observation but under AS
    ///     }
    /// }
    /// ```
    pub fn lli(
        &self,
    ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, LliFlags)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations
                    .iter()
                    .filter_map(|(obs, obsdata)| obsdata.lli.map(|lli| (*e, *sv, obs, lli)))
            })
        }))
    }
    /// Returns an Iterator over "complete" Epochs.
    /// "Complete" Epochs are Epochs were both Phase and Pseudo Range
    /// observations are present on two carriers, sane sampling conditions are met
    /// and an optional minimal SNR criteria is met (disregarded if None).
    pub fn complete_epoch(
        &self,
        min_snr: Option<SNR>,
    ) -> Box<dyn Iterator<Item = (Epoch, Vec<(SV, Carrier)>)> + '_> {
        Box::new(
            self.observation()
                .filter_map(move |((e, flag), (_, vehicles))| {
                    if flag.is_ok() {
                        let mut list: Vec<(SV, Carrier)> = Vec::new();
                        for (sv, observables) in vehicles {
                            let mut l1_pr_ph = (false, false);
                            let mut lx_pr_ph: HashMap<Carrier, (bool, bool)> = HashMap::new();
                            for (observable, observation) in observables {
                                if !observable.is_phase_observable()
                                    && !observable.is_pseudorange_observable()
                                {
                                    continue; // not interesting here
                                }
                                let carrier =
                                    Carrier::from_observable(sv.constellation, observable);
                                if carrier.is_err() {
                                    // fail to identify this signal
                                    continue;
                                }
                                if let Some(min_snr) = min_snr {
                                    if let Some(snr) = observation.snr {
                                        if snr < min_snr {
                                            continue;
                                        }
                                    } else {
                                        continue; // can't compare to criteria
                                    }
                                }
                                let carrier = carrier.unwrap();
                                if carrier == Carrier::L1 {
                                    l1_pr_ph.0 |= observable.is_pseudorange_observable();
                                    l1_pr_ph.1 |= observable.is_phase_observable();
                                } else if let Some((lx_pr, lx_ph)) = lx_pr_ph.get_mut(&carrier) {
                                    *lx_pr |= observable.is_pseudorange_observable();
                                    *lx_ph |= observable.is_phase_observable();
                                } else if observable.is_pseudorange_observable() {
                                    lx_pr_ph.insert(carrier, (true, false));
                                } else if observable.is_phase_observable() {
                                    lx_pr_ph.insert(carrier, (false, true));
                                }
                            }
                            if l1_pr_ph == (true, true) {
                                for (carrier, (pr, ph)) in lx_pr_ph {
                                    if pr && ph {
                                        list.push((*sv, carrier));
                                    }
                                }
                            }
                        }
                        Some((*e, list))
                    } else {
                        None
                    }
                })
                .filter(|(_sv, list)| !list.is_empty()),
        )
    }
    /// Returns Code Multipath bias estimates, for sampled code combination and per SV.
    /// Refer to [Bibliography::ESABookVol1] and [Bibliography::MpTaoglas].
    pub fn code_multipath(
        &self,
    ) -> HashMap<Observable, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
        if let Some(r) = self.record.as_obs() {
            code_multipath(r)
        } else {
            HashMap::new()
        }
    }
}

#[cfg(feature = "nav")]
use crate::navigation::{
    BdModel, EopMessage, Ephemeris, IonMessage, KbModel, NavMsgType, NgModel, StoMessage,
};

//#[cfg(feature = "nav")]
//use hifitime::Unit;
//.sv_speed()

#[cfg(feature = "nav")]
use map_3d::ecef2geodetic;

/*
 * NAV RINEX specific methods: only available on crate feature.
 * Either specific Iterators, or meaningful data we can extract.
 */
#[cfg(feature = "nav")]
#[cfg_attr(docrs, doc(cfg(feature = "nav")))]
impl Rinex {
    /// Returns a Unique Iterator over [`NavMsgType`]s that were identified
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::navigation::NavMsgType;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V2/amel0010.21g")
    ///     .unwrap();
    /// assert!(
    ///     rinex.nav_msg_type().eq(
    ///         vec![NavMsgType::LNAV],
    ///     ),
    ///     "this file only contains legacy frames"
    /// );
    /// ```
    pub fn nav_msg_type(&self) -> Box<dyn Iterator<Item = NavMsgType> + '_> {
        Box::new(
            self.navigation()
                .map(|(_, frames)| {
                    frames
                        .iter()
                        .filter_map(|fr| {
                            if let Some((msg, _, _)) = fr.as_eph() {
                                Some(msg)
                            } else if let Some((msg, _, _)) = fr.as_ion() {
                                Some(msg)
                            } else if let Some((msg, _, _)) = fr.as_eop() {
                                Some(msg)
                            } else if let Some((msg, _, _)) = fr.as_sto() {
                                Some(msg)
                            } else {
                                None
                            }
                        })
                        .fold(vec![], |mut list, msg| {
                            list.push(msg);
                            list
                        })
                        .into_iter()
                })
                .fold(vec![], |mut list, items| {
                    for item in items {
                        if !list.contains(&item) {
                            list.push(item); // create a unique list
                        }
                    }
                    list
                })
                .into_iter(),
        )
    }
    /// Returns Ephemeris frames interator.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::navigation::NavMsgType;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V2/amel0010.21g")
    ///     .unwrap();
    /// for (epoch, (msg, sv, data)) in rinex.ephemeris() {
    ///     // this record only contains Legacy NAV frames
    ///     assert_eq!(msg, NavMsgType::LNAV);
    /// }
    /// ```
    pub fn ephemeris(
        &self,
    ) -> Box<dyn Iterator<Item = (&Epoch, (NavMsgType, SV, &Ephemeris))> + '_> {
        Box::new(self.navigation().flat_map(|(e, frames)| {
            frames.iter().filter_map(move |fr| {
                if let Some((msg, sv, eph)) = fr.as_eph() {
                    Some((e, (msg, sv, eph)))
                } else {
                    None
                }
            })
        }))
    }
    /// Ephemeris selection method. Use this method to select Ephemeris
    /// to be used to navigate using `sv` at instant `t`.
    /// Returns (toe and ephemeris frame).
    /// Note that TOE does not exist for SBAS vehicles, therefore should be discarded.
    pub fn sv_ephemeris(&self, sv: SV, t: Epoch) -> Option<(Epoch, &Ephemeris)> {
        /*
         *  TODO
         *   <o ideally some more advanced fields like
         *      health, iode should also be taken into account
         */
        self.ephemeris()
            .filter_map(|(_toc, (msg, svnn, eph))| {
                if svnn == sv {
                    let ts = svnn.timescale()?;
                    let toe: Option<Epoch> = match msg {
                        NavMsgType::CNAV => {
                            /* in CNAV : specs says toc is toe actually */
                            // TODO Some(toc.in_time_scale(ts))
                            None
                        },
                        _ => {
                            if sv.constellation.is_sbas() {
                                // toe does not exist
                                Some(t)
                            } else {
                                // determine toe
                                eph.toe_gpst(ts)
                            }
                        },
                    };
                    let toe = toe?;
                    let max_dtoe = Ephemeris::max_dtoe(svnn.constellation)?;
                    if (t - toe) < max_dtoe {
                        Some((toe, eph))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .min_by_key(|(toe_i, _)| (t - *toe_i))
    }
    /// Returns an Iterator over SV (embedded) clock offset (s), drift (s.s⁻¹) and
    /// drift rate (s.s⁻²)
    /// ```
    /// use rinex::prelude::*;
    /// let mut rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx")
    ///     .unwrap();
    /// for (epoch, sv, (offset, drift, drift_rate)) in rinex.sv_clock() {
    ///     // sv: satellite vehicle
    ///     // offset [s]
    ///     // clock drift [s.s⁻¹]
    ///     // clock drift rate [s.s⁻²]
    /// }
    /// ```
    pub fn sv_clock(&self) -> Box<dyn Iterator<Item = (Epoch, SV, (f64, f64, f64))> + '_> {
        Box::new(
            self.ephemeris()
                .map(|(e, (_, sv, data))| (*e, sv, data.sv_clock())),
        )
    }
    /// Returns an Iterator over SV position vectors,
    /// expressed in km ECEF for all Epochs.
    /// ```
    /// use rinex::prelude::*;
    ///
    /// let mut rinex =
    ///     Rinex::from_file("../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///         .unwrap();
    ///
    /// for (epoch, sv, (x, y, z)) in rinex.sv_position() {
    ///     // sv: satellite vehicle
    ///     // x: x(t) [km ECEF]
    ///     // y: y(t) [km ECEF]
    ///     // z: z(t) [km ECEF]
    /// }
    /// ```
    pub fn sv_position(&self) -> Box<dyn Iterator<Item = (Epoch, SV, (f64, f64, f64))> + '_> {
        Box::new(self.ephemeris().filter_map(|(e, (_, sv, ephemeris))| {
            if let Some((x, y, z)) = ephemeris.sv_position(sv, *e) {
                Some((*e, sv, (x, y, z)))
            } else {
                // non feasible calculations.
                // most likely due to missing Keplerian parameters,
                // at this Epoch
                None
            }
        }))
    }
    /// Interpolates SV position, expressed in meters ECEF at desired Epoch `t`.
    /// An interpolation order between 4 and 8 is recommended, depending on the
    /// precision you are targetting. Higher orders do not make sense considering the
    /// noise on broadcasted (real time) positions.
    /// In ideal scenarios, Broadcast Ephemeris are complete and evenly spaced in time:
    ///   - the first Epoch we an interpolate is ](N +1)/2 * τ; ...]
    ///   - the last Epoch we an interpolate is  [..;  T - (N +1)/2 * τ]
    /// where N is the interpolation order, τ the broadcast interval and T
    /// the last broadcast message received.
    /// This method is designed to minimize interpolation errors at the expense
    /// of interpolatable Epochs. See [Bibliography::Japhet2021].
    pub fn sv_position_interpolate(
        &self,
        sv: SV,
        t: Epoch,
        order: usize,
    ) -> Option<(f64, f64, f64)> {
        let odd_order = order % 2 > 0;
        let dt = match self.sample_rate() {
            Some(dt) => dt,
            None => match self.dominant_sample_rate() {
                Some(dt) => dt,
                None => {
                    /*
                     * Can't determine anything: not enough information
                     */
                    return None;
                },
            },
        };

        let sv_position: Vec<_> = self
            .sv_position()
            .filter_map(|(e, svnn, (x, y, z))| {
                if sv == svnn {
                    Some((e, (x, y, z)))
                } else {
                    None
                }
            })
            .collect();
        /*
         * Determine cloesest Epoch in time
         */
        let center = match sv_position.iter().find(|(e, _)| (*e - t).abs() < dt) {
            Some(center) => center,
            None => {
                /*
                 * Failed to determine central Epoch for this SV
                 * empty data set: should not happen
                 */
                return None;
            },
        };
        // println!("CENTRAL EPOCH: {:?}", center); // DEBUG
        let center_pos = match sv_position.iter().position(|(e, _)| *e == center.0) {
            Some(center) => center,
            None => {
                /* will never happen at this point */
                return None;
            },
        };

        let (min_before, min_after): (usize, usize) = match odd_order {
            true => ((order + 1) / 2, (order + 1) / 2),
            false => (order / 2, order / 2 + 1),
        };

        if center_pos < min_before || sv_position.len() - center_pos < min_after {
            /* can't design time window */
            return None;
        }

        let mut polynomials = (0.0_f64, 0.0_f64, 0.0_f64);
        let offset = center_pos - min_before;

        for i in 0..order + 1 {
            let mut li = 1.0_f64;
            let (e_i, (x_i, y_i, z_i)) = sv_position[offset + i];
            for j in 0..order + 1 {
                let (e_j, _) = sv_position[offset + j];
                if j != i {
                    li *= (t - e_j).to_seconds();
                    li /= (e_i - e_j).to_seconds();
                }
            }
            polynomials.0 += x_i * li;
            polynomials.1 += y_i * li;
            polynomials.2 += z_i * li;
        }

        Some(polynomials)
    }
    /// Returns an Iterator over SV position vectors,
    /// expressed as geodetic coordinates, with latitude and longitude
    /// in decimal degrees.
    /// ```
    /// use rinex::prelude::*;
    ///
    /// let mut rinex =
    ///     Rinex::from_file("../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///         .unwrap();
    ///
    /// for (epoch, sv, (lat, lon, alt)) in rinex.sv_position_geo() {
    ///     // sv: satellite vehicle
    ///     // lat [ddeg]
    ///     // lon [ddeg]
    ///     // alt: [m ECEF]
    /// }
    /// ```
    pub fn sv_position_geo(&self) -> Box<dyn Iterator<Item = (Epoch, SV, (f64, f64, f64))> + '_> {
        Box::new(self.sv_position().map(|(e, sv, (x, y, z))| {
            let (lat, lon, alt) = ecef2geodetic(x, y, z, map_3d::Ellipsoid::WGS84);
            (e, sv, (lat, lon, alt))
        }))
    }
    /// Returns Iterator over SV speed vectors, expressed in km/s ECEF.
    /// ```
    /// use rinex::prelude::*;
    ///
    /// let mut rinex =
    ///     Rinex::from_file("../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///         .unwrap();
    ///
    /// //for (epoch, (sv, sv_x, sv_y, sv_z)) in rinex.sv_speed() {
    /// //    // sv_x : km/s
    /// //    // sv_y : km/s
    /// //    // sv_z : km/s
    /// //}
    /// ```
    pub fn sv_speed(&self) -> Box<dyn Iterator<Item = (Epoch, SV, (f64, f64, f64))> + '_> {
        todo!("sv_speed");
        //Box::new(
        //    self.sv_position()
        //    self.sv_position()
        //        .skip(1)
        //)
    }
    /// Returns an Iterator over SV elevation and azimuth angles,
    /// both expressed in degrees.
    /// A reference ground position must be known:
    ///   - either it is defined in [Header]
    ///   - otherwise it can be superceeded by user defined position
    ///   - if none of these conditions are matched, method will panic
    /// ```
    /// use rinex::wgs84;
    /// use rinex::prelude::*;
    /// let ref_pos = wgs84!(3582105.291, 532589.7313, 5232754.8054);
    ///
    /// let rinex = Rinex::from_file("../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///     .unwrap();
    ///
    /// let data = rinex.sv_elevation_azimuth(Some(ref_pos));
    /// for (epoch, sv, (elev, azim)) in data {
    ///     // azim: azimuth in °
    ///     // elev: elevation in °
    /// }
    /// ```
    pub fn sv_elevation_azimuth(
        &self,
        ref_position: Option<GroundPosition>,
    ) -> Box<dyn Iterator<Item = (Epoch, SV, (f64, f64))> + '_> {
        let ground_position = match ref_position {
            Some(pos) => pos, // user value superceeds, in case it is passed
            _ => {
                // header must contain this information
                // otherwise, calculation is not feasible
                if let Some(pos) = self.header.ground_position {
                    pos
                } else {
                    panic!("sv_elevation_azimuth(): needs a reference position");
                }
            },
        };
        Box::new(
            self.ephemeris()
                .filter_map(move |(epoch, (_, sv, ephemeris))| {
                    if let Some((elev, azim)) = ephemeris.sv_elev_azim(sv, *epoch, ground_position)
                    {
                        Some((*epoch, sv, (elev, azim)))
                    } else {
                        None // calculations may not be feasible,
                             // mainly when mandatory ephemeris broadcasts are missing
                    }
                }),
        )
    }
    /*
     * [IonMessage] Iterator
     */
    fn ionod_correction_models(
        &self,
    ) -> Box<dyn Iterator<Item = (Epoch, (NavMsgType, SV, IonMessage))> + '_> {
        /*
         * Answers both OLD and MODERN RINEX requirements
         * In RINEX2/3, midnight UTC is the publication datetime
         */
        let t0 = self.first_epoch().unwrap(); // will fail on invalid RINEX
        let t0 = Epoch::from_utc_days(t0.to_utc_days().round());
        Box::new(
            self.header
                .ionod_corrections
                .iter()
                .map(move |(c, ion)| (t0, (NavMsgType::LNAV, SV::new(*c, 1), *ion)))
                .chain(self.navigation().flat_map(|(t, frames)| {
                    frames.iter().filter_map(move |fr| {
                        let (msg, sv, ion) = fr.as_ion()?;
                        Some((*t, (msg, sv, *ion)))
                    })
                })),
        )
    }
    /// Returns [`KbModel`] Iterator.
    /// RINEX4 is the real application of this, as it provides model updates
    /// during the day. You're probably more interested
    /// in using [ionod_correction] instead of this, especially in PPP:
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::navigation::KbRegionCode;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    ///     .unwrap();
    /// for (epoch, _sv, kb_model) in rinex.klobuchar_models() {
    ///     let alpha = kb_model.alpha;
    ///     let beta = kb_model.beta;
    ///     assert_eq!(kb_model.region, KbRegionCode::WideArea);
    /// }
    /// ```
    /// We support all RINEX3 constellations. When working with this revision,
    /// you only get one model per day (24 hour validity period). [ionod_correction]
    /// does that verification internally.
    /// ```
    /// use std::str::FromStr;
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx")
    ///     .unwrap();
    /// let t0 = Epoch::from_str("2021-01-01T00:00:00 UTC")
    ///     .unwrap(); // model publication Epoch
    /// for (t, sv, model) in rinex.klobuchar_models() {
    ///     assert_eq!(t, t0);
    ///     // You should use "t==t0" to compare and verify model validity
    ///     // withint a 24 hour time frame.
    ///     // Note that we support all RINEX3 constellations
    ///     if sv.constellation == Constellation::BeiDou {
    ///         assert_eq!(model.alpha.0, 1.1176E-8);
    ///     }
    /// }
    /// ```
    /// Klobuchar models exists in RINEX2 and this method applies similarly.
    pub fn klobuchar_models(&self) -> Box<dyn Iterator<Item = (Epoch, SV, KbModel)> + '_> {
        Box::new(
            self.ionod_correction_models()
                .filter_map(|(t, (_, sv, ion))| ion.as_klobuchar().map(|model| (t, sv, *model))),
        )
    }
    /// Returns [`NgModel`] Iterator.
    /// RINEX4 is the real application of this, as it provides model updates
    /// during the day. You're probably more interested
    /// in using [ionod_correction] instead of this, especially in PPP:
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    ///     .unwrap();
    /// for (epoch, ng_model) in rinex.nequick_g_models() {
    ///     let (a0, a1, a2) = ng_model.a;
    ///     let region = ng_model.region; // bitflag: supports bitmasking operations
    /// }
    /// ```
    /// We support all RINEX3 constellations. When working with this revision,
    /// you only get one model per day (24 hour validity period). You should prefer
    /// [ionod_correction] which does that check internally:
    /// ```
    /// use std::str::FromStr;
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx")
    ///     .unwrap();
    /// let t0 = Epoch::from_str("2021-01-01T00:00:00 UTC")
    ///     .unwrap(); // model publication Epoch
    /// for (t, model) in rinex.nequick_g_models() {
    ///     assert_eq!(t, t0);
    ///     // You should use "t==t0" to compare and verify model validity
    ///     // within a 24 hour time frame.
    ///     assert_eq!(model.a.0, 66.25_f64);
    /// }
    /// ```
    /// Nequick-G model is not known to RINEX2 and only applies to RINEX V>2.
    pub fn nequick_g_models(&self) -> Box<dyn Iterator<Item = (Epoch, NgModel)> + '_> {
        Box::new(
            self.ionod_correction_models()
                .filter_map(|(t, (_, _, ion))| ion.as_nequick_g().map(|model| (t, *model))),
        )
    }
    /// Returns [`BdModel`] Iterator.
    /// RINEX4 is the real application of this, as it provides model updates
    /// during the day. You're probably more interested
    /// in using [ionod_correction] instead of this, especially in PPP:
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    ///     .unwrap();
    /// for (epoch, bd_model) in rinex.bdgim_models() {
    ///     let alpha_tecu = bd_model.alpha;
    /// }
    /// ```
    /// BDGIM was introduced in RINEX4, therefore this method does not apply
    /// to older revisions and returns an empty Iterator.
    pub fn bdgim_models(&self) -> Box<dyn Iterator<Item = (Epoch, BdModel)> + '_> {
        Box::new(
            self.ionod_correction_models()
                .filter_map(|(t, (_, _, ion))| ion.as_bdgim().map(|model| (t, *model))),
        )
    }
    /// Returns Ionospheric delay compensation, to apply at "t" desired Epoch
    /// and desired location. NB: we only support Klobuchar models at the moment,
    /// as we don't know how to convert other models (feel free to contribute).
    /// "t" must be within a 24 hour time frame of the oldest model.
    /// When working with RINEX2/3, the model is published at midnight
    /// and you should expect discontinuities when a new model is being published.
    pub fn ionod_correction(
        &self,
        t: Epoch,
        sv_elevation: f64,
        sv_azimuth: f64,
        user_lat_ddeg: f64,
        user_lon_ddeg: f64,
        carrier: Carrier,
    ) -> Option<f64> {
        // determine nearest in time
        let (_, (model_sv, model)) = self
            .ionod_correction_models()
            .filter_map(|(t_i, (_, sv_i, msg_i))| {
                // TODO
                // calculations currently limited to KB model: implement others
                let _ = msg_i.as_klobuchar()?;
                // At most 1 day from publication time
                if t_i <= t && (t - t_i) < 24.0 * Unit::Hour {
                    Some((t_i, (sv_i, msg_i)))
                } else {
                    None
                }
            })
            .min_by_key(|(t_i, _)| (t - *t_i))?;

        // TODO
        // calculations currently limited to KB model: implement others
        let kb = model.as_klobuchar().unwrap();
        let h_km = match model_sv.constellation {
            Constellation::BeiDou => 375.0,
            // we only expect BDS or GPS here,
            // wrongly formed RINEX will cause innacurate results
            Constellation::GPS | _ => 350.0,
        };
        Some(kb.meters_delay(
            t,
            sv_elevation,
            sv_azimuth,
            h_km,
            user_lat_ddeg,
            user_lon_ddeg,
            carrier,
        ))
    }
    /// Returns [`StoMessage`] frames Iterator
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    ///     .unwrap();
    /// for (epoch, (msg, sv, data)) in rnx.system_time_offset() {
    ///    let system = data.system.clone(); // time system
    ///    let utc = data.utc.clone(); // UTC provider
    ///    let t_tm = data.t_tm; // message transmission time in week seconds
    ///    let (a, dadt, ddadt) = data.a;
    /// }
    /// ```
    pub fn system_time_offset(
        &self,
    ) -> Box<dyn Iterator<Item = (&Epoch, (NavMsgType, SV, &StoMessage))> + '_> {
        Box::new(self.navigation().flat_map(|(e, frames)| {
            frames.iter().filter_map(move |fr| {
                if let Some((msg, sv, sto)) = fr.as_sto() {
                    Some((e, (msg, sv, sto)))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns [`EopMessage`] frames Iterator
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    ///     .unwrap();
    /// for (epoch, (msg, sv, eop)) in rnx.earth_orientation() {
    ///     let (x, dxdt, ddxdt) = eop.x;
    ///     let (y, dydt, ddydt) = eop.x;
    ///     let t_tm = eop.t_tm;
    ///     let (u, dudt, ddudt) = eop.delta_ut1;
    /// }
    /// ```
    pub fn earth_orientation(
        &self,
    ) -> Box<dyn Iterator<Item = (&Epoch, (NavMsgType, SV, &EopMessage))> + '_> {
        Box::new(self.navigation().flat_map(|(e, frames)| {
            frames.iter().filter_map(move |fr| {
                if let Some((msg, sv, eop)) = fr.as_eop() {
                    Some((e, (msg, sv, eop)))
                } else {
                    None
                }
            })
        }))
    }
}

/*
 * Meteo RINEX specific methods: only available on crate feature.
 * Either specific Iterators, or meaningful data we can extract.
 */
#[cfg(feature = "meteo")]
#[cfg_attr(docrs, doc(cfg(feature = "meteo")))]
impl Rinex {
    /// Returns temperature data iterator, values expressed in Celcius degrees
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, tmp) in rinex.temperature() {
    ///     println!("ts: {}, value: {} °C", epoch, tmp);
    /// }
    /// ```
    pub fn temperature(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::Temperature {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns pressure data iterator, values expressed in hPa
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, p) in rinex.pressure() {
    ///     println!("ts: {}, value: {} hPa", epoch, p);
    /// }
    /// ```
    pub fn pressure(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::Pressure {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns moisture rate iterator, values expressed in saturation rate percentage
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.moisture() {
    ///     println!("ts: {}, value: {} %", epoch, value);
    /// }
    /// ```
    pub fn moisture(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::HumidityRate {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns wind speed observations iterator, values in m/s
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, speed) in rinex.wind_speed() {
    ///     println!("ts: {}, value: {} m/s", epoch, speed);
    /// }
    /// ```
    pub fn wind_speed(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::WindSpeed {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns wind direction observations as azimuth in degrees
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, azimuth) in rinex.wind_direction() {
    ///     println!("ts: {}, azimuth: {}°", epoch, azimuth);
    /// }
    /// ```
    pub fn wind_direction(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::WindDirection {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns rain increment observations iterator, values in tenth of mm.
    /// Each value represents the accumulated rain drop in between two observations.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, ri) in rinex.rain_increment() {
    ///     println!("ts: {}, accumulated: {} mm/10", epoch, ri);
    /// }
    /// ```
    pub fn rain_increment(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::RainIncrement {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns total (wet+dry) Zenith delay, in mm
    /// ```
    /// use rinex::prelude::*;
    /// let mut rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.zenith_delay() {
    ///     println!("ts: {}, value: {} mm", epoch, value);
    /// }
    /// ```
    pub fn zenith_delay(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::ZenithTotalDelay {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns Zenith dry delay, in mm
    /// ```
    /// use rinex::prelude::*;
    /// let mut rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.zenith_dry_delay() {
    ///     println!("ts: {}, value: {} mm", epoch, value);
    /// }
    /// ```
    pub fn zenith_dry_delay(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::ZenithDryDelay {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns Zenith wet delay, in mm
    /// ```
    /// use rinex::prelude::*;
    /// let mut rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.zenith_wet_delay() {
    ///     println!("ts: {}, value: {} mm", epoch, value);
    /// }
    /// ```
    pub fn zenith_wet_delay(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::ZenithWetDelay {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns true if rain was detected during this time frame.
    /// ```
    /// use std::str::FromStr;
    /// use rinex::{filter, Rinex};
    /// use rinex::preprocessing::*; // .filter()
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// // cropping time frame requires the "processing" feature
    /// let rinex = rinex
    ///                 .filter(filter!(">= 2015-01-01T19:00:00 UTC"))
    ///                 .filter(filter!(" < 2015-01-01T20:00:00 UTC"));
    /// assert_eq!(rinex.rain_detected(), false);
    /// ```
    pub fn rain_detected(&self) -> bool {
        for (_, ri) in self.rain_increment() {
            if ri > 0.0 {
                return true;
            }
        }
        false
    }
    /// Returns total accumulated rain in tenth of mm, within this time frame
    /// ```
    /// use std::str::FromStr;
    /// use rinex::{filter, Rinex};
    /// use rinex::preprocessing::*; // .filter()
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// // cropping time frame requires the "processing" feature
    /// let rinex = rinex
    ///                 .filter(filter!(">= 2015-01-01T19:00:00 UTC"))
    ///                 .filter(filter!(" < 2015-01-01T19:30:00 UTC"));
    /// assert_eq!(rinex.accumulated_rain(), 0.0);
    /// assert_eq!(rinex.rain_detected(), false);
    /// ```
    pub fn accumulated_rain(&self) -> f64 {
        self.rain_increment()
            .zip(self.rain_increment().skip(1))
            .fold(0_f64, |mut acc, ((_, rk), (_, rkp1))| {
                if acc == 0.0_f64 {
                    acc = rkp1; // we take r(0) as starting offset
                } else {
                    acc += rkp1 - rk; // then accumulate the deltas
                }
                acc
            })
    }
    /// Returns true if hail was detected during this time frame
    /// ```
    /// use std::str::FromStr;
    /// use rinex::{filter, Rinex};
    /// use rinex::preprocessing::*; // .filter()
    /// let mut rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// // cropping time frame requires the "processing" feature
    /// let rinex = rinex
    ///                 .filter(filter!(">= 2015-01-01T19:00:00 UTC"))
    ///                 .filter(filter!(" < 2015-01-01T20:00:00 UTC"));
    /// assert_eq!(rinex.hail_detected(), false);
    /// ```
    pub fn hail_detected(&self) -> bool {
        if let Some(r) = self.record.as_meteo() {
            for observables in r.values() {
                for (observ, value) in observables {
                    if *observ == Observable::HailIndicator && *value > 0.0 {
                        return true;
                    }
                }
            }
            false
        } else {
            false
        }
    }
}

impl Merge for Rinex {
    /// Merges `rhs` into `Self` without mutable access, at the expense of memcopies
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self` in place
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        self.header.merge_mut(&rhs.header)?;
        if !self.is_antex() {
            if self.epoch().count() == 0 {
                // lhs is empty : overwrite
                self.record = rhs.record.clone();
            } else if rhs.epoch().count() != 0 {
                // real merge
                self.record.merge_mut(&rhs.record)?;
            }
        } else {
            // real merge
            self.record.merge_mut(&rhs.record)?;
        }
        Ok(())
    }
}

impl Split for Rinex {
    /// Splits `Self` at desired epoch
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        let (r0, r1) = self.record.split(epoch)?;
        Ok((
            Self {
                header: self.header.clone(),
                comments: self.comments.clone(),
                record: r0,
                prod_attr: self.prod_attr.clone(),
            },
            Self {
                header: self.header.clone(),
                comments: self.comments.clone(),
                record: r1,
                prod_attr: self.prod_attr.clone(),
            },
        ))
    }
    fn split_dt(&self, _duration: Duration) -> Result<Vec<Self>, split::Error> {
        Ok(Vec::new())
    }
}

#[cfg(feature = "processing")]
#[cfg_attr(docrs, doc(cfg(feature = "processing")))]
impl Preprocessing for Rinex {}

#[cfg(feature = "processing")]
#[cfg_attr(docrs, doc(cfg(feature = "processing")))]
impl Masking for Rinex {
    fn mask(&self, f: &MaskFilter) -> Self {
        let mut s = self.clone();
        s.mask_mut(f);
        s
    }
    fn mask_mut(&mut self, f: &MaskFilter) {
        if let Some(rec) = self.record.as_mut_obs() {
            observation_mask_mut(rec, f)
        } else if let Some(rec) = self.record.as_mut_nav() {
            navigation_mask_mut(rec, f)
        } else if let Some(rec) = self.record.as_mut_clock() {
            clock_mask_mut(rec, f)
        } else if let Some(rec) = self.record.as_mut_meteo() {
            meteo_mask_mut(rec, f)
        } else if let Some(rec) = self.record.as_mut_doris() {
            doris_mask_mut(rec, f)
        } else if let Some(rec) = self.record.as_mut_ionex() {
            ionex_mask_mut(rec, f)
        }
        header_mask_mut(&mut self.header, f);
    }
}

#[cfg(feature = "processing")]
#[cfg_attr(docrs, doc(cfg(feature = "processing")))]
impl Decimate for Rinex {
    fn decimate(&self, f: &DecimationFilter) -> Self {
        let mut s = self.clone();
        s.decimate_mut(f);
        s
    }
    fn decimate_mut(&mut self, f: &DecimationFilter) {
        if let Some(rec) = self.record.as_mut_obs() {
            observation_decim_mut(rec, f)
        } else if let Some(rec) = self.record.as_mut_nav() {
            navigation_decim_mut(rec, f)
        } else if let Some(rec) = self.record.as_mut_clock() {
            clock_decim_mut(rec, f)
        } else if let Some(rec) = self.record.as_mut_meteo() {
            meteo_decim_mut(rec, f)
        } else if let Some(rec) = self.record.as_mut_doris() {
            doris_decim_mut(rec, f)
        } else if let Some(rec) = self.record.as_mut_ionex() {
            ionex_decim_mut(rec, f)
        }
    }
}
#[cfg(feature = "obs")]
use observation::Dcb;

#[cfg(feature = "obs")]
#[cfg_attr(docrs, doc(cfg(feature = "obs")))]
impl Dcb for Rinex {
    fn dcb(&self) -> HashMap<String, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
        if let Some(r) = self.record.as_obs() {
            r.dcb()
        } else {
            panic!("wrong rinex type");
        }
    }
}

#[cfg(feature = "obs")]
use observation::{Combination, Combine};

#[cfg(feature = "obs")]
#[cfg_attr(docrs, doc(cfg(feature = "obs")))]
impl Combine for Rinex {
    fn combine(
        &self,
        c: Combination,
    ) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
        if let Some(r) = self.record.as_obs() {
            r.combine(c)
        } else {
            HashMap::new()
        }
    }
}

#[cfg(feature = "clock")]
use crate::clock::{ClockKey, ClockProfile, ClockProfileType};

/*
 * Clock RINEX specific feature
 */
#[cfg(feature = "clock")]
#[cfg_attr(docrs, doc(cfg(feature = "clock")))]
impl Rinex {
    /// Returns Iterator over Clock RINEX content.
    pub fn precise_clock(
        &self,
    ) -> Box<dyn Iterator<Item = (&Epoch, &BTreeMap<ClockKey, ClockProfile>)> + '_> {
        Box::new(
            self.record
                .as_clock()
                .into_iter()
                .flat_map(|record| record.iter()),
        )
    }
    /// Returns Iterator over Clock RINEX content for Space Vehicles only (not ground stations).
    pub fn precise_sv_clock(
        &self,
    ) -> Box<dyn Iterator<Item = (Epoch, SV, ClockProfileType, ClockProfile)> + '_> {
        Box::new(self.precise_clock().flat_map(|(epoch, rec)| {
            rec.iter().filter_map(|(key, profile)| {
                key.clock_type
                    .as_sv()
                    .map(|sv| (*epoch, sv, key.profile_type.clone(), profile.clone()))
            })
        }))
    }
    /// Interpolates Clock state at desired "t" expressed in the timescale you want.
    /// Clock RINEX usually have a high sample rate, this has two consequences
    ///  - it kind of allows clock states to be interpolated, as long as the
    ///  sample rate is <= 30s (be careful with the end results)
    ///   - they usually match the signal observation sampling.
    ///  If you Clock RINEX matches your OBS RINEX, you don't need interpolation at all.
    pub fn precise_sv_clock_interpolate(
        &self,
        t: Epoch,
        sv: SV,
    ) -> Option<(ClockProfileType, ClockProfile)> {
        let before = self
            .precise_sv_clock()
            .filter_map(|(clk_t, clk_sv, clk, prof)| {
                if clk_t <= t && clk_sv == sv {
                    Some((clk_t, clk, prof))
                } else {
                    None
                }
            })
            .last()?;
        let after = self
            .precise_sv_clock()
            .filter_map(|(clk_t, clk_sv, clk, prof)| {
                if clk_t > t && clk_sv == sv {
                    Some((clk_t, clk, prof))
                } else {
                    None
                }
            })
            .reduce(|k, _| k)?;
        let (before_t, clk_type, before_prof) = before;
        let (after_t, _, after_prof) = after;
        let dt = (after_t - before_t).to_seconds();
        let mut bias = (after_t - t).to_seconds() / dt * before_prof.bias;
        bias += (t - before_t).to_seconds() / dt * after_prof.bias;
        let drift: Option<f64> = match (before_prof.drift, after_prof.drift) {
            (Some(before_drift), Some(after_drift)) => {
                let mut drift = (after_t - t).to_seconds() / dt * before_drift;
                drift += (t - before_t).to_seconds() / dt * after_drift;
                Some(drift)
            },
            _ => None,
        };
        Some((
            clk_type,
            ClockProfile {
                bias,
                drift,
                bias_dev: None,
                drift_dev: None,
                drift_change: None,
                drift_change_dev: None,
            },
        ))
    }
    /// Returns Iterator over Clock RINEX content for Ground Station clocks only (not onboard clocks)
    pub fn precise_station_clock(
        &self,
    ) -> Box<dyn Iterator<Item = (Epoch, String, ClockProfileType, ClockProfile)> + '_> {
        Box::new(self.precise_clock().flat_map(|(epoch, rec)| {
            rec.iter().filter_map(|(key, profile)| {
                key.clock_type.as_station().map(|clk_name| {
                    (
                        *epoch,
                        clk_name.clone(),
                        key.profile_type.clone(),
                        profile.clone(),
                    )
                })
            })
        }))
    }
}

/*
 * IONEX specific feature
 */
#[cfg(feature = "ionex")]
#[cfg_attr(docrs, doc(cfg(feature = "ionex")))]
impl Rinex {
    /// Iterates over IONEX maps, per Epoch and altitude.
    /// ```
    /// use rinex::prelude::*;
    /// ```
    fn ionex(&self) -> Box<dyn Iterator<Item = (&(Epoch, i32), &TECPlane)> + '_> {
        Box::new(
            self.record
                .as_ionex()
                .into_iter()
                .flat_map(|record| record.iter()),
        )
    }
    /// Returns an iterator over TEC values exclusively.
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    /// for (t, lat, lon, alt, tec) in rnx.tec() {
    ///     // t: Epoch
    ///     // lat: ddeg
    ///     // lon: ddeg
    ///     // alt: km
    ///     // tec: TECu (f64: properly scaled)
    /// }
    /// ```
    pub fn tec(&self) -> Box<dyn Iterator<Item = (Epoch, f64, f64, f64, f64)> + '_> {
        Box::new(self.ionex().flat_map(|((e, h), plane)| {
            plane.iter().map(|((lat, lon), tec)| {
                (
                    *e,
                    *lat as f64 / 1000.0_f64,
                    *lon as f64 / 1000.0_f64,
                    *h as f64 / 100.0_f64,
                    tec.tec,
                )
            })
        }))
    }
    /// Returns an iterator over TEC RMS exclusively
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/IONEX/V1/jplg0010.17i.gz")
    ///     .unwrap();
    /// for (t, lat, lon, alt, rms) in rnx.tec_rms() {
    ///     // t: Epoch
    ///     // lat: ddeg
    ///     // lon: ddeg
    ///     // alt: km
    ///     // rms|TECu| (f64)
    /// }
    /// ```
    pub fn tec_rms(&self) -> Box<dyn Iterator<Item = (Epoch, f64, f64, f64, f64)> + '_> {
        Box::new(self.ionex().flat_map(|((e, h), plane)| {
            plane.iter().filter_map(|((lat, lon), tec)| {
                tec.rms.map(|rms| {
                    (
                        *e,
                        *lat as f64 / 1000.0_f64,
                        *lon as f64 / 1000.0_f64,
                        *h as f64 / 100.0_f64,
                        rms,
                    )
                })
            })
        }))
    }
    /// Returns 2D fixed altitude value, expressed in km, in case self is a 2D IONEX.
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/IONEX/V1/jplg0010.17i.gz")
    ///     .unwrap();
    /// assert_eq!(rnx.tec_fixed_altitude(), Some(450.0));
    ///
    /// let rnx = Rinex::from_file("../test_resources/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    /// assert_eq!(rnx.tec_fixed_altitude(), Some(350.0));
    /// ```
    pub fn tec_fixed_altitude(&self) -> Option<f64> {
        if self.is_ionex_2d() {
            let header = self.header.ionex.as_ref()?;
            Some(header.grid.height.start)
        } else {
            None
        }
    }
    /// Returns altitude range of this 3D IONEX as {min, max}
    /// both expressed in km.
    pub fn tec_altitude_range(&self) -> Option<(f64, f64)> {
        if self.is_ionex_3d() {
            let header = self.header.ionex.as_ref()?;
            Some((header.grid.height.start, header.grid.height.end))
        } else {
            None
        }
    }
    /// Returns 2D TEC plane at specified altitude and time.
    /// Refer to the header.grid specification for its width and height.
    pub fn tec_plane(&self, t: Epoch, h: f64) -> Option<&TECPlane> {
        self.ionex()
            .filter_map(|((e, alt), plane)| {
                if t == *e && (*alt as f64) / 100.0 == h {
                    Some(plane)
                } else {
                    None
                }
            })
            .reduce(|plane, _| plane) // is unique, in a normal IONEX
    }
    /// Returns IONEX map borders, expressed as North Eastern
    /// and South Western (latitude; longitude) coordinates,
    /// both expressed in ddeg.
    pub fn tec_map_borders(&self) -> Option<((f64, f64), (f64, f64))> {
        let ionex = self.header.ionex.as_ref()?;
        Some((
            (ionex.grid.latitude.start, ionex.grid.longitude.start),
            (ionex.grid.latitude.end, ionex.grid.longitude.end),
        ))
    }
}

/*
 * ANTEX specific feature
 */
#[cfg(feature = "antex")]
#[cfg_attr(docrs, doc(cfg(feature = "antex")))]
impl Rinex {
    /// Iterates over antenna specifications that are still valid
    pub fn antex_valid_calibrations(
        &self,
        now: Epoch,
    ) -> Box<dyn Iterator<Item = (&Antenna, &HashMap<Carrier, FrequencyDependentData>)> + '_> {
        Box::new(self.antennas().filter_map(move |(ant, data)| {
            if ant.is_valid(now) {
                Some((ant, data))
            } else {
                None
            }
        }))
    }
    /// Returns APC offset for given spacecraft, expressed in NEU coordinates [mm] for given
    /// frequency. "now" is used to determine calibration validity (in time).
    pub fn sv_antenna_apc_offset(
        &self,
        now: Epoch,
        sv: SV,
        freq: Carrier,
    ) -> Option<(f64, f64, f64)> {
        self.antex_valid_calibrations(now)
            .filter_map(|(ant, freqdata)| match &ant.specific {
                AntennaSpecific::SvAntenna(sv_ant) => {
                    if sv_ant.sv == sv {
                        freqdata
                            .get(&freq)
                            .map(|freqdata| freqdata.apc_eccentricity)
                    } else {
                        None
                    }
                },
                _ => None,
            })
            .reduce(|k, _| k) // we're expecting a single match here
    }
    /// Returns APC offset for given RX Antenna model (ground station model).
    /// Model name is the IGS code, which has to match exactly but we're case insensitive.
    /// The APC offset is expressed in NEU coordinates
    /// [mm]. "now" is used to determine calibration validity (in time).
    pub fn rx_antenna_apc_offset(
        &self,
        now: Epoch,
        matcher: AntennaMatcher,
        freq: Carrier,
    ) -> Option<(f64, f64, f64)> {
        let to_match = matcher.to_lowercase();
        self.antex_valid_calibrations(now)
            .filter_map(|(ant, freqdata)| match &ant.specific {
                AntennaSpecific::RxAntenna(rx_ant) => match &to_match {
                    AntennaMatcher::IGSCode(code) => {
                        if rx_ant.igs_type.to_lowercase().eq(code) {
                            freqdata
                                .get(&freq)
                                .map(|freqdata| freqdata.apc_eccentricity)
                        } else {
                            None
                        }
                    },
                    AntennaMatcher::SerialNumber(sn) => {
                        if rx_ant.igs_type.to_lowercase().eq(sn) {
                            freqdata
                                .get(&freq)
                                .map(|freqdata| freqdata.apc_eccentricity)
                        } else {
                            None
                        }
                    },
                },
                _ => None,
            })
            .reduce(|k, _| k) // we're expecting a single match here
    }
}

/*
 * DORIS special features
 */
#[cfg(feature = "doris")]
#[cfg_attr(docrs, doc(cfg(feature = "doris")))]
impl Rinex {
    /// Returns Stations Iterator
    pub fn stations(&self) -> Box<dyn Iterator<Item = &Station> + '_> {
        if let Some(doris) = &self.header.doris {
            Box::new(doris.stations.iter())
        } else {
            Box::new([].iter())
        }
    }
    /// Returns temperature data iterator, per DORIS station. Values expressed in Celcius degrees.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/DOR/V3/cs2rx18164.gz")
    ///     .unwrap();
    /// for (epoch, station, value) in rinex.doris_temperature() {
    ///     println!("{}@{}: {} °C", station.domes, epoch, value);
    /// }
    pub fn doris_temperature(&self) -> Box<dyn Iterator<Item = (Epoch, &Station, f64)> + '_> {
        Box::new(self.doris().flat_map(|((epoch, _), stations)| {
            stations.iter().flat_map(move |(station, observables)| {
                observables.iter().filter_map(move |(observable, data)| {
                    if *observable == Observable::Temperature {
                        Some((*epoch, station, data.value))
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Returns pressure data iterator, per DORIS station. Values expressed in hPa.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/DOR/V3/cs2rx18164.gz")
    ///     .unwrap();
    /// for (epoch, station, value) in rinex.doris_pressure() {
    ///     println!("{}@{}: {} hPa", station.domes, epoch, value);
    /// }
    pub fn doris_pressure(&self) -> Box<dyn Iterator<Item = (Epoch, &Station, f64)> + '_> {
        Box::new(self.doris().flat_map(|((epoch, _), stations)| {
            stations.iter().flat_map(move |(station, observables)| {
                observables.iter().filter_map(move |(observable, data)| {
                    if *observable == Observable::Pressure {
                        Some((*epoch, station, data.value))
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Humidity saturation rate Iterator, per DORIS station. Values expressed in percent.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/DOR/V3/cs2rx18164.gz")
    ///     .unwrap();
    /// for (epoch, station, value) in rinex.doris_humidity() {
    ///     println!("{}@{}: {}%", station.domes, epoch, value);
    /// }
    pub fn doris_humidity(&self) -> Box<dyn Iterator<Item = (Epoch, &Station, f64)> + '_> {
        Box::new(self.doris().flat_map(|((epoch, _), stations)| {
            stations.iter().flat_map(move |(station, observables)| {
                observables.iter().filter_map(move |(observable, data)| {
                    if *observable == Observable::HumidityRate {
                        Some((*epoch, station, data.value))
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Returns phase data iterator, per DORIS station. Values expressed in meters.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/DOR/V3/cs2rx18164.gz")
    ///     .unwrap();
    /// for (epoch, station, code, value) in rinex.doris_phase() {
    ///     println!("{} {}@{}: {}", station.domes, code, epoch, value);
    /// }
    pub fn doris_phase(
        &self,
    ) -> Box<dyn Iterator<Item = (Epoch, &Station, &Observable, f64)> + '_> {
        Box::new(self.doris().flat_map(|((epoch, _), stations)| {
            stations.iter().flat_map(move |(station, observables)| {
                observables.iter().filter_map(move |(observable, data)| {
                    if observable.is_phase_observable() {
                        Some((*epoch, station, observable, data.value))
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// (High precision) Pseudo Range Iterator, per DORIS station. Values expressed in meters.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/DOR/V3/cs2rx18164.gz")
    ///     .unwrap();
    /// for (epoch, station, code, value) in rinex.doris_pseudo_range() {
    ///     println!("{} {}@{}: {}m", station.domes, code, epoch, value);
    /// }
    pub fn doris_pseudo_range(
        &self,
    ) -> Box<dyn Iterator<Item = (Epoch, &Station, &Observable, f64)> + '_> {
        Box::new(self.doris().flat_map(move |((epoch, _), stations)| {
            stations.iter().flat_map(move |(station, observables)| {
                observables.iter().filter_map(move |(observable, data)| {
                    if observable.is_pseudorange_observable() {
                        if let Some(header) = &self.header.doris {
                            // apply a scaling (if any), otherwise preserve data precision
                            if let Some(scaling) = header.scaling.get(&observable) {
                                Some((*epoch, station, observable, data.value / *scaling as f64))
                            } else {
                                Some((*epoch, station, observable, data.value))
                            }
                        } else {
                            Some((*epoch, station, observable, data.value))
                        }
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Returns received signal power Iterator, as observed at each DORIS stations.
    /// Values expressed in [dBm].
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/DOR/V3/cs2rx18164.gz")
    ///     .unwrap();
    /// for (epoch, station, code, value) in rinex.doris_rx_power() {
    ///     println!("{} {}@{}: {} dBm", station.domes, code, epoch, value);
    /// }
    pub fn doris_rx_power(
        &self,
    ) -> Box<dyn Iterator<Item = (Epoch, &Station, &Observable, f64)> + '_> {
        Box::new(self.doris().flat_map(|((epoch, _), stations)| {
            stations.iter().flat_map(move |(station, observables)| {
                observables.iter().filter_map(move |(observable, data)| {
                    if observable.is_power_observable() {
                        Some((*epoch, station, observable, data.value))
                    } else {
                        None
                    }
                })
            })
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{fmt_comment, is_rinex_comment};
    use std::str::FromStr;
    #[test]
    fn fmt_comments_singleline() {
        for desc in [
            "test",
            "just a basic comment",
            "just another lengthy comment blahblabblah",
        ] {
            let comment = fmt_comment(desc);
            assert!(
                comment.len() >= 60,
                "comments should be at least 60 byte long"
            );
            assert_eq!(
                comment.find("COMMENT"),
                Some(60),
                "comment marker should located @ 60"
            );
            assert!(is_rinex_comment(&comment), "should be valid comment");
        }
    }
    #[test]
    fn fmt_wrapped_comments() {
        for desc in ["just trying to form a very lengthy comment that will overflow since it does not fit in a single line",
            "just trying to form a very very lengthy comment that will overflow since it does fit on three very meaningful lines. Imazdmazdpoakzdpoakzpdokpokddddddddddddddddddaaaaaaaaaaaaaaaaaaaaaaa"] {
            let nb_lines = num_integer::div_ceil(desc.len(), 60);
            let comments = fmt_comment(desc);
            assert_eq!(comments.lines().count(), nb_lines);
            for line in comments.lines() {
                assert!(line.len() >= 60, "comment line should be at least 60 byte long");
                assert_eq!(line.find("COMMENT"), Some(60), "comment marker should located @ 60");
                assert!(is_rinex_comment(line), "should be valid comment");
            }
        }
    }
    #[test]
    fn fmt_observables_v3() {
        for (desc, expected) in [
("R    9 C1C L1C S1C C2C C2P L2C L2P S2C S2P",
"R    9 C1C L1C S1C C2C C2P L2C L2P S2C S2P                  SYS / # / OBS TYPES"),
("G   18 C1C L1C S1C C2P C2W C2S C2L C2X L2P L2W L2S L2L L2X         S2P S2W S2S S2L S2X",
"G   18 C1C L1C S1C C2P C2W C2S C2L C2X L2P L2W L2S L2L L2X  SYS / # / OBS TYPES
       S2P S2W S2S S2L S2X                                  SYS / # / OBS TYPES"),
        ] {
            assert_eq!(fmt_rinex(desc, "SYS / # / OBS TYPES"), expected);
        }
    }
}
