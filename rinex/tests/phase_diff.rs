#[cfg(test)]
mod test {
    use rinex::*;
    #[test]
    fn test_acor00esp_r_2021() {
        let rnx = Rinex::from_file("../test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx");
        let mut rnx = rnx.unwrap();
        rnx.retain_constellation_mut(vec![Constellation::GPS]);
        let data = rnx.observation_phase_diff();
    }
}
