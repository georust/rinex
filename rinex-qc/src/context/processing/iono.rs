//! Ionosphere analysis featureso
use crate::prelude::{Orbit, QcContext};
use std::consts::f64::PI as PI_F64;

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

/// [IPPProjection] allows Iterating the Ionosphere
/// piercing coordinates of a signal.
pub struct IPPProjection {
    /// [SV] is the signal source
    pub sv: SV,
    /// [Observable] describes both frequency and modulation
    pub signal: Observable,
    /// Space time coordinates expressed as [IPPCoordinates]
    pub coordinates: IPPCoordinates,
}

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
    fn ionosphere_ipp_calc(
        t_rx: Epoch,
        h_km: f64,
        azimuth_rad: f64,
        elevation_rad: f64,
    ) -> IPPCordinates {
        let mut t_gpst_s = t_rx.to_time_scale(TimeScale::GPST).duration.to_seconds();

        if t_gps_s < 0 {
            t_gps_s += 86400.0;
        } else if t_gps_s >= 86400.0 {
            t_gps_s -= 86400.0;
        }

        let psi = PI_F64 / 2.0
            - elevation
            - EARTH_RADIUS_M / (EARTH_RADIUS_M + h_km * 1000.0) * elevation_rad.cos();

        let lat_rad =
            (psi.cos() * phi_u.sin() + phi_u.cos() * psi.sin() * azimuth_rad.cos()).arcsin();
        let long_rad = long_rad_u + psi * a.sin() / phi_i.cos();
        let piercing_t = t_gps_s + long_rad * 43200.0 / PI_F64;

        IPPCoordinates {
            lat_rad,
            long_rad,
            fixed_alt_m,
            piercing_t,
        }
    }
}

impl QcContext {
    // Iterate through IPProjection] if [QcContext] allows it.
    // - Dual frequency Observation RINEX is required
    // - Navigation RINEX or SP3 must be stacked as well
    pub fn ionosphere_ipp_proj(&self, rx: Orbit) -> Box<dyn Iterator<Item = IPPProjection> + '_> {
        if let Some(obs_rinex) = self.obs_rinex_data() {}
    }
}
