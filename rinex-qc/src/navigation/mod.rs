// pub(crate) mod buffer;
pub(crate) mod eph;
// pub(crate) mod gpx;
// pub(crate) mod kml;
pub(crate) mod clock;
pub(crate) mod orbit;
pub(crate) mod pvt;
pub(crate) mod signal;

#[cfg(feature = "cggtts")]
pub(crate) mod cggtts;

use gnss_rtk::prelude::Carrier as RTKCarrier;
use rinex::prelude::Carrier;

/// Converts [Carrier] to [RTKCarrier]
pub(crate) fn carrier_to_rtk(carrier: &Carrier) -> Option<RTKCarrier> {
    match carrier {
        Carrier::L1 => Some(RTKCarrier::L1),
        Carrier::L2 => Some(RTKCarrier::L2),
        Carrier::L5 => Some(RTKCarrier::L5),
        Carrier::L6 => Some(RTKCarrier::L6),
        Carrier::E1 => Some(RTKCarrier::E1),
        Carrier::E5 => Some(RTKCarrier::E5),
        Carrier::E6 => Some(RTKCarrier::E6),
        Carrier::E5a => Some(RTKCarrier::E5A),
        Carrier::E5b => Some(RTKCarrier::E5B),
        Carrier::B2 => Some(RTKCarrier::B2),
        Carrier::B2A => Some(RTKCarrier::B2A),
        Carrier::B2B | Carrier::B2I => Some(RTKCarrier::B2iB2b),
        Carrier::B1I => Some(RTKCarrier::B1I),
        Carrier::B1A | Carrier::B1C => Some(RTKCarrier::B1aB1c),
        Carrier::B3 | Carrier::B3A => Some(RTKCarrier::B3),
        Carrier::G1(_) | Carrier::G1a | Carrier::G2(_) | Carrier::G2a | Carrier::G3 => None,
        Carrier::S => None,
        Carrier::S1 | Carrier::U2 => None,
    }
}

pub(crate) fn carrier_from_rtk(carrier: &RTKCarrier) -> Carrier {
    match carrier {
        RTKCarrier::B1I => Carrier::B1I,
        RTKCarrier::B1aB1c => Carrier::B1A,
        RTKCarrier::B2 => Carrier::B2,
        RTKCarrier::B2A => Carrier::B2A,
        RTKCarrier::B2iB2b => Carrier::B2I,
        RTKCarrier::B3 => Carrier::B3,
        RTKCarrier::E1 => Carrier::E1,
        RTKCarrier::E5 => Carrier::E5,
        RTKCarrier::E5A => Carrier::E5a,
        RTKCarrier::E5B => Carrier::E5b,
        RTKCarrier::E6 => Carrier::E6,
        RTKCarrier::L1 => Carrier::L1,
        RTKCarrier::L2 => Carrier::L2,
        RTKCarrier::L5 => Carrier::L5,
        RTKCarrier::L6 => Carrier::L6,
    }
}
