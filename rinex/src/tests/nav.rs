#[cfg(test)]
mod test {
    use crate::carrier::Carrier;
    use crate::navigation::*;
    use crate::prelude::*;
    use gnss_rs::prelude::SV;
    use gnss_rs::sv;
    use itertools::*;
    use std::path::Path;
    use std::path::PathBuf;
    use std::str::FromStr;
    #[test]
    #[cfg(feature = "nav")]
    fn v2_amel0010_21g() {
        let test_resource =
            env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/NAV/V2/amel0010.21g";
        let rinex = Rinex::from_file(&test_resource);
        assert!(rinex.is_ok());
        let rinex = rinex.unwrap();
        let record = rinex.record.as_nav();
        assert!(record.is_some());
        let record = record.unwrap();
        assert_eq!(record.len(), 4);

        // Test: parsed correct amount of entries
        assert_eq!(rinex.navigation().count(), 4);

        // Test: only Ephemeris in this record
        assert_eq!(rinex.ephemeris().count(), 6);
        // Test: only Legacy Ephemeris frames in this record
        for (_, (msg, _, _)) in rinex.ephemeris() {
            assert_eq!(msg, NavMsgType::LNAV);
        }

        let epochs = vec![
            Epoch::from_gregorian_utc(2020, 12, 31, 23, 45, 0, 0),
            Epoch::from_gregorian_utc(2021, 01, 01, 11, 15, 0, 0),
            Epoch::from_gregorian_utc(2021, 01, 01, 11, 45, 0, 0),
            Epoch::from_gregorian_utc(2021, 01, 01, 16, 15, 0, 0),
        ];
        assert!(rinex.epoch().eq(epochs), "parsed wrong epoch content");

        let prn: Vec<u8> = vec![1, 2, 7, 3, 4, 5];
        let mut vehicles: Vec<SV> = prn
            .iter()
            .map(|prn| SV {
                constellation: Constellation::Glonass,
                prn: *prn,
            })
            .collect();
        vehicles.sort(); // for comparison purposes
        assert!(
            rinex.sv().sorted().eq(vehicles),
            "parsed wrong vehicle content",
        );

        for (_e, frames) in rinex.navigation() {
            for fr in frames {
                // test : only Ephemeris frames in old rinex
                let fr = fr.as_eph();
                assert!(fr.is_some(), "parsed non ephemeris frame unexpectedly");

                // test : only Legacy frames in old rinex
                let (msg, sv, ephemeris) = fr.unwrap();
                assert!(
                    msg == NavMsgType::LNAV,
                    "only LNAV frames are expected here"
                );

                // test inner data
                if sv.prn == 1 {
                    assert_eq!(
                        ephemeris.sv_clock(),
                        (7.282570004460E-5, 0.0, 7.380000000000E+04)
                    );
                    let data = &ephemeris.orbits;
                    //assert_eq!(
                    //    ephemeris.sv_position(*e),
                    //    Some((-1.488799804690E+03, 1.292880712890E+04, 2.193169775390E+04))
                    //);
                    //TODO test health completly
                    //let health = data.get("health").unwrap();
                    //assert_eq!(health.as_f64(), Some(0.0));
                    let freq = data.get("channel").unwrap();
                    assert_eq!(freq.as_i8(), Some(1));
                    let ageop = data.get("ageOp").unwrap();
                    assert_eq!(ageop.as_f64(), Some(0.0));
                } else if sv.prn == 2 {
                    assert_eq!(ephemeris.clock_bias, 4.610531032090E-04);
                    assert_eq!(ephemeris.clock_drift, 1.818989403550E-12);
                    assert_eq!(ephemeris.clock_drift_rate, 4.245000000000E+04);
                    let data = &ephemeris.orbits;
                    let posx = data.get("satPosX").unwrap();
                    assert_eq!(posx.as_f64(), Some(-8.955041992190E+03));
                    let posy = data.get("satPosY").unwrap();
                    assert_eq!(posy.as_f64(), Some(-1.834875292970E+04));
                    let posz = data.get("satPosZ").unwrap();
                    assert_eq!(posz.as_f64(), Some(1.536620703130E+04));
                    let freq = data.get("channel").unwrap();
                    assert_eq!(freq.as_i8(), Some(-4));
                    let ageop = data.get("ageOp").unwrap();
                    assert_eq!(ageop.as_f64(), Some(0.0));
                } else if sv.prn == 3 {
                    assert_eq!(ephemeris.clock_bias, 2.838205546140E-05);
                    assert_eq!(ephemeris.clock_drift, 0.0);
                    assert_eq!(ephemeris.clock_drift_rate, 4.680000000000E+04);
                    let data = &ephemeris.orbits;
                    let posx = data.get("satPosX").unwrap();
                    assert_eq!(posx.as_f64(), Some(1.502522949220E+04));
                    let posy = data.get("satPosY").unwrap();
                    assert_eq!(posy.as_f64(), Some(-1.458877050780E+04));
                    let posz = data.get("satPosZ").unwrap();
                    assert_eq!(posz.as_f64(), Some(1.455863281250E+04));
                    //TODO test health completly
                    //let health = data.get("health").unwrap();
                    //assert_eq!(health.as_f64(), Some(0.0));
                    let freq = data.get("channel").unwrap();
                    assert_eq!(freq.as_i8(), Some(5));
                    let ageop = data.get("ageOp").unwrap();
                    assert_eq!(ageop.as_f64(), Some(0.0));
                } else if sv.prn == 4 {
                    assert_eq!(ephemeris.clock_bias, 6.817653775220E-05);
                    assert_eq!(ephemeris.clock_drift, 1.818989403550E-12);
                    assert_eq!(ephemeris.clock_drift_rate, 4.680000000000E+04);
                    let data = &ephemeris.orbits;
                    let posx = data.get("satPosX").unwrap();
                    assert_eq!(posx.as_f64(), Some(-1.688173828130E+03));
                    let posy = data.get("satPosY").unwrap();
                    assert_eq!(posy.as_f64(), Some(-1.107156738280E+04));
                    let posz = data.get("satPosZ").unwrap();
                    assert_eq!(posz.as_f64(), Some(2.293745361330E+04));
                    //TODO
                    //let health = data.get("health").unwrap();
                    //assert_eq!(health.as_f64(), Some(0.0));
                    let freq = data.get("channel").unwrap();
                    assert_eq!(freq.as_i8(), Some(6));
                    let ageop = data.get("ageOp").unwrap();
                    assert_eq!(ageop.as_f64(), Some(0.0));
                } else if sv.prn == 5 {
                    assert_eq!(ephemeris.clock_bias, 6.396882236000E-05);
                    assert_eq!(ephemeris.clock_drift, 9.094947017730E-13);
                    assert_eq!(ephemeris.clock_drift_rate, 8.007000000000E+04);
                    let data = &ephemeris.orbits;
                    let posx = data.get("satPosX").unwrap();
                    assert_eq!(posx.as_f64(), Some(-1.754308935550E+04));
                    let posy = data.get("satPosY").unwrap();
                    assert_eq!(posy.as_f64(), Some(-1.481773437500E+03));
                    let posz = data.get("satPosZ").unwrap();
                    assert_eq!(posz.as_f64(), Some(1.847386083980E+04));
                    //TODO
                    //let health = data.get("health").unwrap();
                    //assert_eq!(health.as_f64(), Some(0.0));
                    let freq = data.get("channel").unwrap();
                    assert_eq!(freq.as_i8(), Some(1));
                    let ageop = data.get("ageOp").unwrap();
                    assert_eq!(ageop.as_f64(), Some(0.0));
                } else if sv.prn == 7 {
                    assert_eq!(ephemeris.clock_bias, -4.201009869580E-05);
                    assert_eq!(ephemeris.clock_drift, 0.0);
                    assert_eq!(ephemeris.clock_drift_rate, 2.88E4);
                    let data = &ephemeris.orbits;
                    let posx = data.get("satPosX").unwrap();
                    assert_eq!(posx.as_f64(), Some(1.817068505860E+04));
                    let posy = data.get("satPosY").unwrap();
                    assert_eq!(posy.as_f64(), Some(1.594814404300E+04));
                    let posz = data.get("satPosZ").unwrap();
                    assert_eq!(posz.as_f64(), Some(8.090271484380E+03));
                    //TODO
                    //let health = data.get("health").unwrap();
                    //assert_eq!(health.as_f64(), Some(0.0));
                    let freq = data.get("channel").unwrap();
                    assert_eq!(freq.as_i8(), Some(5));
                    let ageop = data.get("ageOp").unwrap();
                    assert_eq!(ageop.as_f64(), Some(0.0));
                }
            }
        }
    }
    #[test]
    #[cfg(feature = "flate2")]
    fn v2_cbw10010_21n() {
        let test_resources =
            env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/NAV/V2/cbw10010.21n.gz";
        let rinex = Rinex::from_file(&test_resources);
        assert!(rinex.is_ok(), "failed to parse NAV/V2/cbw10010.21n.gz");
        let rinex = rinex.unwrap();

        // test inner data
        for (e, frames) in rinex.navigation() {
            for fr in frames {
                // test : only Ephemeris frames in old rinex
                let fr = fr.as_eph();
                assert!(fr.is_some(), "parsed non ephemeris frame unexpectedly");

                // test : only Legacy frames in old rinex
                let (msg, sv, ephemeris) = fr.unwrap();
                assert!(
                    msg == NavMsgType::LNAV,
                    "only LNAV frames are expected here"
                );

                if *e == Epoch::from_str("2020-12-31T23:59:44").unwrap() {
                    if sv.prn == 7 {
                        assert_eq!(
                            ephemeris.sv_clock(),
                            (4.204921424390E-6, 1.477928890380E-11, 0.0),
                            "parsed wrong clock data"
                        );

                        for (field, data) in vec![
                            ("iode", Some(0.0_f64)),
                            ("crs", Some(-1.509375000000E1)),
                            ("deltaN", Some(5.043781392540E-9)),
                            ("m0", Some(-1.673144695710)),
                            ("cuc", Some(-8.475035429000E-7)),
                            ("e", Some(1.431132073050E-2)),
                            ("cus", Some(5.507841706280E-6)),
                            ("sqrta", Some(5.153606595990E3)),
                            ("toe", Some(4.319840000000E5)),
                            ("cic", Some(2.216547727580E-7)),
                            ("omega0", Some(2.333424778860)),
                            ("cis", Some(-8.009374141690E-8)),
                            ("i0", Some(9.519533967710E-1)),
                            ("crc", Some(2.626562500000E2)),
                            ("omega", Some(-2.356931900380)),
                            ("omegaDot", Some(-8.034263032640E-9)),
                            ("idot", Some(-1.592923432050E-10)),
                            ("l2Codes", Some(1.000000000000)),
                            ("l2pDataFlag", Some(0.000000000000)),
                            ("svAccuracy", Some(0.000000000000)),
                            ("tgd", Some(-1.117587089540E-8)),
                            ("iodc", Some(0.000000000000)),
                            ("t_tm", Some(4.283760000000E5)),
                        ] {
                            let value = ephemeris.get_orbit_f64(field);
                            assert!(value.is_some(), "missing orbit filed \"{}\"", field);
                            assert_eq!(
                                value, data,
                                "parsed wrong \"{}\" value, expecting {:?} got {:?}",
                                field, data, value
                            );
                        }
                        assert!(
                            ephemeris.get_orbit_f64("fitInt").is_none(),
                            "parsed fitInt unexpectedly"
                        );

                        assert_eq!(ephemeris.get_week(), Some(2138));
                    }
                } else if *e == Epoch::from_str("2021-01-02T00:00:00").unwrap() && sv.prn == 30 {
                    assert_eq!(
                        ephemeris.sv_clock(),
                        (-3.621461801230E-04, -6.139089236970E-12, 0.000000000000),
                        "parsed wrong clock data"
                    );

                    for (field, data) in vec![
                        ("iode", Some(8.500000000000E1)),
                        ("crs", Some(-7.500000000000)),
                        ("deltaN", Some(5.476656696160E-9)),
                        ("m0", Some(-1.649762378650)),
                        ("cuc", Some(-6.072223186490E-7)),
                        ("e", Some(4.747916595080E-3)),
                        ("cus", Some(5.392357707020E-6)),
                        ("sqrta", Some(5.153756387710E+3)),
                        ("toe", Some(5.184000000000E+5)),
                        ("cic", Some(7.636845111850E-8)),
                        ("omega0", Some(2.352085289360E+00)),
                        ("cis", Some(-2.421438694000E-8)),
                        ("i0", Some(9.371909002540E-1)),
                        ("crc", Some(2.614687500000E+2)),
                        ("omega", Some(-2.846234079630)),
                        ("omegaDot", Some(-8.435351366240E-9)),
                        ("idot", Some(-7.000291590240E-11)),
                        ("l2Codes", Some(1.000000000000)),
                        ("l2pDataFlag", Some(0.0)),
                        ("svAccuracy", Some(0.0)),
                        ("tgd", Some(3.725290298460E-9)),
                        ("iodc", Some(8.500000000000E1)),
                        ("t_tm", Some(5.146680000000E5)),
                    ] {
                        let value = ephemeris.get_orbit_f64(field);
                        assert!(value.is_some(), "missing orbit filed \"{}\"", field);
                        assert_eq!(
                            value, data,
                            "parsed wrong \"{}\" value, expecting {:?} got {:?}",
                            field, data, value
                        );
                    }
                    assert!(
                        ephemeris.get_orbit_f64("fitInt").is_none(),
                        "parsed fitInt unexpectedly"
                    );

                    assert_eq!(ephemeris.get_week(), Some(2138));
                }
            }
        }
    }
    #[test]
    fn v3_amel00nld_r_2021() {
        let test_resource = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let rinex = Rinex::from_file(&test_resource);
        assert!(rinex.is_ok());

        let rinex = rinex.unwrap();
        assert!(rinex.is_navigation_rinex());
        assert!(rinex.header.obs.is_none());
        assert!(rinex.header.meteo.is_none());

        let record = rinex.record.as_nav();
        assert!(record.is_some());

        let record = record.unwrap();
        assert_eq!(record.len(), 6);

        // Test: only Ephemeris in this record
        let ephemeris: Vec<_> = rinex.ephemeris().collect();
        assert_eq!(ephemeris.len(), 6);

        let epochs = vec![
            Epoch::from_str("2021-01-01T00:00:00 BDT").unwrap(),
            Epoch::from_str("2021-01-01T00:15:00 UTC").unwrap(),
            Epoch::from_str("2021-01-01T05:00:00 BDT").unwrap(),
            Epoch::from_str("2021-01-01T09:45:00 UTC").unwrap(),
            Epoch::from_str("2021-01-01T10:10:00 GST").unwrap(),
            Epoch::from_str("2021-01-01T15:40:00 GST").unwrap(),
        ];

        assert!(
            rinex.epoch().eq(epochs.clone()),
            "Parsed wrong epoch content.\nExpecting {:?}\nGot {:?}",
            epochs.clone(),
            rinex.epoch().collect::<Vec<Epoch>>(),
        );

        let mut vehicles = vec![
            sv!("c05"),
            sv!("c21"),
            sv!("e01"),
            sv!("e03"),
            sv!("r07"),
            sv!("r19"),
        ];
        vehicles.sort(); // for comparison
        assert!(rinex.sv().sorted().eq(vehicles), "parsed wrong sv content");

        for (_e, frames) in record.iter() {
            for fr in frames {
                // test: only Ephemeris frames in V3
                let fr = fr.as_eph();
                assert!(fr.is_some(), "expecting only ephemeris frames here");

                let (msg, sv, ephemeris) = fr.unwrap();

                // test: only Legacy frames in V3
                assert!(msg == NavMsgType::LNAV, "only legacy frames expected here");

                // test some data
                match sv.constellation {
                    Constellation::BeiDou => match sv.prn {
                        5 => {
                            assert_eq!(ephemeris.clock_bias, -0.426337239332e-03);
                            assert_eq!(ephemeris.clock_drift, -0.752518047875e-10);
                            assert_eq!(ephemeris.clock_drift_rate, 0.0);
                            let data = &ephemeris.orbits;
                            let aode = data.get("aode").unwrap();
                            assert_eq!(aode.as_f64(), Some(0.100000000000e+01));
                            let crs = data.get("crs").unwrap();
                            assert_eq!(crs.as_f64(), Some(0.118906250000e+02));
                            let m0 = data.get("m0").unwrap();
                            assert_eq!(m0.as_f64(), Some(-0.255139531119e+01));
                            let i0 = data.get("i0").unwrap();
                            assert_eq!(i0.as_f64(), Some(0.607169709798e-01));
                            let acc = data.get("svAccuracy").unwrap();
                            assert_eq!(acc.as_f64(), Some(0.200000000000e+01));
                            let sath1 = data.get("satH1").unwrap();
                            assert_eq!(sath1.as_f64(), Some(0.0));
                            let tgd1 = data.get("tgd1b1b3").unwrap();
                            assert_eq!(tgd1.as_f64(), Some(-0.599999994133e-09));
                        },
                        21 => {
                            assert_eq!(ephemeris.clock_bias, -0.775156309828e-03);
                            assert_eq!(ephemeris.clock_drift, -0.144968481663e-10);
                            assert_eq!(ephemeris.clock_drift_rate, 0.000000000000e+0);
                            let data = &ephemeris.orbits;
                            let aode = data.get("aode").unwrap();
                            assert_eq!(aode.as_f64(), Some(0.100000000000e+01));
                            let crs = data.get("crs").unwrap();
                            assert_eq!(crs.as_f64(), Some(-0.793437500000e+02));
                            let m0 = data.get("m0").unwrap();
                            assert_eq!(m0.as_f64(), Some(0.206213212749e+01));
                            let i0 = data.get("i0").unwrap();
                            assert_eq!(i0.as_f64(), Some(0.964491154768e+00));
                            let acc = data.get("svAccuracy").unwrap();
                            assert_eq!(acc.as_f64(), Some(0.200000000000e+01));
                            let sath1 = data.get("satH1").unwrap();
                            assert_eq!(sath1.as_f64(), Some(0.0));
                            let tgd1 = data.get("tgd1b1b3").unwrap();
                            assert_eq!(tgd1.as_f64(), Some(0.143000002950e-07));
                        },
                        _ => panic!("identified unexpected BDS vehicle \"{}\"", sv.prn),
                    },
                    Constellation::Glonass => match sv.prn {
                        19 => {
                            assert_eq!(ephemeris.clock_bias, -0.126023776829e-03);
                            assert_eq!(ephemeris.clock_drift, -0.909494701773e-12);
                            assert_eq!(ephemeris.clock_drift_rate, 0.0);
                            let data = &ephemeris.orbits;
                            let pos = data.get("satPosX").unwrap();
                            assert_eq!(pos.as_f64(), Some(0.783916601562e+04));
                            let pos = data.get("satPosY").unwrap();
                            assert_eq!(pos.as_f64(), Some(-0.216949155273e+05));
                            let pos = data.get("satPosZ").unwrap();
                            assert_eq!(pos.as_f64(), Some(0.109021518555e+05));
                        },
                        7 => {
                            assert_eq!(ephemeris.clock_bias, -0.420100986958E-04);
                            assert_eq!(ephemeris.clock_drift, 0.0);
                            assert_eq!(ephemeris.clock_drift_rate, 0.342000000000e+05);
                            let data = &ephemeris.orbits;
                            let pos = data.get("satPosX").unwrap();
                            assert_eq!(pos.as_f64(), Some(0.124900639648e+05));
                            let pos = data.get("satPosY").unwrap();
                            assert_eq!(pos.as_f64(), Some(0.595546582031e+04));
                            let pos = data.get("satPosZ").unwrap();
                            assert_eq!(pos.as_f64(), Some(0.214479208984e+05));
                        },
                        _ => panic!("identified unexpected GLO vehicle \"{}\"", sv.prn),
                    },
                    Constellation::Galileo => match sv.prn {
                        1 => {
                            assert_eq!(ephemeris.clock_bias, -0.101553811692e-02);
                            assert_eq!(ephemeris.clock_drift, -0.804334376880e-11);
                            assert_eq!(ephemeris.clock_drift_rate, 0.0);
                            let data = &ephemeris.orbits;
                            let iodnav = data.get("iodnav").unwrap();
                            assert_eq!(iodnav.as_f64(), Some(0.130000000000e+02));
                            let crs = data.get("crs").unwrap();
                            assert_eq!(crs.as_f64(), Some(0.435937500000e+02));
                            let cis = data.get("cis").unwrap();
                            assert_eq!(cis.as_f64(), Some(0.409781932831e-07));
                            let omega_dot = data.get("omegaDot").unwrap();
                            assert_eq!(omega_dot.as_f64(), Some(-0.518200156545e-08));
                            let idot = data.get("idot").unwrap();
                            assert_eq!(idot.as_f64(), Some(-0.595381942905e-09));
                            let sisa = data.get("sisa").unwrap();
                            assert_eq!(sisa.as_f64(), Some(0.312000000000e+01));
                            let bgd = data.get("bgdE5aE1").unwrap();
                            assert_eq!(bgd.as_f64(), Some(0.232830643654e-09));
                        },
                        3 => {
                            assert_eq!(ephemeris.clock_bias, -0.382520200219e-03);
                            assert_eq!(ephemeris.clock_drift, -0.422062385041e-11);
                            assert_eq!(ephemeris.clock_drift_rate, 0.0);
                            let data = &ephemeris.orbits;
                            let iodnav = data.get("iodnav").unwrap();
                            assert_eq!(iodnav.as_f64(), Some(0.460000000000e+02));
                            let crs = data.get("crs").unwrap();
                            assert_eq!(crs.as_f64(), Some(-0.103750000000e+02));
                            let cis = data.get("cis").unwrap();
                            assert_eq!(cis.as_f64(), Some(0.745058059692e-08));
                            let omega_dot = data.get("omegaDot").unwrap();
                            assert_eq!(omega_dot.as_f64(), Some(-0.539986778331e-08));
                            let idot = data.get("idot").unwrap();
                            assert_eq!(idot.as_f64(), Some(0.701814947695e-09));
                            let sisa = data.get("sisa").unwrap();
                            assert_eq!(sisa.as_f64(), Some(0.312000000000e+01));
                            let bgd = data.get("bgdE5aE1").unwrap();
                            assert_eq!(bgd.as_f64(), Some(0.302679836750e-08));
                        },
                        _ => panic!("identified unexpected GAL vehicle \"{}\"", sv.prn),
                    },
                    _ => panic!("falsely identified \"{}\"", sv),
                }
            } //match sv.constellation
        }
    }
    #[test]
    #[cfg(feature = "flate2")]
    fn v4_kms300dnk_r_202215910() {
        let test_resource = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz";
        let rinex = Rinex::from_file(&test_resource);
        assert!(rinex.is_ok());
        let rinex = rinex.unwrap();
        assert!(rinex.is_navigation_rinex());
        assert!(rinex.header.obs.is_none());
        assert!(rinex.header.meteo.is_none());

        let record = rinex.record.as_nav();
        assert!(record.is_some());
        let record = record.unwrap();

        // test first epoch
        assert_eq!(
            rinex.first_epoch(),
            Some(Epoch::from_str("2022-06-07T23:59:44 GPST").unwrap()),
            "wrong first epoch",
        );

        // test last epoch
        assert_eq!(
            rinex.last_epoch(),
            Some(Epoch::from_str("2022-06-10T19:56:48 GPST").unwrap()),
            "wrong last epoch",
        );

        let mut vehicles: Vec<_> = vec![
            sv!("G02"),
            sv!("G04"),
            sv!("G05"),
            sv!("G09"),
            sv!("G11"),
            sv!("G12"),
            sv!("G16"),
            sv!("G18"),
            sv!("G20"),
            sv!("G22"),
            sv!("G23"),
            sv!("G25"),
            sv!("G26"),
            sv!("G27"),
            sv!("G29"),
            sv!("G31"),
            sv!("G07"),
            sv!("G10"),
            sv!("G15"),
            sv!("G13"),
            sv!("G08"),
            sv!("R03"),
            sv!("R04"),
            sv!("R05"),
            sv!("R10"),
            sv!("R11"),
            sv!("R12"),
            sv!("R20"),
            sv!("R21"),
            sv!("R21"),
            sv!("R11"),
            sv!("R12"),
            sv!("R20"),
            sv!("R04"),
            sv!("R05"),
            sv!("R10"),
            sv!("R13"),
            sv!("R21"),
            sv!("R12"),
            sv!("R11"),
            sv!("R05"),
            sv!("R13"),
            sv!("R20"),
            sv!("R04"),
            sv!("R23"),
            sv!("E01"),
            sv!("E03"),
            sv!("E05"),
            sv!("E07"),
            sv!("E08"),
            sv!("E09"),
            sv!("E10"),
            sv!("E12"),
            sv!("E13"),
            sv!("E14"),
            sv!("E15"),
            sv!("E21"),
            sv!("E24"),
            sv!("E25"),
            sv!("E26"),
            sv!("E31"),
            sv!("E33"),
            sv!("E01"),
            sv!("E03"),
            sv!("E05"),
            sv!("E07"),
            sv!("E08"),
            sv!("E09"),
            sv!("E10"),
            sv!("E12"),
            sv!("E13"),
            sv!("E14"),
            sv!("E15"),
            sv!("E21"),
            sv!("E24"),
            sv!("E25"),
            sv!("E26"),
            sv!("E31"),
            sv!("E33"),
            sv!("E07"),
            sv!("E07"),
            sv!("E26"),
            sv!("E14"),
            sv!("E26"),
            sv!("E25"),
            sv!("E08"),
            sv!("E01"),
            sv!("E10"),
            sv!("E10"),
            sv!("E11"),
            sv!("E14"),
            sv!("E25"),
            sv!("E26"),
            sv!("E08"),
            sv!("E10"),
            sv!("E14"),
            sv!("E10"),
            sv!("E25"),
            sv!("E26"),
            sv!("E08"),
            sv!("E14"),
            sv!("E26"),
            sv!("E08"),
            sv!("E25"),
            sv!("E10"),
            sv!("E14"),
            sv!("E10"),
            sv!("E25"),
            sv!("E26"),
            sv!("E08"),
            sv!("E25"),
            sv!("E14"),
            sv!("E26"),
            sv!("E08"),
            sv!("E10"),
            sv!("E14"),
            sv!("E10"),
            sv!("E25"),
            sv!("E26"),
            sv!("E08"),
            sv!("E33"),
            sv!("E11"),
            sv!("S48"),
            sv!("S36"),
            sv!("S26"),
            sv!("S44"),
            sv!("S23"),
            sv!("S25"),
            sv!("S27"),
            sv!("S26"),
            sv!("S48"),
            sv!("S27"),
            sv!("S36"),
            sv!("S44"),
            sv!("S23"),
            sv!("S48"),
            sv!("S26"),
            sv!("S36"),
            sv!("S44"),
            sv!("S23"),
            sv!("S48"),
            sv!("S27"),
            sv!("S44"),
            sv!("S36"),
            sv!("S23"),
            sv!("S26"),
            sv!("S48"),
            sv!("S44"),
            sv!("S36"),
            sv!("S23"),
            sv!("S48"),
            sv!("S26"),
            sv!("S44"),
            sv!("S27"),
            sv!("S36"),
            sv!("S48"),
            sv!("S23"),
            sv!("S44"),
            sv!("S26"),
            sv!("S36"),
            sv!("S48"),
            sv!("S23"),
            sv!("S44"),
            sv!("S27"),
            sv!("S48"),
            sv!("S36"),
            sv!("S26"),
            sv!("S23"),
            sv!("S44"),
            sv!("S48"),
            sv!("S36"),
            sv!("S23"),
            sv!("S26"),
            sv!("S44"),
            sv!("S48"),
            sv!("S27"),
            sv!("S36"),
            sv!("S23"),
            sv!("S44"),
            sv!("S48"),
            sv!("S26"),
            sv!("S36"),
            sv!("S23"),
            sv!("S44"),
            sv!("S48"),
            sv!("S27"),
            sv!("S36"),
            sv!("S26"),
            sv!("S23"),
            sv!("S44"),
            sv!("S48"),
            sv!("S36"),
            sv!("S23"),
            sv!("S48"),
            sv!("S44"),
            sv!("S26"),
            sv!("S27"),
            sv!("S36"),
            sv!("S23"),
            sv!("S48"),
            sv!("S44"),
            sv!("S26"),
            sv!("S36"),
            sv!("S48"),
            sv!("S44"),
            sv!("S23"),
            sv!("S36"),
            sv!("S48"),
            sv!("S26"),
            sv!("S44"),
            sv!("S23"),
            sv!("S48"),
            sv!("S36"),
            sv!("S44"),
            sv!("S23"),
            sv!("S26"),
            sv!("S48"),
            sv!("S36"),
            sv!("S44"),
            sv!("S23"),
            sv!("S26"),
            sv!("S48"),
            sv!("S44"),
            sv!("S36"),
            sv!("S23"),
            sv!("S28"),
            sv!("S48"),
            sv!("S44"),
            sv!("S26"),
            sv!("S27"),
            sv!("S28"),
            sv!("S36"),
            sv!("S23"),
            sv!("S48"),
            sv!("S44"),
            sv!("S36"),
            sv!("S26"),
            sv!("S23"),
            sv!("S48"),
            sv!("S44"),
            sv!("S27"),
            sv!("S36"),
            sv!("S48"),
            sv!("S23"),
            sv!("S26"),
            sv!("S44"),
            sv!("S36"),
            sv!("S48"),
            sv!("S23"),
            sv!("S44"),
            sv!("S26"),
            sv!("S27"),
            sv!("S48"),
            sv!("S36"),
            sv!("S23"),
            sv!("S44"),
            sv!("S28"),
            sv!("S48"),
            sv!("S36"),
            sv!("S26"),
            sv!("S23"),
            sv!("S44"),
            sv!("S48"),
            sv!("S27"),
            sv!("S36"),
            sv!("S23"),
            sv!("S26"),
            sv!("S44"),
            sv!("S48"),
            sv!("S36"),
            sv!("S23"),
            sv!("S44"),
            sv!("S48"),
            sv!("S26"),
            sv!("S28"),
            sv!("S27"),
            sv!("S36"),
            sv!("S23"),
            sv!("S44"),
            sv!("S48"),
            sv!("C05"),
            sv!("C08"),
            sv!("C10"),
            sv!("C13"),
            sv!("C14"),
            sv!("C20"),
            sv!("C21"),
            sv!("C26"),
            sv!("C27"),
            sv!("C28"),
            sv!("C29"),
            sv!("C30"),
            sv!("C32"),
            sv!("C33"),
            sv!("C35"),
            sv!("C36"),
            sv!("C38"),
            sv!("C41"),
            sv!("C42"),
            sv!("C45"),
            sv!("C46"),
            sv!("C60"),
            sv!("C29"),
            sv!("C45"),
            sv!("C30"),
            sv!("C26"),
            sv!("C35"),
            sv!("C32"),
            sv!("C41"),
            sv!("C36"),
            sv!("C20"),
            sv!("C13"),
            sv!("C08"),
            sv!("C38"),
            sv!("C05"),
            sv!("C24"),
            sv!("J04"),
        ]
        .into_iter()
        .unique()
        .collect();
        vehicles.sort(); // for comparison
        assert!(rinex.sv().sorted().eq(vehicles), "parsed wrong sv content");

        let mut eop_count = 0;
        let mut ion_count = 0;
        let mut sto_count = 0;
        for (e, frames) in record {
            for fr in frames {
                if let Some(fr) = fr.as_eph() {
                    let (msgtype, sv, ephemeris) = fr;
                    if sv.constellation == Constellation::QZSS {
                        if sv.prn != 4 {
                            panic!("got unexpected QZSS vehicle \"{}\"", sv.prn)
                        }
                        assert_eq!(*e, Epoch::from_str("2022-06-08T11:00:00 GPST").unwrap());
                        assert_eq!(msgtype, NavMsgType::LNAV);
                        assert_eq!(ephemeris.clock_bias, 1.080981455743E-04);
                        assert_eq!(ephemeris.clock_drift, 3.751665644813E-12);
                        assert_eq!(ephemeris.clock_drift_rate, 0.0);
                    }
                } else if let Some(fr) = fr.as_sto() {
                    sto_count += 1; // STO test
                    let (_msg, _sv, sto) = fr;
                    if sto.system.eq("GAUT") {
                        assert_eq!(*e, Epoch::from_str("2022-06-08T00:00:00 GST").unwrap());
                        assert_eq!(sto.t_tm, 295207);
                        assert_eq!(
                            sto.a,
                            (-1.862645149231E-09, 8.881784197001E-16, 0.000000000000E+00)
                        );
                    } else if sto.system.eq("GAGP") {
                        assert_eq!(*e, Epoch::from_str("2022-06-08T00:00:00 GST").unwrap());
                        assert_eq!(
                            sto.a,
                            (3.201421350241E-09, -4.440892098501E-15, 0.000000000000E+00)
                        );
                        assert_eq!(sto.t_tm, 295240);
                    } else if sto.system.eq("GPUT") {
                        assert_eq!(*e, Epoch::from_str("2022-06-10T19:56:48 GPST").unwrap());
                        assert_eq!(
                            sto.a,
                            (9.313225746155E-10, 2.664535259100E-15, 0.000000000000E+00)
                        );
                        assert_eq!(sto.t_tm, 295284);
                    } else {
                        panic!("got unexpected system time \"{}\"", sto.system)
                    }
                } else if let Some(_fr) = fr.as_eop() {
                    eop_count += 1; // EOP test
                                    //TODO
                                    // we do not have EOP frame examples at the moment
                } else if let Some(fr) = fr.as_ion() {
                    ion_count += 1; // ION test
                    let (_msg, _sv, model) = fr;
                    if let Some(model) = model.as_klobuchar() {
                        let e0 = Epoch::from_str("2022-06-08T09:59:48 GPST").unwrap();
                        let e1 = Epoch::from_str("2022-06-08T09:59:50 BDT").unwrap();
                        if *e == e0 {
                            assert_eq!(
                                model.alpha,
                                (
                                    1.024454832077E-08,
                                    2.235174179077E-08,
                                    -5.960464477539E-08,
                                    -1.192092895508E-07
                                )
                            );
                            assert_eq!(
                                model.beta,
                                (
                                    9.625600000000E+04,
                                    1.310720000000E+05,
                                    -6.553600000000E+04,
                                    -5.898240000000E+05
                                )
                            );
                        } else if *e == e1 {
                            assert_eq!(
                                model.alpha,
                                (
                                    2.142041921616E-08,
                                    1.192092895508E-07,
                                    -1.013278961182E-06,
                                    1.549720764160E-06
                                )
                            );
                            assert_eq!(
                                model.beta,
                                (
                                    1.208320000000E+05,
                                    1.474560000000E+05,
                                    -1.310720000000E+05,
                                    -6.553600000000E+04
                                )
                            );
                        } else {
                            panic!("misplaced ION message {:?} @ {}", model, e)
                        }
                        assert_eq!(model.region, KbRegionCode::WideArea);
                    } else if let Some(model) = model.as_nequick_g() {
                        assert_eq!(*e, Epoch::from_str("2022-06-08T09:59:57 GST").unwrap());
                        assert_eq!(model.region, NgRegionFlags::empty());
                    }
                }
            }
        }
        assert_eq!(sto_count, 3);
        assert_eq!(ion_count, 3);
        assert_eq!(eop_count, 0); // no EOP in this file
    }
    #[test]
    #[cfg(feature = "flate2")]
    fn v3_brdc00gop_r_2021_gz() {
        let test_resource = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/NAV/V3/BRDC00GOP_R_20210010000_01D_MN.rnx.gz";
        let rinex = Rinex::from_file(&test_resource);
        assert!(rinex.is_ok());

        let rinex = rinex.unwrap();
        assert!(rinex.is_navigation_rinex());
        assert!(rinex.header.obs.is_none());
        assert!(rinex.header.meteo.is_none());
        assert_eq!(rinex.epoch().count(), 4);

        let record = rinex.record.as_nav();
        assert!(record.is_some());

        let record = record.unwrap();
        let mut epochs: Vec<Epoch> = vec![
            Epoch::from_str("2021-01-01T00:00:00 BDT").unwrap(),
            Epoch::from_str("2021-01-01T07:15:00 UTC").unwrap(),
            Epoch::from_str("2021-01-01T01:28:00 GPST").unwrap(),
            Epoch::from_str("2021-01-01T08:20:00 GST").unwrap(),
        ];
        epochs.sort(); // for comparison purposes

        assert!(
            rinex.epoch().sorted().eq(epochs.clone()),
            "parsed wrong epoch content.\nExpecting {:?}\nGot {:?}",
            epochs.clone(),
            rinex.epoch().collect::<Vec<Epoch>>(),
        );

        let mut vehicles: Vec<SV> = vec![
            SV::from_str("E03").unwrap(),
            SV::from_str("C01").unwrap(),
            SV::from_str("R10").unwrap(),
            SV::from_str("S36").unwrap(),
        ];
        vehicles.sort(); // for comparison purposes
        assert!(rinex.sv().sorted().eq(vehicles), "parsed wrong sv content");

        for (_, frames) in record {
            for fr in frames {
                let fr = fr.as_eph();
                assert!(fr.is_some(), "only ephemeris frames expected here");
                let (msg, _sv, _data) = fr.unwrap();
                assert!(msg == NavMsgType::LNAV, "only lnav frame expected here");
            }
        }
    }
    #[test]
    #[cfg(feature = "nav")]
    #[cfg(feature = "flate2")]
    fn v4_nav_messages() {
        let test_resource = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz";
        let rinex = Rinex::from_file(&test_resource);
        assert!(rinex.is_ok());
        let rinex = rinex.unwrap();

        for (_epoch, (msg, sv, _ephemeris)) in rinex.ephemeris() {
            match sv.constellation {
                Constellation::GPS | Constellation::QZSS => {
                    let expected = [NavMsgType::LNAV, NavMsgType::CNAV, NavMsgType::CNV2];
                    assert!(
                        expected.contains(&msg),
                        "parsed invalid GPS/QZSS V4 message \"{}\"",
                        msg
                    );
                },
                Constellation::Galileo => {
                    let expected = [NavMsgType::FNAV, NavMsgType::INAV];
                    assert!(
                        expected.contains(&msg),
                        "parsed invalid Galileo V4 message \"{}\"",
                        msg
                    );
                },
                Constellation::BeiDou => {
                    let expected = [
                        NavMsgType::D1,
                        NavMsgType::D2,
                        NavMsgType::CNV1,
                        NavMsgType::CNV2,
                        NavMsgType::CNV3,
                    ];
                    assert!(
                        expected.contains(&msg),
                        "parsed invalid BeiDou V4 message \"{}\"",
                        msg
                    );
                },
                Constellation::Glonass => {
                    assert_eq!(
                        msg,
                        NavMsgType::FDMA,
                        "parsed invalid Glonass V4 message \"{}\"",
                        msg
                    );
                },
                Constellation::SBAS => {
                    assert_eq!(
                        msg,
                        NavMsgType::SBAS,
                        "parsed invalid SBAS V4 message \"{}\"",
                        msg
                    );
                },
                _ => {},
            }
        }
    }
    #[test]
    #[cfg(feature = "nav")]
    #[cfg(feature = "flate2")]
    fn v4_brd400dlr_s2023() {
        let path = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("NAV")
            .join("V4")
            .join("BRD400DLR_S_20230710000_01D_MN.rnx.gz");
        let rinex = Rinex::from_file(&path.to_string_lossy());
        assert!(
            rinex.is_ok(),
            "failed to parse NAV/V4/BRD400DLR_S_20230710000_01D_MN.rnx.gz, error: {:?}",
            rinex.err()
        );
        let rinex = rinex.unwrap();
        for (epoch, (msg, sv, data)) in rinex.ephemeris() {
            if sv == sv!("G01") {
                assert!(
                    (msg == NavMsgType::LNAV) || (msg == NavMsgType::CNAV),
                    "parsed bad ephemeris message {} for G01 {}",
                    msg,
                    epoch
                );

                if *epoch == Epoch::from_str("2023-03-12T12:00:00 UTC").unwrap() {
                    assert_eq!(msg, NavMsgType::LNAV);
                    assert_eq!(
                        data.sv_clock(),
                        (2.035847865045e-04, -3.865352482535e-12, 0.000000000000e+00)
                    );
                } else if *epoch == Epoch::from_str("2023-03-12T01:30:00 UTC").unwrap() {
                    assert_eq!(msg, NavMsgType::CNAV);
                    assert_eq!(
                        data.sv_clock(),
                        (2.037292579189e-04, -3.829825345747e-12, 0.000000000000e+00)
                    );
                } else if *epoch == Epoch::from_str("2023-03-12T12:23:00 UTC").unwrap() {
                    assert_eq!(msg, NavMsgType::CNAV);
                    assert_eq!(
                        data.sv_clock(),
                        (2.034264325630e-04, -3.819167204711e-12, 0.000000000000e+00)
                    );
                }
            } else if sv == sv!("C19") {
                assert!(
                    (msg == NavMsgType::D1)
                        || (msg == NavMsgType::CNV1)
                        || (msg == NavMsgType::CNV2),
                    "parsed bad ephemeris message {} for C19 {}",
                    msg,
                    epoch
                );

                if *epoch == Epoch::from_str("2023-03-12T12:00:00 UTC").unwrap() {
                    if msg == NavMsgType::CNV1 {
                        assert_eq!(
                            data.sv_clock(),
                            (-8.956581004895e-04, -1.113775738304e-12, 0.000000000000e+00)
                        );
                    } else if msg == NavMsgType::CNV2 {
                        assert_eq!(
                            data.sv_clock(),
                            (-8.956581004895e-04, -1.113775738304e-12, 0.000000000000e+00)
                        );
                    } else if msg == NavMsgType::D1 {
                        assert_eq!(
                            data.sv_clock(),
                            (-8.956581586972e-04, -1.113775738304e-12, 0.000000000000e+00)
                        );
                    }
                }
            } else if sv == sv!("J04") {
                assert!(
                    (msg == NavMsgType::LNAV)
                        || (msg == NavMsgType::CNAV)
                        || (msg == NavMsgType::CNV2),
                    "parsed bad ephemeris message {} for J04 {}",
                    msg,
                    epoch
                );
                if *epoch == Epoch::from_str("2023-03-12T12:03:00 UTC").unwrap() {
                    if msg == NavMsgType::LNAV {
                        assert_eq!(
                            data.sv_clock(),
                            (9.417533874512e-05, 0.000000000000e+00, 0.000000000000e+00)
                        );
                    } else if msg == NavMsgType::CNAV {
                        assert_eq!(
                            data.sv_clock(),
                            (9.417530964129e-05, -3.552713678801e-14, 0.000000000000e+00)
                        );
                    }
                }
            } else if sv == sv!("I09") {
                assert!(
                    msg == NavMsgType::LNAV,
                    "parsed bad ephemeris message {} for I09 {}",
                    msg,
                    epoch
                );
                if *epoch == Epoch::from_str("2023-03-12T20:05:36 UTC").unwrap() {
                    assert_eq!(
                        data.sv_clock(),
                        (7.255990058184e-04, 1.716671249596e-11, 0.000000000000e+00)
                    );
                }
            } else if sv == sv!("R10") {
                assert!(
                    msg == NavMsgType::FDMA,
                    "parsed bad ephemeris message {} for I09 {}",
                    msg,
                    epoch
                );
                if *epoch == Epoch::from_str("2023-03-12T01:45:00 UTC").unwrap() {
                    assert_eq!(
                        data.sv_clock(),
                        (-9.130407124758e-05, 0.000000000000e+00, 5.430000000000e+03)
                    );
                }
            }
        }
        for (epoch, (msg, sv, iondata)) in rinex.ionod_correction_models() {
            if sv == sv!("G21") {
                assert_eq!(msg, NavMsgType::LNAV);
                if *epoch == Epoch::from_str("2023-03-12T00:08:54 UTC").unwrap() {
                    let kb = iondata.as_klobuchar();
                    assert!(kb.is_some());
                    let kb = kb.unwrap();
                    assert_eq!(
                        kb.alpha,
                        (
                            2.887099981308e-08,
                            7.450580596924e-09,
                            -1.192092895508e-07,
                            0.000000000000e+00
                        )
                    );
                    assert_eq!(
                        kb.beta,
                        (
                            1.331200000000e+05,
                            0.000000000000e+00,
                            -2.621440000000e+05,
                            1.310720000000e+05
                        )
                    );
                    assert_eq!(kb.region, KbRegionCode::WideArea);
                } else if *epoch == Epoch::from_str("2023-03-12T23:41:24 UTC").unwrap() {
                    let kb = iondata.as_klobuchar();
                    assert!(kb.is_some());
                    let kb = kb.unwrap();
                    assert_eq!(
                        kb.alpha,
                        (
                            2.887099981308e-08,
                            7.450580596924e-09,
                            -1.192092895508e-07,
                            0.000000000000e+00
                        )
                    );
                    assert_eq!(
                        kb.beta,
                        (
                            1.331200000000e+05,
                            0.000000000000e+00,
                            -2.621440000000e+05,
                            1.310720000000e+05
                        )
                    );
                    assert_eq!(kb.region, KbRegionCode::WideArea);
                }
            } else if sv == sv!("G21") {
                assert_eq!(msg, NavMsgType::CNVX);
            } else if sv == sv!("J04")
                && *epoch == Epoch::from_str("2023-03-12T02:01:54 UTC").unwrap()
            {
                let kb = iondata.as_klobuchar();
                assert!(kb.is_some());
                let kb = kb.unwrap();
                assert_eq!(
                    kb.alpha,
                    (
                        3.259629011154e-08,
                        -1.490116119385e-08,
                        -4.172325134277e-07,
                        -1.788139343262e-07
                    )
                );
                assert_eq!(
                    kb.beta,
                    (
                        1.269760000000e+05,
                        -1.474560000000e+05,
                        1.310720000000e+05,
                        2.490368000000e+06
                    )
                );
                assert_eq!(kb.region, KbRegionCode::WideArea);
            }
        }
        for (epoch, (msg, sv, eop)) in rinex.earth_orientation() {
            if sv == sv!("J04") {
                assert_eq!(msg, NavMsgType::CNVX);
                if *epoch == Epoch::from_str("2023-03-12T06:00:00 UTC").unwrap() {
                    assert_eq!(
                        eop.x,
                        (-4.072475433350e-02, 2.493858337402e-04, 0.000000000000e+00)
                    );
                    assert_eq!(
                        eop.y,
                        (3.506240844727e-01, 3.324031829834e-03, 0.000000000000e+00)
                    );
                    assert_eq!(eop.t_tm, 18186);
                    assert_eq!(
                        eop.delta_ut1,
                        (-1.924991607666e-02, -7.354915142059e-04, 0.000000000000e+00)
                    );
                }
            } else if sv == sv!("C30") {
                assert_eq!(msg, NavMsgType::CNVX);
                if *epoch == Epoch::from_str("2023-03-12T11:00:00 UTC").unwrap() {
                    assert_eq!(
                        eop.x,
                        (-4.079341888428e-02, 6.389617919922e-04, 0.000000000000e+00)
                    );
                    assert_eq!(
                        eop.y,
                        (3.462553024292e-01, 2.998828887939e-03, 0.000000000000e+00)
                    );
                    assert_eq!(eop.t_tm, 60483);
                    assert_eq!(
                        eop.delta_ut1,
                        (-1.820898056030e-02, -5.761086940765e-04, 0.000000000000e+00)
                    );
                }
            }
        }
    }
    #[test]
    #[cfg(feature = "nav")]
    #[cfg(feature = "flate2")]
    #[ignore]
    fn sv_interp() {
        let path = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("NAV")
            .join("V3")
            .join("MOJN00DNK_R_20201770000_01D_MN.rnx.gz");
        let rinex = Rinex::from_file(&path.to_string_lossy());
        assert!(
            rinex.is_ok(),
            "failed to parse NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz, error: {:?}",
            rinex.err()
        );
        let rinex = rinex.unwrap();
        let first_epoch = rinex.first_epoch().expect("failed to determine 1st epoch");
        let last_epoch = rinex.last_epoch().expect("failed to determine last epoch");
        let dt = rinex.dominant_sample_rate().unwrap();
        let total_epochs = rinex.epoch().count();

        for (order, max_error) in [(7, 1E-1_f64), (9, 1.0E-2_64), (11, 0.5E-3_f64)] {
            let tmin = first_epoch + (order / 2) * dt;
            let tmax = last_epoch - (order / 2) * dt;
            println!("running Interp({}) testbench..", order);
            for (index, (epoch, sv, (x, y, z))) in rinex.sv_position().enumerate() {
                let feasible = epoch > tmin && epoch <= tmax;
                let interpolated = rinex.sv_position_interpolate(sv, epoch, order as usize);
                let achieved = interpolated.is_some();
                //DEBUG
                println!(
                    "tmin: {} | tmax: {} | epoch: {} | feasible : {} | achieved: {}",
                    tmin, tmax, epoch, feasible, achieved
                );
                if feasible {
                    assert!(
                        achieved == feasible,
                        "interpolation should have been feasible @ epoch {}",
                        epoch,
                    );
                } else {
                    assert!(
                        achieved == feasible,
                        "interpolation should not have been feasible @ epoch {}",
                        epoch,
                    );
                }
                if !feasible {
                    continue;
                }
                //TODO FIX THIS PLEASE
                if interpolated.is_none() {
                    continue;
                }
                /*
                 * test interpolation errors
                 */
                let (x_interp, y_interp, z_interp) = interpolated.unwrap();
                let err = (
                    (x_interp - x).abs() * 1.0E3, // error in km
                    (y_interp - y).abs() * 1.0E3,
                    (z_interp - z).abs() * 1.0E3,
                );
                assert!(
                    err.0 < max_error,
                    "x error too large: {} for Interp({}) for {} @ Epoch {}/{}",
                    err.0,
                    order,
                    sv,
                    index,
                    total_epochs,
                );
                assert!(
                    err.1 < max_error,
                    "y error too large: {} for Interp({}) for {} @ Epoch {}/{}",
                    err.1,
                    order,
                    sv,
                    index,
                    total_epochs,
                );
                assert!(
                    err.2 < max_error,
                    "z error too large: {} for Interp({}) for {} @ Epoch {}/{}",
                    err.2,
                    order,
                    sv,
                    index,
                    total_epochs,
                );
            }
        }
    }
    #[test]
    #[cfg(feature = "nav")]
    fn sv_toe_ephemeris() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("NAV")
            .join("V3")
            .join("AMEL00NLD_R_20210010000_01D_MN.rnx");
        let rinex = Rinex::from_file(path.to_string_lossy().as_ref());
        assert!(rinex.is_ok());
        let rinex = rinex.unwrap();
        for (toc, (_, sv, ephemeris)) in rinex.ephemeris() {
            let e0 = Epoch::from_str("2021-01-01T00:00:00 BDT").unwrap();
            let e1 = Epoch::from_str("2021-01-01T05:00:00 BDT").unwrap();
            let e2 = Epoch::from_str("2021-01-01T10:10:00 GST").unwrap();
            let e3 = Epoch::from_str("2021-01-01T15:40:00 GST").unwrap();

            let ts = sv.timescale();
            assert!(ts.is_some(), "timescale should be determined");
            let ts = ts.unwrap();

            if let Some(toe) = ephemeris.toe(ts) {
                let mut expected_sv = SV::default();
                let mut expected_toe = Epoch::default();
                if *toc == e0 {
                    expected_toe = Epoch::from_str("2021-01-01T00:00:33 BDT").unwrap();
                    expected_sv = sv!("C05");
                } else if *toc == e1 {
                    expected_toe = Epoch::from_str("2021-01-01T05:00:33 BDT").unwrap();
                    expected_sv = sv!("C21");
                } else if *toc == e2 {
                    expected_toe = Epoch::from_str("2021-01-01T10:10:19 GST").unwrap();
                    expected_sv = sv!("E01");
                } else if *toc == e3 {
                    expected_toe = Epoch::from_str("2021-01-01T15:40:19 GST").unwrap();
                    expected_sv = sv!("E03");
                } else {
                    panic!("unhandled toc {}", toc);
                }
                assert_eq!(sv, expected_sv, "wrong sv");
                assert_eq!(toe, expected_toe, "wrong toe evaluated");
                /*
                 * Rinex.sv_ephemeris(@ toe) should return exact ephemeris
                 */
                assert_eq!(
                    rinex.sv_ephemeris(expected_sv, toe),
                    Some((expected_toe, ephemeris)),
                    "sv_ephemeris(sv,t) @ toe should strictly identical ephemeris"
                );
            }
        }
    }
    #[test]
    #[cfg(feature = "nav")]
    fn v3_ionospheric_corr() {
        let path = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("NAV")
            .join("V3")
            .join("CBW100NLD_R_20210010000_01D_MN.rnx");
        let rinex = Rinex::from_file(&path.to_string_lossy());
        assert!(
            rinex.is_ok(),
            "failed to parse NAV/V3/BCW100NLD_R_2021, error: {:?}",
            rinex.err()
        );
        let rinex = rinex.unwrap();

        for (t0, should_work) in [
            // MIDNIGHT T0 exact match
            (Epoch::from_gregorian_utc_at_midnight(2021, 01, 01), true),
            // VALID day course : 1sec into that day
            (Epoch::from_gregorian_utc(2021, 01, 01, 00, 00, 1, 0), true),
            // VALID day course : random into that dat
            (Epoch::from_gregorian_utc(2021, 01, 01, 05, 33, 24, 0), true),
            // VALID day course : 1 sec prior next day
            (Epoch::from_str("2021-01-01T23:59:59 GPST").unwrap(), true),
            // TOO LATE : MIDNIGHT DAY +1
            (Epoch::from_str("2021-01-02T00:00:00 GPST").unwrap(), false),
            // TOO LATE : MIDNIGHT DAY +1
            (Epoch::from_gregorian_utc_at_midnight(2021, 02, 01), false),
            // TOO EARLY
            (Epoch::from_gregorian_utc_at_midnight(2020, 12, 31), false),
        ] {
            let ionod_corr = rinex.ionod_correction(
                t0,
                30.0,               // fake elev: DONT CARE
                30.0,               // fake azim: DONT CARE
                10.0,               // fake latitude: DONT CARE
                20.0,               // fake longitude: DONT CARE
                Carrier::default(), // fake signal: DONT CARE
            );
            if should_work {
                assert!(
                    ionod_corr.is_some(),
                    "v3 ionod corr: should have returned a correction model for datetime {:?}",
                    t0
                );
            } else {
                assert!(
                    ionod_corr.is_none(),
                    "v3 ionod corr: should not have returned a correction model for datetime {:?}",
                    t0
                );
            }
        }
    }
}
