//! SP3 interpolation specific tests
#[cfg(test)]
mod test {
    use crate::prelude::*;
    use std::path::PathBuf;
    use std::str::FromStr;

    use crate::{lagrange_interpolation, Vector3D};

    // fn max_error(values: Vec<(Epoch, f64)>, epoch: Epoch, order: usize) -> f64 {
    //     let mut q = 1.0_f64;
    //     for (e, _) in values {
    //         q *= (epoch - e).to_seconds();
    //     }
    //     let factorial: usize = (1..=order + 1).product();
    //     q.abs() / factorial as f64 // TODO f^(n+1)[x]
    // }

    #[test]
    #[cfg(feature = "flate2")]
    fn interpolation_feasibility() {
        let path = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("SP3")
            .join("EMR0OPSULT_20232391800_02D_15M_ORB.SP3.gz");

        let sp3 = SP3::from_gzip_file(&path);

        assert!(
            sp3.is_ok(),
            "failed to parse EMR0OPSULT_20232391800_02D_15M_ORB.SP3.gz"
        );

        let sp3 = sp3.unwrap();

        let g01 = SV::from_str("G01").unwrap();
        let g72 = SV::from_str("G72").unwrap();

        let t0 = Epoch::from_str("2023-08-27T18:00:00 GPST").unwrap();
        let t0_5m = Epoch::from_str("2023-08-27T18:05:00 GPST").unwrap();
        let t1 = Epoch::from_str("2023-08-27T18:15:00 GPST").unwrap();
        let t2 = Epoch::from_str("2023-08-27T18:30:00 GPST").unwrap();
        let t2_10m = Epoch::from_str("2023-08-27T18:40:00 GPST").unwrap();
        let tn2 = Epoch::from_str("2023-08-29T17:15:00 GPST").unwrap();
        let tn2_10m = Epoch::from_str("2023-08-29T17:25:00 GPST").unwrap();
        let tn1 = Epoch::from_str("2023-08-29T17:30:00 GPST").unwrap();
        let tn1_10m = Epoch::from_str("2023-08-29T17:40:00 GPST").unwrap();
        let tn = Epoch::from_str("2023-08-29T17:45:00 GPST").unwrap();

        // test: invalid SV
        assert!(sp3
            .satellites_position_interpolate(g72, t0, 7, lagrange_interpolation)
            .is_err());
        assert!(sp3
            .satellites_position_interpolate(g72, t1, 7, lagrange_interpolation)
            .is_err());
        assert!(sp3
            .satellites_position_interpolate(g72, t2, 7, lagrange_interpolation)
            .is_err());
        assert!(sp3
            .satellites_position_interpolate(g72, t2_10m, 7, lagrange_interpolation)
            .is_err());
        assert!(sp3
            .satellites_position_interpolate(g72, tn, 7, lagrange_interpolation)
            .is_err());

        // test: even order
        let even_err = sp3
            .satellites_position_interpolate(g72, t0, 2, lagrange_interpolation)
            .err();
        match even_err {
            Some(Error::EvenInterpolationOrder) => {},
            _ => panic!("invalid error"),
        }

        // test: too early (x3)
        for t in [t0, t0_5m] {
            let error = sp3
                .satellites_position_interpolate(g01, t, 3, lagrange_interpolation)
                .err();
            match error {
                Some(Error::InterpolationWindow) => {},
                _ => panic!("invalid error"),
            }
        }

        // test: too late (x3)
        for t in [tn] {
            let error = sp3
                .satellites_position_interpolate(g01, t, 3, lagrange_interpolation)
                .err();
            match error {
                Some(Error::InterpolationWindow) => {},
                _ => panic!("invalid error"),
            }
        }
    }
}
