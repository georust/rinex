use std::str::FromStr;

use crate::{
    ionex::MappingFunction,
    prelude::{Epoch, Header, Rinex, RinexType},
    tests::toolkit::{generic_rinex_test, TimeFrame},
};

/// Basic tests for Observation [Rinex]
fn basic_header_tests(
    dut: &Header,
    epoch_of_first_map: &str,
    epoch_of_last_map: &str,
    lat_grid_start: f64,
    lat_grid_end: f64,
    lat_grid_spacing: f64,
    long_grid_start: f64,
    long_grid_end: f64,
    long_grid_spacing: f64,
    h_grid_start: Option<f64>,
    h_grid_end: Option<f64>,
    h_grid_spacing: Option<f64>,
    exponent: i8,
    base_radius: f32,
    elevation_cutoff: f32,
    mapping_function: Option<&str>,
) {
    assert!(dut.obs.is_none());
    assert!(dut.meteo.is_none());
    assert!(dut.clock.is_none());
    assert!(dut.ionex.is_some());

    let specs = dut.ionex.as_ref().expect("missing specific specs");

    let epoch_of_first_map = Epoch::from_str(epoch_of_first_map.trim()).unwrap();
    assert_eq!(specs.epoch_of_first_map, epoch_of_first_map);

    let epoch_of_last_map = Epoch::from_str(epoch_of_last_map.trim()).unwrap();
    assert_eq!(specs.epoch_of_last_map, epoch_of_last_map);

    assert_eq!(specs.grid.latitude.start, lat_grid_start);
    assert_eq!(specs.grid.latitude.end, lat_grid_end);
    assert_eq!(specs.grid.latitude.spacing, lat_grid_spacing);

    assert_eq!(specs.grid.longitude.start, long_grid_start);
    assert_eq!(specs.grid.longitude.end, long_grid_end);
    assert_eq!(specs.grid.longitude.spacing, long_grid_spacing);

    if let Some(h_grid_start) = h_grid_start {
        assert_eq!(specs.grid.height.start, h_grid_start);
    }
    if let Some(h_grid_end) = h_grid_end {
        assert_eq!(specs.grid.height.end, h_grid_end);
    }
    if let Some(h_grid_spacing) = h_grid_spacing {
        assert_eq!(specs.grid.height.spacing, h_grid_spacing);
    }

    assert_eq!(specs.exponent, exponent);
    assert_eq!(specs.base_radius, base_radius);
    assert_eq!(specs.elevation_cutoff, elevation_cutoff);

    if let Some(mapping_func) = mapping_function {
        let mapping_func = MappingFunction::from_str(mapping_func).unwrap();
        assert_eq!(specs.mapping, Some(mapping_func));
    } else {
        assert!(specs.mapping.is_none());
    }
}

/// Generic test that we can use for Observation [Rinex]
pub fn generic_ionex_test(
    dut: &Rinex,
    model: Option<&Rinex>,
    version: &str,
    is_3d: bool,
    epoch_of_first_map: &str,
    epoch_of_last_map: &str,
    lat_grid_start: f64,
    lat_grid_end: f64,
    lat_grid_spacing: f64,
    long_grid_start: f64,
    long_grid_end: f64,
    long_grid_spacing: f64,
    h_grid_start: Option<f64>,
    h_grid_end: Option<f64>,
    h_grid_spacing: Option<f64>,
    header_exponent: i8,
    base_radius: f32,
    elevation_cutoff: f32,
    mapping_function: Option<&str>,
    time_frame: TimeFrame,
) {
    assert!(dut.is_ionex());

    if is_3d {
        assert!(dut.is_ionex_3d());
    } else {
        assert!(dut.is_ionex_2d());
    }

    let dut_rec = dut.record.as_ionex().unwrap();

    generic_rinex_test(
        dut,
        version,
        None,
        RinexType::IonosphereMaps,
        Some(time_frame),
    );

    basic_header_tests(
        &dut.header,
        epoch_of_first_map,
        epoch_of_last_map,
        lat_grid_start,
        lat_grid_end,
        lat_grid_spacing,
        long_grid_start,
        long_grid_end,
        long_grid_spacing,
        h_grid_start,
        h_grid_end,
        h_grid_spacing,
        header_exponent,
        base_radius,
        elevation_cutoff,
        mapping_function,
    );

    // Test TEC in volume
    // for tec in tec_points {
    //     let k = point.key;
    //     let values = dut_rec
    //         .get(&k)
    //         .expect(&format!("missing data point for {:?}", k));

    //     let mut passed = false;

    //     for signal in values.signals.iter() {
    //         if signal.sv == point.signal.sv {
    //             if signal.observable == point.signal.observable {
    //                 assert_eq!(signal.value, point.signal.value);
    //                 assert_eq!(signal.lli, point.signal.lli);
    //                 //assert_eq!(signal.snr, point.signal.snr); //TODO unlock
    //                 passed = true;
    //             }
    //         }
    //     }
    //     assert!(passed, "missing data point {:?}", point);
    // }
}
