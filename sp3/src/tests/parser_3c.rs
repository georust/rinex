//! SP3-C dedicated tests
#[cfg(test)]
mod test {
    use crate::prelude::*;
    use gnss::prelude::Constellation;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    #[cfg(feature = "flate2")]
    fn esa0opsrap_20232339_01d_15m() {
        let path = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("SP3")
            .join("ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz");

        let sp3 = SP3::from_gzip_file(&path);

        assert!(
            sp3.is_ok(),
            "failed to parse ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz : {:?}",
            sp3.err()
        );

        let sp3 = sp3.unwrap();

        assert_eq!(sp3.header.version, Version::C);
        assert_eq!(sp3.header.data_type, DataType::Position);

        assert!(sp3.has_satellite_clock_offset());
        assert!(!sp3.has_satellite_clock_drift());
        assert!(!sp3.has_satellite_velocity());

        assert_eq!(
            sp3.first_epoch(),
            Epoch::from_str("2023-08-27T00:00:00 GPST").unwrap()
        );

        assert_eq!(sp3.total_epochs(), 96, "bad number of epochs");

        assert_eq!(sp3.header.coord_system, "ITRF2");
        assert_eq!(sp3.header.orbit_type, OrbitType::BHN);
        assert_eq!(sp3.header.time_scale, TimeScale::GPST);
        assert_eq!(sp3.header.constellation, Constellation::Mixed);
        assert_eq!(sp3.header.agency, "ESOC");

        assert_eq!(sp3.header.week_counter, 2277);
        assert_eq!(sp3.header.week_sow, 0.0_f64);
        assert_eq!(sp3.header.mjd, 60183.0);

        assert_eq!(sp3.header.epoch_interval, Duration::from_seconds(900.0_f64));

        // TODO
        // for (index, epoch) in sp3.epochs_iter().enumerate() {
        //     match index {
        //         0 => {
        //             assert_eq!(
        //                 epoch,
        //                 Epoch::from_str("2023-08-27T00:00:00 GPST").unwrap(),
        //                 "parsed wrong epoch"
        //             );
        //         },
        //         1 => {
        //             assert_eq!(
        //                 epoch,
        //                 Epoch::from_str("2023-08-27T00:15:00 GPST").unwrap(),
        //                 "parsed wrong epoch"
        //             );
        //         },
        //         _ => {},
        //     }
        // }

        assert_eq!(
            sp3.comments.len(),
            4,
            "failed to parse files comment correctly"
        );
        assert_eq!(
            sp3.comments,
            vec![
                "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC",
                "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC",
                "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC",
                "PCV:IGS20_2274 OL/AL:EOT11A   NONE     YN ORB:CoN CLK:CoN",
            ],
        );
    }
}
