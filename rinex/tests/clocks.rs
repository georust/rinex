#[cfg(test)]
mod test {
    use rinex::*;
    #[test]
    fn v3_demo() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/CLK/V3/USNO1.txt";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.header.clocks.is_some(), true);
        let clocks = rinex.header.clocks
            .as_ref()
            .unwrap();
        assert_eq!(clocks.codes, vec![
            clocks::DataType::As,
            clocks::DataType::Ar,
            clocks::DataType::Cr,
            clocks::DataType::Dr]);
        assert_eq!(clocks.agency, Some(clocks::Agency {
            code: String::from("USN"),
            name: String::from("USNO USING GIPSY/OASIS-II"),
        }));
        assert_eq!(clocks.station, Some(clocks::Station {
            name: String::from("USNO"),
            id: String::from("40451S003"),
        }));
        //assert_eq!(rinex.is_ok(), true);
        //let rinex = rinex.unwrap();
        //assert_eq!(rinex.is_clocks_rinex(), true);
    }

    #[test]
    fn is_new_epoch() {
        let l = "AR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(clocks::is_new_epoch(l), true);
        let l = "RA AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(clocks::is_new_epoch(l), false);
        let l = "DR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(clocks::is_new_epoch(l), true);
        let l = "CR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(clocks::is_new_epoch(l), true);
        let l = "AS AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(clocks::is_new_epoch(l), true);
    }
}
