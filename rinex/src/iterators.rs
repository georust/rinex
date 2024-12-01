use crate::prelude::{Epoch, EpochFlag, Observable, Rinex};

impl Rinex {
    /// Returns temperature measurements iterator, values expressed in Celcius degrees.
    /// Applies to Meteo RINEX and DORIS.
    ///
    /// Meteo example:
    /// ```
    /// use rinex::prelude::Rinex;
    ///
    /// let meteo = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    ///
    /// for (epoch, value) in meteo.temperature_iter() {
    ///     println!("{} value: {} °C", epoch, value);
    /// }
    /// ```
    ///
    /// DORIS example:
    /// ```
    /// use rinex::prelude::Rinex;
    ///
    /// let doris = Rinex::from_gzip_file("../test_resources/DOR/V3/cs2rx18164.gz")
    ///     .unwrap();
    ///
    /// for (epoch, value) in doris.temperature_iter() {
    ///     println!("{} value: {} °C", epoch, value);
    /// }
    /// ```
    pub fn temperature_iter(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        if self.is_meteo_rinex() {
            Box::new(self.meteo_observations_iter().filter_map(|(k, v)| {
                if k.observable == Observable::Temperature {
                    Some((k.epoch, *v))
                } else {
                    None
                }
            }))
        } else if self.is_doris() {
            Box::new(
                self.doris_ground_station_signals_iter()
                    .filter_map(|(k, v)| {
                        if v.observable == Observable::Temperature {
                            Some((k.epoch, v.value))
                        } else {
                            None
                        }
                    }),
            )
        } else {
            Box::new([].into_iter())
        }
    }

    /// Returns pressure measurements iterator, values expressed in hPa.
    /// Applies to Meteo RINEX and DORIS.
    ///
    /// Meteo example:
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.pressure_iter() {
    ///     println!("{} value: {} hPa", epoch, value);
    /// }
    /// ```
    ///
    /// DORIS pressure at observed ground stations:
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/DOR/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.pressure_iter() {
    ///     println!("{} value: {} hPa", epoch, value);
    /// }
    /// ```
    pub fn pressure_iter(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        if self.is_meteo_rinex() {
            Box::new(self.meteo_observations_iter().filter_map(|(k, v)| {
                if k.observable == Observable::Pressure {
                    Some((k.epoch, *v))
                } else {
                    None
                }
            }))
        } else if self.is_doris() {
            Box::new(
                self.doris_ground_station_signals_iter()
                    .filter_map(|(k, v)| {
                        if v.observable == Observable::Pressure {
                            Some((k.epoch, v.value))
                        } else {
                            None
                        }
                    }),
            )
        } else {
            Box::new([].into_iter())
        }
    }

    /// Returns moisture rate measurement iterator, values expressed in saturation percentage rate.
    /// Applies to Meteo RINEX and DORIS.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.moisture_rate_iter() {
    ///     println!("{} value: {} %", epoch, value);
    /// }
    /// ```
    pub fn moisture_rate_iter(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        if self.is_meteo_rinex() {
            Box::new(self.meteo_observations_iter().filter_map(|(k, v)| {
                if k.observable == Observable::HumidityRate {
                    Some((k.epoch, *v))
                } else {
                    None
                }
            }))
        } else if self.is_doris() {
            Box::new(
                self.doris_ground_station_signals_iter()
                    .filter_map(|(k, v)| {
                        if v.observable == Observable::HumidityRate {
                            Some((k.epoch, v.value))
                        } else {
                            None
                        }
                    }),
            )
        } else {
            Box::new([].into_iter())
        }
    }
}
