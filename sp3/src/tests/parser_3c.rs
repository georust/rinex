//! SP3-C dedicated tests
#[cfg(test)]
mod test {
    use crate::prelude::*;
    use rinex::prelude::Constellation;
    use std::path::PathBuf;
    use std::str::FromStr;
    #[cfg(feature = "flate2")]
    #[test]
    fn esa0opsrap_20232339_01d_15m() {
        let path = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("SP3")
            .join("ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz");

        let sp3 = SP3::from_file(&path.to_string_lossy());
        assert!(
            sp3.is_ok(),
            "failed to parse ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz : {:?}",
            sp3.err()
        );

        let sp3 = sp3.unwrap();

        /*
         * Test general infos
         */
        assert_eq!(sp3.version, Version::C);
        assert_eq!(sp3.data_type, DataType::Position);

        assert_eq!(
            sp3.first_epoch(),
            Some(Epoch::from_str("2023-08-27T00:00:00 GPST").unwrap())
        );

        assert_eq!(sp3.nb_epochs(), 96, "bad number of epochs");
        assert_eq!(sp3.coord_system, "ITRF2");
        assert_eq!(sp3.orbit_type, OrbitType::BHN);
        assert_eq!(sp3.time_system, TimeScale::GPST);
        assert_eq!(sp3.constellation, Constellation::Mixed);
        assert_eq!(sp3.agency, "ESOC");

        assert_eq!(sp3.week_counter, (2277, 0.0_f64));
        assert_eq!(sp3.epoch_interval, Duration::from_seconds(900.0_f64));
        assert_eq!(sp3.mjd_start, (60183, 0.0_f64));

        for (index, epoch) in sp3.epoch().enumerate() {
            match index {
                0 => {
                    assert_eq!(
                        epoch,
                        Epoch::from_str("2023-08-27T00:00:00 GPST").unwrap(),
                        "parsed wrong epoch"
                    );
                },
                1 => {
                    assert_eq!(
                        epoch,
                        Epoch::from_str("2023-08-27T00:15:00 GPST").unwrap(),
                        "parsed wrong epoch"
                    );
                },
                _ => {},
            }
        }

        //for (index, (epoch, sv, clock)) in sp3.sv_clock().enumerate() {}

        /*
         * Test file comments
         */
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
