//! Meteo RINEX module
pub mod record;
pub mod sensor;
pub use record::Record;

use crate::Observable;

/// Meteo specific header fields
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Observation types contained in this file
    pub codes: Vec<Observable>,
    /// Sensors that produced the following observables
    pub sensors: Vec<sensor::Sensor>,
}

use crate::prelude::Epoch;

/// Meteo RINEX record iteration methods.
/// Faillible: if used on other RINEX types
pub trait MeteoIter {
    /// Returns temperature data iterator, values expressed in Celcius degrees
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::meteo::MeteoIter;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, tmp) in rinex.temperature() {
    ///     println!("ts: {}, value: {} °C", epoch, tmp);
    /// }
    /// ```
    fn temperature(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_>;
    /// Returns pressure data iterator, values expressed in hPa
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::meteo::MeteoIter;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, p) in rinex.pressure() {
    ///     println!("ts: {}, value: {} hPa", epoch, p);
    /// }
    /// ```
    fn pressure(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_>;
    /// Returns moisture rate iterator, values expressed in percent
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::meteo::MeteoIter;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.moisture() {
    ///     println!("ts: {}, value: {} %", epoch, value);
    /// }
    /// ```
    fn moisture(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_>;
    /// Returns wind direction as azimuth in degrees
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::meteo::MeteoIter;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, azimuth) in rinex.wind_direction() {
    ///     println!("ts: {}, azimuth: {}°", epoch, azimuth);
    /// }
    /// ```
    fn wind_direction(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_>;
    /// Returns wind speed estimates iterator, values in m/s
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::meteo::MeteoIter;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, speed) in rinex.wind_direction() {
    ///     println!("ts: {}, value: {} m/s", epoch, speed);
    /// }
    /// ```
    fn wind_speed(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_>;
    /// Returns rain increment values iterator, values in tenth of mm.
    /// Each value represents the accumulated rain drop since previous estimate.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::meteo::MeteoIter;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, ri) in rinex.rain_increment() {
    ///     println!("ts: {}, accumulated: {} mm/10", epoch, ri);
    /// }
    /// ```
    fn rain_increment(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_>;
    /// Returns Zenith dry delay, in mm
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::meteo::MeteoIter;
    /// let mut rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.zenith_dry_delay() {
    ///     println!("ts: {}, value: {} mm", epoch, value);
    /// }
    /// ```
    fn zenith_dry_delay(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_>;
    /// Returns Zenith wet delay, in mm
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::meteo::MeteoIter;
    /// let mut rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.zenith_wet_delay() {
    ///     println!("ts: {}, value: {} mm", epoch, value);
    /// }
    /// ```
    fn zenith_wet_delay(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_>;
    /// Returns Total (Wet + Dry) Zenith delay, in mm
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::meteo::MeteoIter;
    /// let mut rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.zenith_delay() {
    ///     println!("ts: {}, value: {} mm", epoch, value);
    /// }
    /// ```
    fn zenith_delay(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_>;
}

#[cfg(feature = "meteo")]
#[cfg_attr(docrs, doc(cfg(feature = "meteo")))]
pub trait Meteo {
    /// Returns total accumulated rain in tenth of mm, within this time frame
    /// ```
    /// use std::str::FromStr;
    /// use rinex::meteo::Meteo;
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
    fn accumulated_rain(&self) -> f64;
    /// Returns true if rain was detected during this time frame.
    /// ```
    /// use std::str::FromStr;
    /// use rinex::meteo::Meteo;
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
    fn rain_detected(&self) -> bool;
    /// Returns true if hail was detected during this time frame
    /// ```
    /// use std::str::FromStr;
    /// use rinex::meteo::Meteo;
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
    fn hail_detected(&self) -> bool;
}
