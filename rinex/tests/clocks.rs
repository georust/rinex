#[cfg(test)]
mod test {
    use rinex::*;
    use rinex::clocks;
    use rinex::clocks::record::DataType;
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
            DataType::As,
            DataType::Ar,
            DataType::Cr,
            DataType::Dr]);
        assert_eq!(clocks.agency, Some(clocks::Agency {
            code: String::from("USN"),
            name: String::from("USNO USING GIPSY/OASIS-II"),
        }));
        assert_eq!(clocks.station, Some(clocks::Station {
            name: String::from("USNO"),
            id: String::from("40451S003"),
        }));
        println!("{:#?}", rinex.record);
        //assert_eq!(rinex.is_ok(), true);
        //let rinex = rinex.unwrap();
        //assert_eq!(rinex.is_clocks_rinex(), true);
    }
}
