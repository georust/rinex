#[cfg(test)]
mod test {
    use crate::prelude::*;
    use qc_traits::processing::{Filter, FilterItem, MaskOperand, Preprocessing};
    use std::str::FromStr;
    #[test]
    fn constell_esbcdnk() {
        let rnx =
            Rinex::from_file("../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S.crx.gz")
                .unwrap();

        let mask = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ConstellationItem(vec![Constellation::GPS]),
        );

        let dut = rnx.filter(&mask);
        assert_eq!(dut.constellation().count(), 1, "mask:constel failed");
        assert_eq!(dut.sv().count(), 31, "mask:constel - wrong # of SV");

        let mask = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ConstellationItem(vec![Constellation::SBAS]),
        );

        let dut = rnx.filter(&mask);
        assert_eq!(dut.constellation().count(), 3, "mask:constell(SBAS) failed");
        assert_eq!(dut.sv().count(), 5, "mask:constell(SBAS) failed");
    }
    #[test]
    fn sv_filter_v3_duth0630() {
        let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();
        let mask = Filter::mask(
            MaskOperand::Equals,
            FilterItem::SvItem(vec![
                SV::new(Constellation::GPS, 1),
                SV::new(Constellation::GPS, 3),
            ]),
        );
        let rnx = rnx.filter(&mask);
        assert_eq!(rnx.sv().count(), 2);
    }
    #[test]
    fn gnss_filter_v3_duth0630() {
        let mut rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();
        let mask = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ConstellationItem(vec![Constellation::GPS]),
        );
        rnx.filter_mut(&mask);
        assert_eq!(rnx.sv().count(), 12);
    }
    #[test]
    #[ignore]
    fn v2_cari0010_07m_phys_filter() {
        let rnx = Rinex::from_file("../test_resources/MET/V2/cari0010.07m").unwrap();
        let mask = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ComplexItem(vec!["L1C".to_string()]),
        );
        let dut = rnx.filter(&mask);
        assert_eq!(dut.observable().count(), 0);

        let mask = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ComplexItem(vec!["L1C".to_string()]),
        );
        let dut = rnx.filter(&mask);
        assert_eq!(dut.observable().count(), 1);
    }
    #[test]
    fn v2_clar0020_00m_phys_filter() {
        let rnx = Rinex::from_file("../test_resources/MET/V2/clar0020.00m").unwrap();

        let mask = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ComplexItem(vec!["L1C".to_string()]),
        );
        let dut = rnx.filter(&mask);
        assert_eq!(dut.observable().count(), 0);

        let mask = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ComplexItem(vec!["PR".to_string()]),
        );
        let dut = rnx.filter(&mask);
        assert_eq!(dut.observable().count(), 1);
    }
    #[test]
    fn v2_cari0010_07m_time_filter() {
        let rnx = Rinex::from_file("../test_resources/MET/V2/cari0010.07m").unwrap();

        let mask = Filter::mask(
            MaskOperand::GreaterThan,
            FilterItem::EpochItem(Epoch::from_str("2000-01-02T22:00:00 UTC").unwrap()),
        );
        let dut = rnx.filter(&mask);
        assert_eq!(dut.epoch().count(), 0);

        let mask = Filter::mask(
            MaskOperand::LowerEquals,
            FilterItem::EpochItem(Epoch::from_str("1996-04-02T00:00:00 UTC").unwrap()),
        );
        let dut = rnx.filter(&mask);
        assert_eq!(dut.epoch().count(), 3);

        let mask = Filter::mask(
            MaskOperand::LowerEquals,
            FilterItem::EpochItem(Epoch::from_str("1996-04-01T00:00:30 UTC").unwrap()),
        );
        let dut = rnx.filter(&mask);
        assert_eq!(dut.epoch().count(), 2);

        let mask = Filter::mask(
            MaskOperand::LowerThan,
            FilterItem::EpochItem(Epoch::from_str("1996-04-01T00:00:30 UTC").unwrap()),
        );
        let dut = rnx.filter(&mask);
        assert_eq!(dut.epoch().count(), 1);
    }
}
