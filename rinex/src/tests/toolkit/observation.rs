use std::str::FromStr;

use crate::{
    observation::{parse_epoch, HeaderFields},
    prelude::{
        ClockObservation, Constellation, Epoch, EpochFlag, GeodeticMarker, GroundPosition, Header,
        ObsKey, Observable, Observations, Rinex, RinexType, SignalObservation, Version, SV,
    },
    tests::toolkit::{
        generic_null_rinex_test, generic_rinex_test, gnss_csv as gnss_from_csv,
        observables_csv as observable_from_csv, sv_csv as sv_from_csv, TimeFrame,
    },
};

use itertools::Itertools;

pub struct ClockDataPoint {
    pub key: ObsKey,
    pub clock: ClockObservation,
}

impl ClockDataPoint {
    pub fn new(epoch: Epoch, flag: EpochFlag, offset_s: f64) -> Self {
        Self {
            key: ObsKey { epoch, flag },
            clock: ClockObservation::default().with_offset_s(epoch, offset_s),
        }
    }
}

pub struct SignalDataPoint {
    pub key: ObsKey,
    pub signal: SignalObservation,
}

impl SignalDataPoint {
    pub fn new(epoch: Epoch, flag: EpochFlag, sv: SV, observable: Observable, value: f64) -> Self {
        let mut signal = SignalObservation {
            sv,
            observable,
            value,
            lli: None,
            snr: None,
        };
        Self {
            key: ObsKey { epoch, flag },
            signal,
        }
    }
}

/// Basic tests for Observation [Rinex]
fn basic_header_tests(dut: &Header, timeof_first_obs: Option<&str>, timeof_last_obs: Option<&str>) {
    assert!(dut.obs.is_some(),);
    assert!(dut.meteo.is_none(),);
    assert!(dut.ionex.is_none(),);
    assert!(dut.clock.is_none(),);

    let specs = dut.obs.as_ref().expect("missing specific specs");

    if let Some(t) = timeof_first_obs {
        let t = Epoch::from_str(t).unwrap();
        assert_eq!(specs.timeof_first_obs, Some(t));
    }

    if let Some(t) = timeof_last_obs {
        let t = Epoch::from_str(t).unwrap();
        assert_eq!(specs.timeof_last_obs, Some(t));
    }
}

/// Generic test that we can use for Observation [Rinex]
pub fn generic_observation_rinex_test(
    dut: &Rinex,
    model: Option<&Rinex>,
    version: &str,
    header_constellation: Option<&str>,
    has_clock: bool,
    sv_csv: &str,
    gnss_csv: &str,
    gnss_observ_csv: &[(&str, &str)],
    timeof_first_obs: Option<&str>,
    timeof_last_obs: Option<&str>,
    ground_ref_wgs84_m: Option<(f64, f64, f64)>,
    observer: Option<&str>,
    geodetic_marker: Option<GeodeticMarker>,
    time_frame: TimeFrame,
    signal_points: Vec<SignalDataPoint>,
    clock_points: Vec<ClockDataPoint>,
) {
    assert!(dut.is_observation_rinex());

    let dut_rec = dut.record.as_obs().unwrap();

    generic_rinex_test(
        dut,
        version,
        header_constellation,
        RinexType::ObservationData,
        Some(time_frame),
    );

    basic_header_tests(&dut.header, timeof_first_obs, timeof_last_obs);

    if let Some((x_m, y_m, z_m)) = ground_ref_wgs84_m {
        assert_eq!(
            dut.header.ground_position,
            Some(GroundPosition::from_ecef_wgs84((x_m, y_m, z_m)))
        );
    }

    if let Some(observer) = observer {
        assert_eq!(dut.header.observer, observer);
    }

    if let Some(marker) = geodetic_marker {
        assert_eq!(dut.header.geodetic_marker, Some(marker));
    }

    // verifies header specs
    let specs = dut.header.obs.as_ref().unwrap();
    for (gnss, observable_csv) in gnss_observ_csv {
        let gnss = Constellation::from_str(gnss).unwrap();
        let expected = observable_from_csv(observable_csv);
        let found = specs
            .codes
            .get(&gnss)
            .expect(&format!("missing header specs for {}", gnss))
            .into_iter()
            .cloned()
            .sorted()
            .collect::<Vec<_>>();
        assert_eq!(found, expected);
    }

    let clocks = dut.clock_observations_iter().collect::<Vec<_>>();
    if has_clock {
        assert!(clocks.len() > 0, "missing clock data");
    } else {
        assert!(clocks.len() == 0, "found invalid clock data");
    }

    // Check SV content
    let content = dut.sv().collect::<Vec<_>>();
    let expected = sv_from_csv(sv_csv);
    assert_eq!(content, expected);

    // Check GNSS content
    let content = dut.constellation().sorted().collect::<Vec<_>>();
    let expected = gnss_from_csv(gnss_csv);
    assert_eq!(content, expected);

    // Self - Self should be 0
    let null_dut = dut.substract(&dut);
    generic_null_rinex_test(&null_dut);

    // Check against provided model
    if let Some(model) = model {
        generic_observation_rinex_against_model(dut, model);
    }

    // Test signal data points
    for point in signal_points {
        let k = point.key;
        let values = dut_rec.get(&k).unwrap();
        assert!(values.signals.contains(&point.signal));
    }

    // Test clock data points
    for point in clock_points {
        let k = point.key;
        let values = dut_rec.get(&k).unwrap();
        assert_eq!(values.clock, Some(point.clock));
    }
}

/// [Rinex] against [Rinex] model verification
pub fn generic_observation_rinex_against_model(dut: &Rinex, model: &Rinex) {
    let rec_dut = dut.record.as_obs().expect("failed to unwrap rinex record");

    let rec_model = model
        .record
        .as_obs()
        .expect("failed to unwrap rinex record");

    // verify constellations
    let dut_content = dut.constellation().sorted().collect::<Vec<_>>();
    let expected_content = model.constellation().sorted().collect::<Vec<_>>();
    assert_eq!(dut_content, expected_content);

    // verify observables
    let dut_content = dut.observable().sorted().collect::<Vec<_>>();
    let expected_content = dut.observable().sorted().collect::<Vec<_>>();
    assert_eq!(dut_content, expected_content);

    // TODO : verify carriers

    for (k, _) in rec_dut.iter() {
        assert!(
            rec_model.get(k).is_some(),
            "found unexpected content: {:?}",
            k
        );
    }

    for (k, v) in rec_model.iter() {
        let dut = rec_dut.get(k).expect(&format!("missing content {:?}", k));

        // clock comparison
        assert_eq!(v.clock, dut.clock);

        // signal comparison
        assert_eq!(v.signals, dut.signals);
    }
}

/// Generic method to use at the epoch decoding level (testing)
pub fn generic_observation_epoch_decoding_test(
    content: &str,
    major: u8,
    header_constell: Constellation,
    header_gnss_obs_csv: &[(&str, &str)],
    timeof_first_obs: &str,
    num_signals: usize,
    key_epoch: &str,
    key_flag: EpochFlag,
    clock: Option<ClockObservation>,
) {
    // build [Header]
    let t0 = Epoch::from_str(timeof_first_obs).unwrap();
    let key_epoch = Epoch::from_str(key_epoch).unwrap();

    let ts = t0.time_scale;

    let mut specs = HeaderFields::default().with_timeof_first_obs(t0);

    for (constell, observable_csv) in header_gnss_obs_csv.iter() {
        let constell = Constellation::from_str(constell).unwrap();
        let observables = observable_from_csv(observable_csv);
        specs.codes.insert(constell, observables);
    }

    let header = Header::default()
        .with_version(Version { major, minor: 0 })
        .with_constellation(header_constell)
        .with_observation_fields(specs);

    // PARSE
    let mut obs = Observations::default();

    let key = parse_epoch(&header, content, ts, &mut obs).unwrap();

    assert_eq!(key.epoch, key_epoch);
    assert_eq!(key.flag, key_flag);
    assert_eq!(obs.clock, clock);
    assert_eq!(obs.signals.len(), num_signals);

    // TODO: test data points
}
