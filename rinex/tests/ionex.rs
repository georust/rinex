#[cfg(test)]
mod test {
    use rinex::ionex::*;
    use rinex::epoch::*;
    use rinex::prelude::*;
    #[test]
    fn v1_ckmg0020_22i() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/IONEX/V1/CKMG0020.22I.gz";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_ionex(), true);
        let header = rinex.header.clone();
        assert_eq!(header.version.major, 1);
        assert_eq!(header.version.minor, 0);
        assert_eq!(header.ionex.is_some(), true);
        let header = header
            .ionex
            .as_ref()
            .unwrap();
        let grid = header.grid.clone();
        assert_eq!(grid.height.start, 350.0);
        assert_eq!(grid.height.end, 350.0);
        assert_eq!(rinex.is_ionex_2d(), true);
        assert_eq!(grid.latitude.start, 87.5);
        assert_eq!(grid.latitude.end, -87.5);
        assert_eq!(grid.latitude.spacing, -2.5);
        assert_eq!(grid.longitude.start, -180.0);
        assert_eq!(grid.longitude.end, 180.0);
        assert_eq!(grid.longitude.spacing, 5.0);
        assert_eq!(header.exponent, -1);
        assert_eq!(header.base_radius, 6371.0);
        assert_eq!(header.elevation_cutoff, 0.0);
        assert_eq!(header.mapping, None);

        let record = rinex.record.as_ionex();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        assert_eq!(record.len(), 25);
        
        for (_, (_, rms, h)) in record {
            assert_eq!(h.is_none(), true);
            assert_eq!(rms.is_none(), true);
        }

        let e = Epoch {
            date: str2date("2022 1 2 0 0 0").unwrap(),
            flag: EpochFlag::default(),
        };
        let data = record.get(&e);
        let (tec, _, _) = data.unwrap();
        for p in tec {
            assert_eq!(p.altitude, 350.0);
            if p.latitude == 87.5 {
                if p.longitude == -180.0 {
                    assert_eq!(p.value, 9.2);
                }
                if p.longitude == -175.0 {
                    assert_eq!(p.value, 9.2);
                }
            }
            if p.latitude == 85.0 {
                if p.longitude == -180.0 {
                    assert_eq!(p.value, 9.2);
                }
            }
            if p.latitude == 32.5 {
                if p.longitude == -180.0 {
                    assert_eq!(p.value, 17.7);
                }
                if p.longitude == -175.0 {
                    assert_eq!(p.value, 16.7);
                }
            }
        }
    }
}
