use crate::context::RnxContext;

/// Dual context for DGNSS using RINEX data
#[derive(Default, Debug, Clone)]
#[cfg_attr(docrs, doc(cfg(feature = "rtk")))]
pub struct RTKContext {
    /// Rover context
    pub rover: RnxContext,
    /// Base station reference context
    pub base: RnxContext,
}
