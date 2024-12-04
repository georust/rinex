use crate::prelude::QcContext;

impl QcContext {
    /// Resolve requested SV [Orbit]al attitude if [QcContext] allows it.
    /// Requirements:
    /// - Navigation RINEX or SP3 is mandatory
    /// ## Input
    ///  - t: signal reception time (as [Epoch])
    ///  - sv: [SV] target
    pub fn sv_orbit_proj(t: Epoch, sv: SV) -> Option<Orbit> {}
}
