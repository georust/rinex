//! NAV RINEX module
mod eopmessage;
mod ephemeris;
mod health;
mod ionmessage;
mod stomessage;

pub mod orbits;
pub mod record;

pub use eopmessage::EopMessage;
pub use ephemeris::Ephemeris;
pub use health::{GeoHealth, GloHealth, Health, IrnssHealth};
pub use ionmessage::{BdModel, IonMessage, KbModel, KbRegionCode, NgModel, NgRegionFlags};
pub use orbits::OrbitItem;
pub use record::{Frame, FrameClass, MsgType, Record};
pub use stomessage::StoMessage;

#[cfg(feature = "nav")]
use crate::{Epoch, GroundPosition};

#[cfg(feature = "nav")]
use std::collections::BTreeMap;

pub trait Navigation {
    /// Returns position vectors as geodetic coordinates
    /// expressed in decimal degrees ECEF, for all Epochs.
    fn sv_geodetic(&self) -> BTreeMap<Epoch, (f64, f64, f64)>;
    
    /// Returns position vector at desired Epoch, as geodetic coordinates
    /// expressed in decimal degrees ECEF.
    fn sv_geodetic_epoch(&self, epoch: Epoch) -> Option<(f64, f64, f64)>;
    
    /// Returns speed vectors, in meters/sec ECEF for all Epochs.
    fn sv_speed(&self) -> BTreeMap<Epoch, (f64, f64, f64)>;
    
    /// Returns speed vector at desired Epoch, expressed in meters/sec ECEF.
    fn sv_speed_epoch(&self, epoch: Epoch) -> Option<(f64, f64, f64)>;
    
    /// Returns acceleration vectors, in meters/sec-2 ECEF for all Epochs.
    fn sv_accel(&self) -> BTreeMap<Epoch, (f64, f64, f64)>;
    
    /// Returns acceleration vector at desired Epoch, in meters/sec-2 ECEF.
    fn sv_accel_epoch(&self, epoch: Epoch) -> Option<(f64, f64, f64)>;
}
