use crate::prelude::{Epoch, SP3, SV};
use std::path::PathBuf;
use std::str::FromStr;

#[test]
fn satellite_maneuver_flag() {
    let path = PathBuf::new()
        .join(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("test_resources")
        .join("SP3")
        .join("sp3d.txt");

    let sp3 = SP3::from_file(&path).unwrap();

    let g01 = SV::from_str("G01").unwrap();
    let t0 = Epoch::from_str("2019-10-27T00:00:00 GPST").unwrap();

    assert!(sp3.has_satellite_maneuver());

    for (_, sv, _) in sp3.satellites_position_km_iter() {
        assert!(sv != g01, "vehicles being maneuvered should be excluded");
    }

    for (t, sv) in sp3.satellites_epoch_maneuver_iter() {
        assert_eq!(t, t0);
        assert_eq!(sv, g01);
    }

    for (t, sv) in sp3.satellites_epoch_clock_event_iter() {
        assert_eq!(t, t0);
        assert_eq!(sv, g01);
    }
}
