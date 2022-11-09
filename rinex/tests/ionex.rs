#[cfg(test)]
mod test {
    use rinex::ionex::*;
    use rinex::prelude::*;
    #[test]
    fn v1_ckmg0020_22i() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/IONEX/V1/CKMG0020.22I.gz";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_ionex(), true);
        let header = rinex.header;
        assert_eq!(header.version.major, 1);
        assert_eq!(header.version.minor, 0);
        assert_eq!(header.ionex.is_some(), true);
        let header = header
            .ionex
            .as_ref()
            .unwrap();
    }
}
