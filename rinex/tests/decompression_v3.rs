#[cfg(test)]
mod test {
    use rinex::*;
    use rinex::header::Header;
    use rinex::version::Version;
    use rinex::hatanaka::Hatanaka;
    use rinex::observation::Crinex;
    use rinex::observation::HeaderFields;
    use rinex::constellation::{
        Constellation,
        augmentation::Augmentation,
    };
    use std::collections::HashMap;
    #[test]
    fn test_crx3_decompression() {
        // object
        let mut decompressor = Hatanaka::new(Hatanaka::MAX_COMPRESSION_ORDER)
            .unwrap();
        // fake header
        let mut header = Header::basic_obs()
            .with_version(Version {
                major: 3,
                minor: 5,
            });
        header.obs = Some(observation::HeaderFields {
            crinex: Some(
                Crinex {
                    version: Version { 
                        minor: 0,
                        major: 3,
                    },
                    prog: String::from("Test"),
                    date: chrono::Utc::now().naive_utc(),
                }
            ),
            codes: {
                let mut codes: HashMap<Constellation, Vec<String>> = HashMap::new();
                codes.insert(
                    Constellation::BeiDou,
                    vec![
                        String::from("C2I"),
                        String::from("C6I"),
                        String::from("C7I"),
                        String::from("D2I"),
                        String::from("D6I"),
                        String::from("D7I"),
                        String::from("L2I"),
                        String::from("L6I"),
                        String::from("L7I"),
                        String::from("S2I"),
                        String::from("S6I"),
                        String::from("L6C"),
                    ]);
                codes.insert(
                    Constellation::Galileo,
                    vec![
                        String::from("C1C"),
                        String::from("C5Q"), 
                        String::from("C6C"), 
                        String::from("C7Q"), 
                        String::from("C8Q"), 
                        String::from("D1C"), 
                        String::from("D5Q"), 
                        String::from("D6C"), 
                        String::from("D7Q"), 
                        String::from("D8Q"), 
                        String::from("L1C"), 
                        String::from("L5Q"), 
                        String::from("L6C"),
                        String::from("L7Q"), 
                        String::from("L8Q"), 
                        String::from("S1C"), 
                        String::from("S5Q"), 
                        String::from("S6C"), 
                        String::from("S7Q"), 
                        String::from("S8Q"),
                    ]);
                codes.insert(
                    Constellation::GPS,
                    vec![
                        String::from("C1C"),
                        String::from("C1W"), 
                        String::from("C2L"), 
                        String::from("C2W"), 
                        String::from("C5Q"), 
                        String::from("D1C"), 
                        String::from("D2L"), 
                        String::from("D2W"), 
                        String::from("D5Q"), 
                        String::from("L1C"), 
                        String::from("L2L"), 
                        String::from("L2W"), 
                        String::from("L5Q"),
                        String::from("S1C"), 
                        String::from("S1W"), 
                        String::from("S2L"), 
                        String::from("S2W"), 
                        String::from("S5Q"),
                    ]);
                codes.insert(
                    Constellation::IRNSS,
                    vec![
                        String::from("C5A"),
                        String::from("D5A"), 
                        String::from("L5A"), 
                        String::from("S5A"),
                    ]);
                codes.insert(
                    Constellation::QZSS,
                    vec![
                        String::from("C1C"), 
                        String::from("C2L"), 
                        String::from("C5Q"), 
                        String::from("D1C"), 
                        String::from("D2L"), 
                        String::from("D5Q"), 
                        String::from("L1C"), 
                        String::from("L2L"), 
                        String::from("L5Q"), 
                        String::from("S1C"), 
                        String::from("S2L"), 
                        String::from("S5Q"),
                    ]);
                codes.insert(
                    Constellation::Glonass,
                    vec![
                        String::from("C1C"), 
                        String::from("C1P"), 
                        String::from("C2C"), 
                        String::from("C2P"), 
                        String::from("C3Q"), 
                        String::from("D1C"), 
                        String::from("D1P"), 
                        String::from("D2C"), 
                        String::from("D2P"), 
                        String::from("D3Q"), 
                        String::from("L1C"), 
                        String::from("L1P"), 
                        String::from("L2C"),
                        String::from("L2P"), 
                        String::from("L3Q"), 
                        String::from("S1C"), 
                        String::from("S1P"), 
                        String::from("S2C"), 
                        String::from("S2P"), 
                        String::from("S3Q"),
                    ]);
                codes.insert(
                    Constellation::SBAS(Augmentation::default()),
                    vec![
                        String::from("C1C"), 
                        String::from("C5I"), 
                        String::from("D1C"), 
                        String::from("D5I"), 
                        String::from("L1C"), 
                        String::from("L5I"), 
                        String::from("S1C"), 
                        String::from("S5I"),
                    ]);
                codes
            },
            clock_offset_applied: false,
        });
    }
} 
