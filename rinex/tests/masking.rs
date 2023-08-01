#[cfg(test)]
mod test {
    use rinex::preprocessing::*;
    use rinex::*;
    use std::str::FromStr;
    #[test]
    fn v3_duth0630_g01_g02_filter() {
        let mut rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();
        rnx.filter_mut(filter!("G01,G03"));
        assert_eq!(rnx.sv().len(), 2);
    }
    #[test]
    fn v3_duth0630_gps_filter() {
        let mut rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();
        rnx.filter_mut(filter!("GPS"));
        assert_eq!(rnx.sv().len(), 12);
    }
    //#[test]
    //fn v3_duth0630_gps_prn_filter() {
    //    let mut rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();
    //    rnx.filter_mut(filter!(">=G26"));
    //    assert_eq!(rnx.sv().len(), 2);
    //}
    //#[test]
    //fn v2_cari0010_07m_phys_filter() {
    //    let mut rnx = Rinex::from_file("../test_resources/MET/V2/cari0010.07m").unwrap();
    //    let rnx = rnx.filter(filter!("L1C"));
    //    assert_eq!(rnx.observables().len(), 3);
    //    let rnx = rnx.filter(filter!("TD"));
    //    assert_eq!(rnx.observables().len(), 1);
    //}
    //#[test]
    //fn v2_clar0020_00m_phys_filter() {
    //    let mut rnx = Rinex::from_file("../test_resources/MET/V2/clar0020.00m").unwrap();
    //    rnx.filter_mut(filter!("L1C"));
    //    assert_eq!(rnx.observables().len(), 3);
    //    rnx.filter_mut(filter!("PR"));
    //    assert_eq!(rnx.observables().len(), 1);
    //}
    //#[test]
    //fn v2_cari0010_07m_time_filter() {
    //    let mut rnx = Rinex::from_file("../test_resources/MET/V2/cari0010.07m").unwrap();
    //    rnx.filter_mut(filter!(">= 2000-01-02T22:00:00UTC"));
    //}
}
