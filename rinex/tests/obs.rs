#[cfg(test)]
mod test {
    use rinex::*;
    use std::str::FromStr;
    use std::process::Command;
    #[test]
    fn v2_aopr0010_17o() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/OBS/V2/aopr0010.17o";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_observation_rinex(), true);
        assert_eq!(rinex.header.obs.is_some(), true);
        assert_eq!(rinex.header.meteo.is_none(), true);
    }
    #[test]
    fn test_is_new_epoch() {
        assert_eq!(        
            observation::is_new_epoch("95 01 01 00 00 00.0000000  0  7 06 17 21 22 23 28 31",
                version::Version {
                    major: 2,
                    minor: 0,
                }
            ),
            true
        );
        assert_eq!(        
            observation::is_new_epoch("21700656.31447  16909599.97044          .00041  24479973.67844  24479975.23247",
                version::Version {
                    major: 2,
                    minor: 0,
                }
            ),
            false
        );
        assert_eq!(        
            observation::is_new_epoch("95 01 01 11 00 00.0000000  0  8 04 16 18 19 22 24 27 29",
                version::Version {
                    major: 2,
                    minor: 0,
                }
            ),
            true
        );
        assert_eq!(        
            observation::is_new_epoch("95 01 01 11 00 00.0000000  0  8 04 16 18 19 22 24 27 29",
                version::Version {
                    major: 3,
                    minor: 0,
                }
            ),
            false 
        );
        assert_eq!(        
            observation::is_new_epoch("> 2022 01 09 00 00 30.0000000  0 40",
                version::Version {
                    major: 3,
                    minor: 0,
                }
            ),
            true 
        );
        assert_eq!(        
            observation::is_new_epoch("> 2022 01 09 00 00 30.0000000  0 40",
                version::Version {
                    major: 2,
                    minor: 0,
                }
            ),
            false
        );
        assert_eq!(        
            observation::is_new_epoch("G01  22331467.880   117352685.28208        48.950    22331469.28",
                version::Version {
                    major: 3,
                    minor: 0,
                }
            ),
            false
        );
    }
}
