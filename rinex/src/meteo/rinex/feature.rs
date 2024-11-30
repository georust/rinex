//! Feature dependent high level methods

use crate::{
    meteo::Sensor,
    prelude::{Epoch, Observable, Rinex},
};

impl Rinex {
    /// Returns Meteo [Sensor]s Iterator
    pub fn meteo_sensors_iter(&self) -> Box<dyn Iterator<Item = &Sensor> + '_> {
        if let Some(meteo) = &self.header.meteo {
            Box::new(meteo.sensors.iter())
        } else {
            Box::new([].into_iter())
        }
    }

    /// Returns temperature measurements iterator, values expressed in Celcius degrees.
    /// Applies to Meteo RINEX.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, tmp) in rinex.temperature() {
    ///     println!("ts: {}, value: {} °C", epoch, tmp);
    /// }
    /// ```
    pub fn temperature_iter(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo_observations_iter().filter_map(|(k, v)| {
            if k.observable == Observable::Temperature {
                Some((k.epoch, *v))
            } else {
                None
            }
        }))
    }

    //   /// Returns pressure data iterator, values expressed in hPa
    //   /// ```
    //   /// use rinex::prelude::*;
    //   /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    //   ///     .unwrap();
    //   /// for (epoch, p) in rinex.pressure() {
    //   ///     println!("ts: {}, value: {} hPa", epoch, p);
    //   /// }
    //   /// ```
    //   pub fn pressure(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
    //       Box::new(self.meteo().flat_map(|(epoch, v)| {
    //           v.iter().filter_map(|(k, value)| {
    //               if *k == Observable::Pressure {
    //                   Some((*epoch, *value))
    //               } else {
    //                   None
    //               }
    //           })
    //       }))
    //   }

    //   /// Returns moisture rate iterator, values expressed in saturation rate percentage
    //   /// ```
    //   /// use rinex::prelude::*;
    //   /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    //   ///     .unwrap();
    //   /// for (epoch, value) in rinex.moisture() {
    //   ///     println!("ts: {}, value: {} %", epoch, value);
    //   /// }
    //   /// ```
    //   pub fn moisture(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
    //       Box::new(self.meteo().flat_map(|(epoch, v)| {
    //           v.iter().filter_map(|(k, value)| {
    //               if *k == Observable::HumidityRate {
    //                   Some((*epoch, *value))
    //               } else {
    //                   None
    //               }
    //           })
    //       }))
    //   }

    //   /// Returns wind speed observations iterator, values in m/s
    //   /// ```
    //   /// use rinex::prelude::*;
    //   /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    //   ///     .unwrap();
    //   /// for (epoch, speed) in rinex.wind_speed() {
    //   ///     println!("ts: {}, value: {} m/s", epoch, speed);
    //   /// }
    //   /// ```
    //   pub fn wind_speed(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
    //       Box::new(self.meteo().flat_map(|(epoch, v)| {
    //           v.iter().filter_map(|(k, value)| {
    //               if *k == Observable::WindSpeed {
    //                   Some((*epoch, *value))
    //               } else {
    //                   None
    //               }
    //           })
    //       }))
    //   }

    //   /// Returns wind direction observations as azimuth in degrees
    //   /// ```
    //   /// use rinex::prelude::*;
    //   /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    //   ///     .unwrap();
    //   /// for (epoch, azimuth) in rinex.wind_direction() {
    //   ///     println!("ts: {}, azimuth: {}°", epoch, azimuth);
    //   /// }
    //   /// ```
    //   pub fn wind_direction(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
    //       Box::new(self.meteo().flat_map(|(epoch, v)| {
    //           v.iter().filter_map(|(k, value)| {
    //               if *k == Observable::WindDirection {
    //                   Some((*epoch, *value))
    //               } else {
    //                   None
    //               }
    //           })
    //       }))
    //   }

    //   /// Returns rain increment observations iterator, values in tenth of mm.
    //   /// Each value represents the accumulated rain drop in between two observations.
    //   /// ```
    //   /// use rinex::prelude::*;
    //   /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    //   ///     .unwrap();
    //   /// for (epoch, ri) in rinex.rain_increment() {
    //   ///     println!("ts: {}, accumulated: {} mm/10", epoch, ri);
    //   /// }
    //   /// ```
    //   pub fn rain_increment(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
    //       Box::new(self.meteo().flat_map(|(epoch, v)| {
    //           v.iter().filter_map(|(k, value)| {
    //               if *k == Observable::RainIncrement {
    //                   Some((*epoch, *value))
    //               } else {
    //                   None
    //               }
    //           })
    //       }))
    //   }

    //   /// Returns total (wet+dry) Zenith delay, in mm
    //   /// ```
    //   /// use rinex::prelude::*;
    //   /// let mut rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    //   ///     .unwrap();
    //   /// for (epoch, value) in rinex.zenith_delay() {
    //   ///     println!("ts: {}, value: {} mm", epoch, value);
    //   /// }
    //   /// ```
    //   pub fn zenith_delay(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
    //       Box::new(self.meteo().flat_map(|(epoch, v)| {
    //           v.iter().filter_map(|(k, value)| {
    //               if *k == Observable::ZenithTotalDelay {
    //                   Some((*epoch, *value))
    //               } else {
    //                   None
    //               }
    //           })
    //       }))
    //   }

    //   /// Returns Zenith dry delay, in mm
    //   /// ```
    //   /// use rinex::prelude::*;
    //   /// let mut rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    //   ///     .unwrap();
    //   /// for (epoch, value) in rinex.zenith_dry_delay() {
    //   ///     println!("ts: {}, value: {} mm", epoch, value);
    //   /// }
    //   /// ```
    //   pub fn zenith_dry_delay(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
    //       Box::new(self.meteo().flat_map(|(epoch, v)| {
    //           v.iter().filter_map(|(k, value)| {
    //               if *k == Observable::ZenithDryDelay {
    //                   Some((*epoch, *value))
    //               } else {
    //                   None
    //               }
    //           })
    //       }))
    //   }

    //   /// Returns Zenith wet delay, in mm
    //   /// ```
    //   /// use rinex::prelude::*;
    //   /// let mut rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    //   ///     .unwrap();
    //   /// for (epoch, value) in rinex.zenith_wet_delay() {
    //   ///     println!("ts: {}, value: {} mm", epoch, value);
    //   /// }
    //   /// ```
    //   pub fn zenith_wet_delay(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
    //       Box::new(self.meteo().flat_map(|(epoch, v)| {
    //           v.iter().filter_map(|(k, value)| {
    //               if *k == Observable::ZenithWetDelay {
    //                   Some((*epoch, *value))
    //               } else {
    //                   None
    //               }
    //           })
    //       }))
    //   }

    //   /// Returns true if rain was detected during this time frame.
    //   /// ```
    //   /// use std::str::FromStr;
    //   /// use rinex::prelude::*;
    //   /// use rinex::prelude::Preprocessing; // only on "processing" feature
    //   ///
    //   /// // parse a RINEX
    //   /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    //   ///     .unwrap();
    //   ///
    //   /// // only on "processing" feature
    //   /// let morning = Filter::lower_than("2015-01-01T12:00:00 UTC")
    //   ///     .unwrap();
    //   ///
    //   /// let rinex = rinex.filter(&morning);
    //   /// assert_eq!(rinex.rain_detected(), false);
    //   /// ```
    //   pub fn rain_detected(&self) -> bool {
    //       for (_, ri) in self.rain_increment() {
    //           if ri > 0.0 {
    //               return true;
    //           }
    //       }
    //       false
    //   }

    //   /// Returns total accumulated rain in tenth of mm, within this time frame
    //   /// ```
    //   /// use std::str::FromStr;
    //   /// use rinex::prelude::*;
    //   /// use rinex::prelude::Preprocessing; // only on "processing" feature
    //   ///
    //   /// // parse a RINEX
    //   /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    //   ///     .unwrap();
    //   ///
    //   /// // only when built with "processing" feature
    //   /// let afternoon = Filter::greater_than("2015-01-01T12:00:00 UTC")
    //   ///     .unwrap();
    //   ///
    //   /// let rinex = rinex.filter(&afternoon);
    //   /// assert_eq!(rinex.accumulated_rain(), 0.0);
    //   /// assert_eq!(rinex.rain_detected(), false);
    //   /// ```
    //   pub fn accumulated_rain(&self) -> f64 {
    //       self.rain_increment()
    //           .zip(self.rain_increment().skip(1))
    //           .fold(0_f64, |mut acc, ((_, rk), (_, rkp1))| {
    //               if acc == 0.0_f64 {
    //                   acc = rkp1; // we take r(0) as starting offset
    //               } else {
    //                   acc += rkp1 - rk; // then accumulate the deltas
    //               }
    //               acc
    //           })
    //   }

    //   /// Returns true if hail was detected during this time frame
    //   /// ```
    //   /// use std::str::FromStr;
    //   /// use rinex::prelude::*;
    //   /// use rinex::prelude::Preprocessing; // only on "processing" feature
    //   ///
    //   /// // parse a RINEX
    //   /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    //   ///     .unwrap();
    //   ///
    //   /// // only when built with "processing" feature
    //   /// let morning = Filter::lower_than("2015-01-01T12:00:00 UTC")
    //   ///     .unwrap();
    //   ///
    //   /// let rinex = rinex.filter(&morning);
    //   /// assert_eq!(rinex.accumulated_rain(), 0.0);
    //   /// assert_eq!(rinex.rain_detected(), false);
    //   /// ```
    //   pub fn hail_detected(&self) -> bool {
    //       if let Some(r) = self.record.as_meteo() {
    //           for observables in r.values() {
    //               for (observ, value) in observables {
    //                   if *observ == Observable::HailIndicator && *value > 0.0 {
    //                       return true;
    //                   }
    //               }
    //           }
    //           false
    //       } else {
    //           false
    //       }
    //   }
} //
