#[cfg(test)]
mod test {
    use crate::filter;
    use crate::prelude::*;
    use crate::preprocessing::*;
    use std::str::FromStr;
    #[test]
    fn sv_filter_v3_duth0630() {
        let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();
        let rnx = rnx.filter(filter!("G01,G03"));
        assert_eq!(rnx.sv().count(), 2);
    }
    #[test]
    #[ignore]
    fn gnss_filter_v3_duth0630() {
        let mut rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();
        rnx.filter_mut(filter!("GPS"));
        assert_eq!(rnx.sv().count(), 12);
    }
    #[test]
    #[ignore]
    fn v2_cari0010_07m_phys_filter() {
        let mut rnx = Rinex::from_file("../test_resources/MET/V2/cari0010.07m").unwrap();
        let rnx = rnx.filter(filter!("L1C"));
        assert_eq!(rnx.observable().count(), 3);
        let rnx = rnx.filter(filter!("TD"));
        assert_eq!(rnx.observable().count(), 1);
    }
    #[test]
    #[ignore]
    fn v2_clar0020_00m_phys_filter() {
        let mut rnx = Rinex::from_file("../test_resources/MET/V2/clar0020.00m").unwrap();
        rnx.filter_mut(filter!("L1C"));
        assert_eq!(rnx.observable().count(), 3);
        rnx.filter_mut(filter!("PR"));
        assert_eq!(rnx.observable().count(), 1);
    }
    #[test]
    #[ignore]
    fn v2_cari0010_07m_time_filter() {
        let mut rnx = Rinex::from_file("../test_resources/MET/V2/cari0010.07m").unwrap();
        rnx.filter_mut(filter!(">= 2000-01-02T22:00:00UTC"));
    }
}
