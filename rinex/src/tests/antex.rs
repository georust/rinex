#[cfg(test)]
mod test {
    use crate::antex::pcv::Pcv;
    use crate::antex::CalibrationMethod;
    use crate::prelude::*;
    #[test]
    fn v1_trosar_25r4_leit_2020_09_23() {
        let test_resource = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/ATX/V1/TROSAR25.R4__LEIT_2020_09_23.atx";
        let rinex = Rinex::from_file(&test_resource);
        assert!(rinex.is_ok());
        let rinex = rinex.unwrap();
        assert!(rinex.is_antex());
        let header = rinex.header;
        assert_eq!(header.version.major, 1);
        assert_eq!(header.version.minor, 4);
        assert!(header.antex.is_some());
        let atx_header = header.antex.as_ref().unwrap();
        assert_eq!(atx_header.pcv_type, Pcv::Absolute);

        /*
         * record test
         */
        let record = rinex.record.as_antex();
        assert!(record.is_some());
        let record = record.unwrap();

        //assert_eq!(record.len(), 1); // Only 1 antenna
        //let (antenna, frequencies) = record.first().unwrap();
        //assert_eq!(antenna.ant_type, "TROSAR25.R4");
        //assert_eq!(antenna.sn, "LEIT727259");
        //let cal = &antenna.calibration;
        //assert_eq!(cal.method, CalibrationMethod::Chamber);
        //assert_eq!(cal.agency, "IGG, Univ. Bonn");
        //assert_eq!(cal.date, "23-SEP-20");
        //assert_eq!(antenna.dazi, 5.0);
        //assert_eq!(antenna.zen, (0.0, 90.0));
        //assert_eq!(antenna.dzen, 5.0);
        //assert!(antenna.valid_from.is_none());
        //assert!(antenna.valid_until.is_none());
        //for freq in frequencies.iter() {
        //    let first = freq.patterns.first();
        //    assert!(first.is_some());
        //    let first = first.unwrap();
        //    assert!(!first.is_azimuth_dependent());
        //    let mut angle = 0.0_f64;
        //    for i in 1..freq.patterns.len() {
        //        let p = &freq.patterns[i];
        //        assert!(p.is_azimuth_dependent());
        //        let (a, _) = p.azimuth_pattern().unwrap();
        //        assert_eq!(angle, a);
        //        angle += antenna.dzen;
        //    }
        //}
    }
}
