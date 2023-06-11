#[cfg(test)]
mod test {
    use rinex::{header::*, observation::Record, observation::*, prelude::*, processing::*};
    use std::collections::HashMap;
    use std::str::FromStr;
    #[test]
    fn test_min() {
        /*
         * Test on OBSERVATION RINEX
         */
        let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O");
        assert!(
            rinex.is_ok(),
            "Failed to parse RINEX file \"{}\"",
            "OBS/V3/DUTH0630.22O"
        );
        let rinex = rinex.unwrap();

        let (min_clock, min) = rinex.min();
        assert!(min_clock.is_none());

        let g01_min = min.get(&Sv::from_str("G01").unwrap());
        assert!(
            g01_min.is_some(),
            "dataset.min() is missing results for vehicle G01"
        );
        let g01_min = g01_min.unwrap();
        let g01_c1c_min = g01_min.get(&Observable::from_str("C1C").unwrap());
        assert!(
            g01_c1c_min.is_some(),
            "dataset.min() is missing results for C1C observable"
        );
        let g01_c1c_min = g01_c1c_min.unwrap();
        assert_eq!(*g01_c1c_min, 20243517.560);

        let min = rinex.min_observable();
        let min_s1c_observations = min.get(&Observable::from_str("S1C").unwrap());
        assert!(
            min_s1c_observations.is_some(),
            "dataset.min() is missing results for S1C observable"
        );
        let min_s1c_observations = min_s1c_observations.unwrap();
        assert_eq!(*min_s1c_observations, 37.750);

        /*
         * Test on METEO RINEX
         */
        let rinex = Rinex::from_file("../test_resources/MET/V2/clar0020.00m").unwrap();
        let min = rinex.min_observable();
        for (observable, minimum) in min {
            if observable == Observable::Temperature {
                assert_eq!(minimum, 8.4);
            }
        }
    }
    #[test]
    fn test_max() {
        /*
         * Test on OBSERVATION RINEX
         */
        let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O");
        assert!(
            rinex.is_ok(),
            "Failed to parse RINEX file \"{}\"",
            "OBS/V3/DUTH0630.22O"
        );
        let rinex = rinex.unwrap();

        let (max_clock, max) = rinex.max();
        assert!(max_clock.is_none());

        let g01_max = max.get(&Sv::from_str("G01").unwrap());
        assert!(
            g01_max.is_some(),
            "dataset.max() is missing results for vehicle G01"
        );
        let g01_max = g01_max.unwrap();

        let g01_c1c_max = g01_max.get(&Observable::from_str("C1C").unwrap());
        assert!(
            g01_c1c_max.is_some(),
            "dataset.max() is missing results for C1C observable"
        );
        let g01_c1c_max = g01_c1c_max.unwrap();
        assert_eq!(*g01_c1c_max, 21653418.260);

        let max = rinex.max_observable();
        let max_s1c_observations = max.get(&Observable::from_str("S1C").unwrap());
        assert!(
            max_s1c_observations.is_some(),
            "dataset.max() is missing results for S1C observable"
        );
        let max_s1c_observations = max_s1c_observations.unwrap();
        assert_eq!(*max_s1c_observations, 51.25);

        /*
         * Test on METEO RINEX
         */
        let rinex = Rinex::from_file("../test_resources/MET/V2/clar0020.00m").unwrap();
        let max = rinex.max_observable();
        for (observable, max) in max {
            if observable == Observable::Temperature {
                assert_eq!(max, 16.2);
            }
        }
    }
}
