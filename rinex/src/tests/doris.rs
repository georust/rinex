#[cfg(test)]
mod test {
    use crate::tests::toolkit::doris_check_observables;
    use crate::{erratic_time_frame, evenly_spaced_time_frame, tests::toolkit::TestTimeFrame};
    use crate::{observation::*, prelude::*};
    use itertools::Itertools;
    use std::path::Path;
    use std::str::FromStr;
    #[test]
    fn v3_cs2rx18164() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("DORIS")
            .join("V3")
            .join("cs2rx18164.001");
        let fullpath = path.to_string_lossy();
        let rinex = Rinex::from_file(fullpath.as_ref());
        //assert!(rinex.is_ok());

        let rinex = rinex.unwrap();

        doris_check_observables(
            &rinex,
            &["L1", "L2", "C1", "C2", "W1", "W2", "F", "P", "T", "H"],
        );
    }
}
