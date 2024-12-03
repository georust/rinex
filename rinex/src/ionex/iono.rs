/// Modeled Ionosphere characteristics
pub struct IonosphereParameters {
    /// Amplitude of the ionospheric delay (s)
    pub amplitude_s: f64,
    /// Period of the ionospheric delay (s)
    pub period_s: f64,
    /// Phase of the ionospheric delay (rad)
    pub phase_rad: f64,
    /// Slant factor is the projection factor
    /// from a vertical signal propagation,
    /// to actual angled shifted propagation (no unit)
    pub slant_factor: f64,
}
