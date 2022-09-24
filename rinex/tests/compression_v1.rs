#[cfg(test)]
mod test {
    use rinex::*;
    use rinex::header::Header;
    use rinex::version::Version;
    use rinex::hatanaka::Compressor;
    use rinex::observation;
    use rinex::observation::Crinex;
    use std::collections::HashMap;
    use rinex::constellation::Constellation;
    #[test]
    fn crx1_compression() {
        // object
        let mut compressor = Compressor::new(Compressor::MAX_COMPRESSION_ORDER)
            .unwrap();
        // fake header
        let mut header = Header::basic_obs()
            .with_version(Version {
                major: 2,
                minor: 11,
            });
        header.obs = Some(observation::HeaderFields {
            crinex: Some(
                Crinex {
                    version: Version { 
                        minor: 0,
                        major: 1,
                    },
                    prog: String::from("Test"),
                    date: chrono::Utc::now().naive_utc(),
                }
            ),
            codes: {
                let mut codes: HashMap<Constellation, Vec<String>> = HashMap::new();
                codes.insert(
                    Constellation::GPS,
                    vec![
                        String::from("L1"),
                        String::from("L2"),
                        String::from("C1"),
                        String::from("P2"),
                        String::from("P1"),
                        String::from("S1"),
                        String::from("S2")
                    ]);
                codes.insert(
                    Constellation::Glonass,
                    vec![
                        String::from("L1"),
                        String::from("L2"),
                        String::from("C1"),
                        String::from("P2"),
                        String::from("P1"),
                        String::from("S1"),
                        String::from("S2")
                    ]);
                codes
            },
            clock_offset_applied: false,
        });
        
        // RNX epoch
        let content = " 21  1  1  0  0  0.0000000  0 20G07G23G26G20G21G18R24R09G08G27G10G16
                                R18G13R01R16R17G15R02R15";

        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // + empty clock offset line, due to missing field [epoch #1]
        let expected = "&21  1  1  0  0  0.0000000  0 20G07G23G26G20G21G18R24R09G08G27G10G16R18G13R01R16R17G15R02R15\n\n";
        assert_eq!(result, expected);
        
        // epoch#1 sat#1
        let content = " 126298057.858 6  98414080.64743  24033720.416    24033721.351    24033719.353";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, ""); // description not complete

        let content = "        40.000          22.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // description now complete
        assert_eq!(result, "3&126298057858 3&98414080647 3&24033720416 3&24033721351 3&24033719353 3&40000 3&22000  643        4");
    }
       
}
