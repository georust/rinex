//! SP3-D dedicated tests
#[cfg(test)]
mod test {
    use crate::prelude::*;
    use rinex::prelude::{Constellation, Sv};
    use rinex::sv;
    use std::path::PathBuf;
    use std::str::FromStr;
    #[test]
    fn sp3d_txt() {
        let path = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("SP3")
            .join("sp3d.txt");
        let sp3 = SP3::from_file(&path.to_string_lossy());
        assert!(
            sp3.is_ok(),
            "failed to parse data/sp3d.txt: {:?}",
            sp3.err()
        );

        let sp3 = sp3.unwrap();

        /*
         * Test general infos
         */
        assert_eq!(sp3.version, Version::D);
        assert_eq!(sp3.data_type, DataType::Position);
        assert_eq!(
            sp3.first_epoch(),
            Some(Epoch::from_str("2019-10-27T00:00:00 GPST").unwrap())
        );
        assert_eq!(sp3.nb_epochs(), 1, "bad number of epochs");
        assert_eq!(sp3.coord_system, "IGS14");
        assert_eq!(sp3.orbit_type, OrbitType::FIT);
        assert_eq!(sp3.time_system, TimeScale::GPST);
        assert_eq!(sp3.constellation, Constellation::Mixed);
        assert_eq!(sp3.agency, "IGS");
        assert_eq!(sp3.week_counter, (2077, 0.0_f64));
        assert_eq!(sp3.epoch_interval, Duration::from_seconds(300.0_f64));
        assert_eq!(sp3.mjd_start, (58783, 0.0_f64));

        for (epoch, sv, position) in sp3.sv_position() {
            assert_eq!(epoch, Epoch::from_str("2019-10-27T00:00:00 GPST").unwrap());
            if sv == sv!("C01") {
                assert_eq!(
                    position,
                    (-32312.652253, 27060.656563, 205.195454),
                    "bad position data"
                );
            } else if sv == sv!("E01") {
                assert_eq!(
                    position,
                    (-15325.409333, 5781.454973, -24645.410980),
                    "bad position data"
                );
            } else if sv == sv!("G01") {
                assert_eq!(
                    position,
                    (-22335.782004, -14656.280389, -1218.238499),
                    "bad position data"
                );
            } else if sv == sv!("J01") {
                assert_eq!(
                    position,
                    (-30616.656355, 26707.752269, 16227.934145),
                    "bad position data"
                );
            } else if sv == sv!("R01") {
                assert_eq!(
                    position,
                    (15684.717752, -12408.390324, -15847.221180),
                    "bad position data"
                );
            } else {
                panic!("identified wrong sv");
            }
        }

        let mut clk: Vec<_> = sp3.sv_clock().collect();
        for (epoch, sv, clock) in sp3.sv_clock() {
            assert_eq!(epoch, Epoch::from_str("2019-10-27T00:00:00 GPST").unwrap());
            if sv == sv!("C01") {
                assert_eq!(clock, 63.035497, "bad clock data");
            } else if sv == sv!("E01") {
                assert_eq!(clock, -718.927492, "bad clock data");
            } else if sv == sv!("G01") {
                assert_eq!(clock, -176.397152, "bad clock data");
            } else if sv == sv!("J01") {
                assert_eq!(clock, -336.145158, "bad clock data");
            } else if sv == sv!("R01") {
                assert_eq!(clock, 51.759894, "bad clock data");
            } else {
                panic!("identified wrong sv");
            }
        }
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
                "PCV:IGS14_2074 OL/AL:FES2004  NONE     YN CLK:CoN ORB:CoN",
                "THIS EXAMPLE OF SP3 FILE IS PART OF THE gLAB TOOL SUITE",
                "FILE PREPARED BY: MOWEN LI",
                "PLEASE EMAIL ANY COMMENT OR REQUEST TO glab.gage @upc.edu",
            ],
        );
    }
}
