use crate::{ionex::IonosphereModel, prelude::Epoch};

/// IPPCoordinates; Ionosphere Pierce Point Coordinates,
/// describe the location in space-time a signal pierced
/// the Ionosphere layer.
struct IPPCoordinates {
    /// Latitude (rad)
    lat_rad: f64,
    /// Longitude (rad)
    long_rad: f64,
    /// Instant the signal pierced ionosphere
    piercing_t: Epoch,
}

impl IPPCoordinates {
    pub fn piercing_epoch(&self) -> Epoch {
        self.piercing_t
    }
    pub fn latitude_ddeg(&self) -> f64 {
        self.lat_rad.to_degrees()
    }
    pub fn latitude_rad(&self) -> f64 {
        self.lat_rad
    }
    pub fn longitude_ddeg(&self) -> f64 {
        self.long_rad.to_degrees()
    }
    pub fn longitude_rad(&self) -> f64 {
        self.long_rad
    }

    /// Deduce [IonosphereParameters] from these [IPPCoordinates]
    pub fn to_parameters_model(&self) -> IonosphereParameters {}
}
