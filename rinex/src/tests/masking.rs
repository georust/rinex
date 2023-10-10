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
    fn gnss_filter_v3_duth0630() {
        let mut rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();
        rnx.filter_mut(filter!("GPS"));
        assert_eq!(rnx.sv().count(), 12);
    }
    #[test]
    fn v2_cari0010_07m_phys_filter() {
        let rnx = Rinex::from_file("../test_resources/MET/V2/cari0010.07m").unwrap();
        let rnx = rnx.filter(filter!("L1C"));
        assert_eq!(rnx.observable().count(), 0);
        let rnx = rnx.filter(filter!("TD"));
        assert_eq!(rnx.observable().count(), 1);
    }
    #[test]
    fn v2_clar0020_00m_phys_filter() {
        let mut rnx = Rinex::from_file("../test_resources/MET/V2/clar0020.00m").unwrap();
        let dut = rnx.filter(filter!("L1C"));
        assert_eq!(dut.observable().count(), 0);
        let dut = rnx.filter(filter!("PR"));
        assert_eq!(dut.observable().count(), 1);
    }
    #[test]
    fn v2_cari0010_07m_time_filter() {
        let rnx = Rinex::from_file("../test_resources/MET/V2/cari0010.07m").unwrap();
        let dut = rnx.filter(filter!(">2000-01-02T22:00:00 UTC"));
        assert_eq!(dut.epoch().count(), 0);
        let dut = rnx.filter(filter!("<=1996-04-02T00:00:00 UTC"));
        assert_eq!(dut.epoch().count(), 3);
        let dut = rnx.filter(filter!("<=1996-04-01T00:00:30 UTC"));
        assert_eq!(dut.epoch().count(), 2);
        let dut = rnx.filter(filter!("< 1996-04-01T00:00:30 UTC"));
        assert_eq!(dut.epoch().count(), 1);
    }
}
