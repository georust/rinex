//! SP3 merging opreations

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use std::path::PathBuf;
    use std::str::FromStr;
    use qc_traits::Merge;

    #[test]
    #[cfg(feature = "qc")]
    #[cfg(feature = "flate2")]
    fn merge_failure() {
        let test_pool = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("SP3");

        let path_a = test_pool
            .clone()
            .join("EMR0OPSULT_20232391800_02D_15M_ORB.SP3.gz");
        let file_a = SP3::from_gzip_file(&path_a);
        assert!(
            file_a.is_ok(),
            "failed to parse EMR0OPSULT_20232391800_02D_15M_ORB.SP3.gz"
        );
        let file_a = file_a.unwrap();

        let path_b = test_pool
            .clone()
            .join("ESA0OPSULT_20232320600_02D_15M_ORB.SP3.gz");
        let file_b = SP3::from_gzip_file(&path_b);
        assert!(
            file_b.is_ok(),
            "failed to parse ESA0OPSULT_20232320600_02D_15M_ORB.SP3.gz"
        );
        let file_b = file_b.unwrap();

        let new = file_a.merge(&file_b);
        assert!(
            new.is_err(),
            "should not be able to merge files from two different data providers"
        );
    }
    
    #[test]
    #[cfg(feature = "qc")]
    #[cfg(feature = "flate2")]
    fn esa0opsrap_esa0opsult_2023() {
        let test_pool = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("SP3");

        let path_a = test_pool
            .clone()
            .join("ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz");
        let file_a = SP3::from_file(&path_a);
        assert!(
            file_a.is_ok(),
            "failed to parse ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz"
        );
        let file_a = file_a.unwrap();

        let path_b = test_pool
            .clone()
            .join("ESA0OPSULT_20232320600_02D_15M_ORB.SP3.gz");
        let file_b = SP3::from_file(&path_b);
        assert!(
            file_b.is_ok(),
            "failed to parse ESA0OPSULT_20232320600_02D_15M_ORB.SP3.gz"
        );
        let file_b = file_b.unwrap();

        let merged = file_a.merge(&file_b);
        assert!(
            merged.is_ok(),
            "failed to merge into ESA0OPSULT_20232320600 ESA0OPSRAP_20232390000, {:?}",
            merged.err()
        );

        let merged = merged.unwrap();
        assert_eq!(merged.nb_epochs(), 192 + 96);
        assert_eq!(
            merged.first_epoch(),
            Some(Epoch::from_str("2023-08-20T06:00:00 GPST").unwrap())
        );
        assert_eq!(
            merged.last_epoch(),
            Some(Epoch::from_str("2023-08-27T23:45:00 GPST").unwrap())
        );
    }
}
