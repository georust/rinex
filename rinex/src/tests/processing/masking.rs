#[cfg(test)]
mod test {
    use crate::prelude::*;
    use itertools::Itertools;
    use qc_traits::{Filter, FilterItem, MaskOperand, Preprocessing};
    use std::str::FromStr;
    #[test]
    fn obs_gnss_v3_esbcdnk() {
        let rnx =
            Rinex::from_file("../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
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
    fn obs_sv_v3_duth0630() {
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
    fn obs_gnss_v3_duth0630() {
        let mut rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();
        let mask = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ConstellationItem(vec![Constellation::GPS]),
        );
        rnx.filter_mut(&mask);
        assert_eq!(rnx.sv().count(), 12);
    }
    #[test]
    fn meteo_obsrv_v2_clar0020() {
        let rnx = Rinex::from_file("../test_resources/MET/V2/clar0020.00m").unwrap();

        let pressure = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ComplexItem(vec!["PR".to_string()]),
        );

        let dut = rnx.filter(&pressure);
        assert_eq!(dut.observable().count(), 1);

        let temp = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ComplexItem(vec!["TD".to_string()]),
        );
        let dut = rnx.filter(&temp);
        assert_eq!(dut.observable().count(), 1);

        let moist = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ComplexItem(vec!["HR".to_string()]),
        );
        let dut = rnx.filter(&moist);
        assert_eq!(dut.observable().count(), 1);

        let none = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ComplexItem(vec!["L1C".to_string()]),
        );
        let dut = rnx.filter(&none);
        assert_eq!(dut.observable().count(), 0);
    }
    #[test]
    fn obs_observ_v3_duth0630() {
        let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();

        let total = rinex.observable().count();
        assert_eq!(total, 12);

        let pr_l1 = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ComplexItem(vec!["C1C".to_string()]),
        );
        let rnx = rinex.filter(&pr_l1);
        assert_eq!(rnx.observable().count(), 1);

        let pr_dop_l1 = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ComplexItem(vec!["C1C".to_string(), "D1C".to_string()]),
        );
        let rnx = rinex.filter(&pr_dop_l1);
        assert_eq!(rnx.observable().count(), 2);

        let pr_l1_dop_l2 = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ComplexItem(vec!["C1C".to_string(), "D2W".to_string()]),
        );
        let rnx = rinex.filter(&pr_l1_dop_l2);
        assert_eq!(rnx.observable().count(), 2);

        let not_pr_l1 = Filter::mask(
            MaskOperand::NotEquals,
            FilterItem::ComplexItem(vec!["C1C".to_string()]),
        );
        let rnx = rinex.filter(&not_pr_l1);
        assert_eq!(rnx.observable().count(), total - 1);

        let not_pr_l1_dop_l2 = Filter::mask(
            MaskOperand::NotEquals,
            FilterItem::ComplexItem(vec!["C1C".to_string(), "D2W".to_string()]),
        );
        let rnx = rinex.filter(&not_pr_l1_dop_l2);
        assert_eq!(rnx.observable().count(), total - 2);
    }
    #[test]
    fn meteo_time_v2_cari0010() {
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
    #[test]
    fn obs_epoch_v3_vlns0630() {
        let rinex = Rinex::from_file("../test_resources/CRNX/V3/VLNS0630.22D").unwrap();

        let before = Filter::greater_than("2022-03-03T00:00:00 GPST").unwrap();
        let first_eq = Filter::greater_equals("2022-03-04T00:00:00 GPST").unwrap();
        let second = Filter::greater_equals("2022-03-04T00:00:30 GPST").unwrap();
        let dut = rinex.filter(&before);
        assert_eq!(dut.epoch().count(), 2);
        let dut = rinex.filter(&first_eq);
        assert_eq!(dut.epoch().count(), 2);
        let dut = rinex.filter(&second);
        assert_eq!(dut.epoch().count(), 1);
    }
    #[test]
    #[cfg(feature = "flate2")]
    fn obs_epoch_v3_esbc00dnk() {
        let rinex =
            Rinex::from_file("../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
                .unwrap();
        let last_eq = Filter::equals("2020-06-25T23:59:30 GPST").unwrap();
        let last_geq = Filter::greater_equals("2020-06-25T23:59:30 GPST").unwrap();
        let last_gt = Filter::greater_than("2020-06-25T23:59:30 GPST").unwrap();
        let dut = rinex.filter(&last_eq);
        assert_eq!(dut.epoch().count(), 1);
        let dut = rinex.filter(&last_geq);
        assert_eq!(dut.epoch().count(), 1);
        let dut = rinex.filter(&last_gt);
        assert_eq!(dut.epoch().count(), 0);
    }
    #[test]
    fn obs_signals_v3_duth0630() {
        let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();

        let carriers = rinex.carrier_iter().sorted().collect::<Vec<_>>();
        assert_eq!(
            carriers,
            vec![
                Carrier::L1,
                Carrier::L2,
                Carrier::G1(None),
                Carrier::G2(None),
            ]
        );

        let l1_only = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ComplexItem(vec!["C1C".to_string(), "D1C".to_string()]),
        );
        let dut = rinex.filter(&l1_only);
        assert_eq!(dut.constellation().count(), 2);

        let carriers = dut.carrier_iter().sorted().collect::<Vec<_>>();
        assert_eq!(carriers, vec![Carrier::L1, Carrier::G1(None)]);

        let glo_l2_only = Filter::mask(
            MaskOperand::Equals,
            FilterItem::ComplexItem(vec!["D2P".to_string()]),
        );
        let dut = rinex.filter(&glo_l2_only);
        assert_eq!(dut.constellation().count(), 1);

        let carriers = dut.carrier_iter().sorted().collect::<Vec<_>>();
        assert_eq!(carriers, vec![Carrier::G2(None)]);
    }
}
