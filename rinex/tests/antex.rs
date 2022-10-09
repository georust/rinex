#[cfg(test)]
mod test {
    use rinex::*;
    use rinex::antex::pcv::Pcv;
    use rinex::antex::CalibrationMethod;
    #[test]
    fn v1_trosar_25r4_leit_2020_09_23() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/ATX/V1/TROSAR25.R4__LEIT_2020_09_23.atx";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_antex_rinex(), true);
        let header = rinex.header;
        assert_eq!(header.version.major, 1);
        assert_eq!(header.version.minor, 4);
        assert_eq!(header.antex.is_some(), true);
        let atx_header = header
            .antex
            .as_ref()
            .unwrap();
        assert_eq!(atx_header.pcv, Pcv::Absolute); 
        let record = rinex.record.as_antex();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        assert_eq!(record.len(), 1); // Only 1 antenna
        let (antenna, frequencies) = record.first()
            .unwrap();
        assert_eq!(antenna.ant_type, "TROSAR25.R4"); 
        assert_eq!(antenna.sn, "LEIT727259");
        let cal = &antenna.calibration;
        assert_eq!(cal.method, CalibrationMethod::Chamber);
        assert_eq!(cal.agency, "IGG, Univ. Bonn");
        assert_eq!(cal.date, "23-SEP-20");
        assert_eq!(antenna.dazi, 5.0);
        assert_eq!(antenna.zen, (0.0, 90.0));
        assert_eq!(antenna.dzen, 5.0);
        assert_eq!(antenna.valid_from.is_none(), true);
        assert_eq!(antenna.valid_until.is_none(), true);
        for freq in frequencies.iter() {
            let first = freq.patterns.first();
            assert_eq!(first.is_some(), true);
            let first = first.unwrap();
            assert_eq!(first.is_azimuth_dependent(), false);
            let mut angle = 0.0_f64;
            for i in 1..freq.patterns.len() {
                let p = &freq.patterns[i];
                assert_eq!(p.is_azimuth_dependent(), true);
                let (a, _) = p.azimuth_pattern().unwrap();
                assert_eq!(angle, a);
                angle += antenna.dzen;
            }
        }
    }
}
