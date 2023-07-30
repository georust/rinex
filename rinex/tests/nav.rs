#[cfg(test)]
mod test {
    use itertools::*;
    use rinex::navigation::*;
    use rinex::prelude::*;
    use std::str::FromStr;
    #[test]
    fn v2_amel0010_21g() {
        let test_resource =
            env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/NAV/V2/amel0010.21g";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_navigation_rinex(), true);
        assert_eq!(rinex.header.obs.is_none(), true);
        assert_eq!(rinex.header.meteo.is_none(), true);
        let record = rinex.record.as_nav();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        assert_eq!(record.len(), 4);
        let expected_epochs: Vec<Epoch> = vec![
            Epoch::from_gregorian_utc(2020, 12, 31, 23, 45, 0, 0),
            Epoch::from_gregorian_utc(2021, 01, 01, 11, 15, 0, 0),
            Epoch::from_gregorian_utc(2021, 01, 01, 11, 45, 0, 0),
            Epoch::from_gregorian_utc(2021, 01, 01, 16, 15, 0, 0),
        ];
        let expected_prns: Vec<u8> = vec![1, 2, 7, 3, 4, 5];
        let mut expected_vehicles: Vec<Sv> = Vec::new();
        for prn in expected_prns.iter() {
            expected_vehicles.push(Sv {
                constellation: Constellation::Glonass,
                prn: *prn,
            });
        }

        for (index, (e, classes)) in record.iter().enumerate() {
            assert_eq!(*e, expected_epochs[index]);
            for (class, frames) in classes.iter() {
                // only Legacy Ephemeris in V2
                assert_eq!(*class, FrameClass::Ephemeris);
                for frame in frames.iter() {
                    // only EPH
                    let ephemeris = frame.as_eph(); // ONLY EPH in V2
                    assert_eq!(ephemeris.is_some(), true);
                    let (msgtype, sv, ephemeris) = ephemeris.unwrap();
                    assert_eq!(msgtype, &MsgType::LNAV); // legacy NAV
                    assert_eq!(expected_vehicles.contains(&sv), true);
                    if sv.prn == 1 {
                        assert_eq!(ephemeris.clock_bias, 7.282570004460E-5);
                        assert_eq!(ephemeris.clock_drift, 0.0);
                        assert_eq!(ephemeris.clock_drift_rate, 7.380000000000E+04);
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
    }
    #[test]
    fn v3_amel00nld_r_2021() {
        let test_resource = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_navigation_rinex(), true);
        assert_eq!(rinex.header.obs.is_none(), true);
        assert_eq!(rinex.header.meteo.is_none(), true);
        let record = rinex.record.as_nav();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        assert_eq!(record.len(), 6);
        let expected_epochs: Vec<Epoch> = vec![
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 00, 0, 0),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 15, 0, 0),
            Epoch::from_gregorian_utc(2021, 01, 01, 05, 00, 0, 0),
            Epoch::from_gregorian_utc(2021, 01, 01, 09, 45, 0, 0),
            Epoch::from_gregorian_utc(2021, 01, 01, 10, 10, 0, 0),
            Epoch::from_gregorian_utc(2021, 01, 01, 15, 40, 0, 0),
        ];
        assert_eq!(rinex.epochs(), expected_epochs);

        let mut index = 0;
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
                // only Legacy Ephemeris in V3
                assert_eq!(*class, FrameClass::Ephemeris);
                for frame in frames.iter() {
                    // Only EPH in V3
                    let ephemeris = frame.as_eph(); // Only EPH in V3
                    assert_eq!(ephemeris.is_some(), true);
                    let (msgtype, sv, ephemeris) = ephemeris.unwrap();
                    assert_eq!(msgtype, &MsgType::LNAV); // legacy NAV
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
                        _ => panic!("falsely identified \"{}\"", sv.to_string()),
                    }
                }
            }
            index += 1
        }
    }
    //#[cfg(feature = "flate2")]
    //use itertools::*;
    #[test]
    #[cfg(feature = "flate2")]
    fn v4_kms300dnk_r_202215910() {
        let test_resource = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_navigation_rinex(), true);
        assert_eq!(rinex.header.obs.is_none(), true);
        assert_eq!(rinex.header.meteo.is_none(), true);
        let record = rinex.record.as_nav();
        assert_eq!(record.is_some(), true);
        let epochs: Vec<Epoch> = vec![
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 44, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 44, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 44, 32, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 44, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 57, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 12, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 50, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 44, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 48, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 57, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 44, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 48, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 12, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 12, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 12, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 12, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 12, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 12, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 12, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 12, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 12, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 10, 19, 56, 48, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 12, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 12, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 12, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 12, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 12, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 15, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 15, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 15, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 15, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 15, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 15, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 15, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 15, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 45, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 08, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 08, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 07, 20, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 08, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 06, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 06, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 08, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 08, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 08, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 07, 20, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 08, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 06, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 08, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 57, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 00, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 00, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 50, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 10, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 10, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 10, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 10, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 10, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 10, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 10, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 10, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 10, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 10, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 10, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 10, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 20, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 20, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 20, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 20, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 20, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 20, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 20, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 20, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 20, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 20, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 30, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 12, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 58, 56, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 32, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 32, 00),
            Epoch::from_gregorian_utc(2022, 06, 07, 23, 59, 44, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 58, 24, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 01, 36, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 01, 20, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 02, 40, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 02, 08, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 02, 40, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 02, 40, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 02, 24, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 04, 16, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 04, 16, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 04, 48, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 04, 48, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 04, 32, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 06, 56, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 06, 56, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 06, 24, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 06, 56, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 07, 12, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 06, 40, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 08, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 08, 32, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 09, 04, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 08, 48, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 09, 52, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 10, 08, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 11, 12, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 10, 40, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 10, 56, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 11, 12, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 12, 16, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 12, 32, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 12, 48, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 12, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 13, 20, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 14, 24, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 15, 28, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 14, 08, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 14, 56, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 15, 28, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 15, 28, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 16, 32, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 16, 16, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 17, 04, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 17, 36, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 18, 08, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 18, 40, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 18, 24, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 19, 44, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 19, 12, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 19, 44, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 20, 48, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 20, 32, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 21, 04, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 21, 20, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 21, 52, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 22, 56, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 21, 36, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 24, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 23, 28, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 23, 44, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 24, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 24, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 23, 44, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 25, 36, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 26, 08, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 25, 52, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 26, 08, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 26, 40, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 28, 16, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 27, 44, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 28, 16, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 28, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 28, 16, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 29, 20, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 29, 52, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 29, 04, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 30, 24, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 30, 24, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 32, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 31, 12, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 32, 16, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 32, 32, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 32, 32, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 33, 20, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 34, 08, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 34, 40, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 34, 40, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 34, 56, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 35, 28, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 36, 32, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 36, 48, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 36, 48, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 37, 36, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 37, 36, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 38, 56, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 38, 40, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 38, 56, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 36, 48, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 38, 40, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 32, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 41, 04, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 41, 04, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 48, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 41, 20, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 40, 48, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 42, 08, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 42, 56, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 43, 12, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 43, 28, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 42, 56, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 44, 16, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 45, 20, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 45, 04, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 45, 04, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 45, 36, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 46, 08, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 46, 24, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 47, 12, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 47, 12, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 47, 44, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 48, 32, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 48, 48, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 49, 36, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 48, 16, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 49, 20, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 49, 52, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 50, 40, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 49, 36, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 50, 24, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 51, 28, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 51, 28, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 52, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 52, 48, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 52, 32, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 53, 52, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 53, 36, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 54, 08, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 54, 24, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 54, 56, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 54, 40, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 55, 44, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 56, 16, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 56, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 55, 44, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 57, 04, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 58, 08, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 58, 08, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 57, 52, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 58, 24, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 58, 08, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 57, 52, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 07, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 08, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 07, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 07, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 08, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 08, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 50, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 06, 08, 11, 00, 00, 00),
        ];
        let mut epochs: Vec<_> = epochs.iter().unique().collect();
        epochs.sort(); // like the parser will

        let record = rinex.record.as_nav();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();

        /*
         * test epochs
         */
        for (index, (e, _)) in record.iter().enumerate() {
            assert_eq!(epochs[index], e);
        }

        let mut eop_count = 0;
        let mut ion_count = 0;
        let mut sto_count = 0;
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
                if *class == FrameClass::SystemTimeOffset {
                    sto_count += frames.len(); // STO testbench
                    for frame in frames.iter() {
                        let (msg, sv, sto) = frame.as_sto().unwrap();
                        if sto.system.eq("GAUT") {
                            assert_eq!(*e, Epoch::from_gregorian_utc(2022, 06, 08, 00, 00, 00, 00));
                            assert_eq!(sto.t_tm, 295207);
                            assert_eq!(
                                sto.a,
                                (-1.862645149231E-09, 8.881784197001E-16, 0.000000000000E+00)
                            );
                        } else if sto.system.eq("GAGP") {
                            assert_eq!(*e, Epoch::from_gregorian_utc(2022, 06, 08, 00, 00, 00, 00));
                            assert_eq!(
                                sto.a,
                                (3.201421350241E-09, -4.440892098501E-15, 0.000000000000E+00)
                            );
                            assert_eq!(sto.t_tm, 295240);
                        } else if sto.system.eq("GPUT") {
                            assert_eq!(*e, Epoch::from_gregorian_utc(2022, 06, 10, 19, 56, 48, 00));
                            assert_eq!(
                                sto.a,
                                (9.313225746155E-10, 2.664535259100E-15, 0.000000000000E+00)
                            );
                            assert_eq!(sto.t_tm, 295284);
                        } else {
                            panic!("got unexpected system time \"{}\"", sto.system)
                        }
                    }
                } else if *class == FrameClass::EarthOrientation {
                    eop_count += frames.len(); // EOP testbench
                } else if *class == FrameClass::IonosphericModel {
                    ion_count += frames.len(); // ION testbench
                    for frame in frames.iter() {
                        let (msg, sv, model) = frame.as_ion().unwrap();
                        if let Some(model) = model.as_klobuchar() {
                            let e0 = Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 48, 0);
                            let e1 = Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 50, 0);
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
                                panic!("misplaced ION message")
                            }
                            assert_eq!(model.region, KbRegionCode::WideArea);
                        } else if let Some(model) = model.as_nequick_g() {
                            assert_eq!(*e, Epoch::from_gregorian_utc(2022, 06, 08, 09, 59, 57, 00));
                            assert_eq!(model.region, NgRegionFlags::empty());
                        }
                    }
                } else if *class == FrameClass::Ephemeris {
                    for frame in frames.iter() {
                        let (msgtype, sv, ephemeris) = frame.as_eph().unwrap();
                        if sv.constellation == Constellation::QZSS {
                            if sv.prn != 4 {
                                panic!("got unexpected QZSS vehicle \"{}\"", sv.prn)
                            }
                            assert_eq!(*e, Epoch::from_gregorian_utc(2022, 06, 08, 11, 00, 00, 00));
                            assert_eq!(msgtype, &MsgType::LNAV);
                            assert_eq!(ephemeris.clock_bias, 1.080981455743E-04);
                            assert_eq!(ephemeris.clock_drift, 3.751665644813E-12);
                            assert_eq!(ephemeris.clock_drift_rate, 0.0);
                        }
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
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_navigation_rinex(), true);
        assert_eq!(rinex.header.obs.is_none(), true);
        assert_eq!(rinex.header.meteo.is_none(), true);
        //assert_eq!(rinex.epochs().len(), 4);
        let record = rinex.record.as_nav();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        let mut epochs: Vec<Epoch> = vec![
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 00, 00, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 01, 28, 00, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 07, 15, 00, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 08, 20, 00, 00),
        ];
        epochs.sort();
        assert_eq!(rinex.epochs(), epochs);

        let mut vehicles: Vec<Sv> = vec![
            Sv::from_str("C01").unwrap(),
            Sv::from_str("S36").unwrap(),
            Sv::from_str("R10").unwrap(),
            Sv::from_str("E03").unwrap(),
        ];
        vehicles.sort();
        assert_eq!(rinex.space_vehicles(), vehicles);

        for (epoch, classes) in record {
            for (class, frames) in classes.iter() {
                if *class == FrameClass::Ephemeris {
                    for frame in frames.iter() {
                        let (_, sv, _) = frame.as_eph().unwrap();
                    }
                }
            }
        }
    }
}
