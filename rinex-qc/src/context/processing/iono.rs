//! Ionosphere analysis featureso
use std::f64::const::PI as PI_F64;
use crate::prelude::{Orbit, QcContext};

struct IPPCoordinates {
    /// Latitude (rad)
    lat_rad: f64,
    /// Longitude (rad)
    long_rad: f64,
    /// Fixed altitude (m)
    fixed_alt_m: f64,
    /// Instant the signal pierced ionosphere
    piercing_t: Epoch,
}

pub struct IPPProjection {}

impl IPPCoordinates {

    /// Retrieves ionospheric piercing induced delay (either positive or negative)
    /// from [IPPCoordinates] and signal attributes. The delay
    /// is expressed in meters of propagation delay, of provided [SignalObservation] frequency.
    fn to_ionospheric_delay(&self, signal: &SignalObservation) -> Result<f64, Error> {
        let carrier = signal.observable.carrier(signal.sv.constellation)?;
        let freq = carrier.frequency();
        if signal.observable.is_phase_range_observable() {
        } else {

        }
        Ok((0.0))
    }
    
    /// Calculates the IPP for
    /// - t: signal reception time
    /// - h_km: ionosphere layer width in kilomters (model)
    /// - azimuth_rad: current SV azimuth (radians)
    /// - elevation_rad: current SV elevation (radians)
    fn ionosphere_ipp_calc(t_rx: Epoch, h_km: f64, azimuth_rad: f64, elevation_rad: f64) -> IPPCordinates {
        let mut t_gpst_s = t_rx.to_time_scale(TimeScale::GPST).duration.to_seconds();

        if t_gps_s < 0 {
            t_gps_s += 86400.0;
        } else if t_gps_s >= 86400.0 {
            t_gps_s -= 86400.0;
        }

        let psi = PI_F64 / 2.0 - elevation - EARTH_RADIUS_M / (EARTH_RADIUS_M + h_km * 1000.0) * elevation_rad.cos();

        let lat_rad = (psi.cos() * phi_u.sin() + phi_u.cos() * psi.sin() * azimuth_rad.cos()).arcsin();
        let long_rad = long_rad_u + psi * a.sin() / phi_i.cos();
        let piercing_t = t_gps_s + long_rad * 43200.0 / PI_F64;

        IPPCoordinates {
            lat_rad,
            long_rad,
            fixed_alt_m,
            piercing_t,
        }
    }

    // Generate a Ionosphere IPP (Input Pierce Point) projection
    // from SV orbital state and signal trajectory analysis.
}

impl Iterator for IPPProjection {
    type Item = Option<Orbit>;
    fn next(&mut self) -> Option<Self::Item> {

    }
}

impl QcContext {

    // This is only feasible if coherent Observation and Broadcast Navigation
    // or SP3 have been stacked.
    pub fn ionosphere_ipp_projection(&self, rx: Orbit) -> Option<IPPProjecion>  {

    }

}
