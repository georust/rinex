#[cfg(test)]
mod test {
    use rinex::*;
    use std::str::FromStr;
    use std::process::Command;
    #[test]
    fn v2_amel0010_21g() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/NAV/V2/amel0010.21g";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_navigation_rinex(), true);
        assert_eq!(rinex.header.obs.is_none(), true);
        assert_eq!(rinex.header.meteo.is_none(), true);
        /*
        // RECORD
        match data {
        "NAV" => {
            // NAV files checks
            assert_eq!(rinex.header.obs.is_none(), true);
            assert_eq!(rinex.is_navigation_rinex(), true);
            assert_eq!(rinex.header.meteo.is_none(), true);
            let record = rinex.record.as_nav().unwrap();
            println!("----- EPOCHs ----- \n{:#?}", record.keys());
            let mut epochs = record.keys();
            // Testing event description finder
            if let Some(event) = epochs.nth(0) {
                // [!] with dummy t0 = 1st epoch timestamp
                    //     this will actually return `header section` timestamps
                    println!("EVENT @ {:#?} - description: {:#?}", event, rinex.event_description(*event)); 
                }
            },
        }*/
    }
}
