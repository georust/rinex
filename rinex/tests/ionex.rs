#[cfg(test)]
mod test {
    use rinex::prelude::*;
    #[test]
    fn v1_ckmg0020_22i() {
        let test_resource =
            env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/IONEX/V1/CKMG0020.22I.gz";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert!(rinex.is_ionex());
        assert!(rinex.is_2d_ionex());

        let header = rinex.header.clone();
        assert_eq!(header.version.major, 1);
        assert_eq!(header.version.minor, 0);
        assert_eq!(header.ionex.is_some(), true);
        let header = header.ionex.as_ref().unwrap();

        let map_grid = header.map_grid.clone();
        assert_eq!(map_grid.h_grid.start, 350.0);
        assert_eq!(map_grid.h_grid.end, 350.0);

        assert_eq!(map_grid.lat_grid.start, 87.5);
        assert_eq!(map_grid.lat_grid.end, -87.5);
        assert_eq!(map_grid.lat_grid.spacing, -2.5);

        assert_eq!(map_grid.lon_grid.start, -180.0);
        assert_eq!(map_grid.lon_grid.end, 180.0);
        assert_eq!(map_grid.lon_grid.spacing, 5.0);

        assert_eq!(header.exponent, -1);
        assert_eq!(header.base_radius, 6371.0);
        assert_eq!(header.elevation_cutoff, 0.0);
        assert_eq!(header.mapping, None);

        let record = rinex.record.as_ionex();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        assert_eq!(record.len(), 25);

        // epoch [1]
        let e = Epoch::from_gregorian_utc(2022, 1, 2, 0, 0, 0, 0);
        let data = record.get(&e);
        assert!(data.is_some(), "epoch is missing");
        let data = data.unwrap();
        for (z, latitudes) in data {
            assert_eq!(*z, 350.0); // static altitude in this file
            for (lat, longitudes) in latitudes {
                for (lon, tec) in longitudes {
                    // test some values
                    if *lat == 87.5 {
                        if *lon == -180.0 {
                            assert!((*tec - 9.2).abs() < 1E-3);
                        } else if *lon == -175.0 {
                            assert!((*tec - 9.2).abs() < 1E-3);
                        }
                    } else if *lat == 85.0 {
                        if *lon == -180.0 {
                            assert!((*tec - 9.2).abs() < 1E-3);
                        }
                    } else if *lat == 32.5 {
                        //if *lon == -180.0 {
                        //    assert!((*tec - 17.7).abs() < 1E-3, "{}", *tec);
                        //} else
                        if *lon == -175.0 {
                            assert!((*tec - 16.7).abs() < 1E-3, "{}", *tec);
                        }
                    }
                }
            }
        }
        // epoch [N-2]
        let e = Epoch::from_gregorian_utc(2022, 1, 2, 23, 0, 0, 0);
        let data = record.get(&e);
        assert!(data.is_some(), "epoch is missing");
        let data = data.unwrap();
        for (z, latitudes) in data {
            assert_eq!(*z, 350.0); // fixed altitude in this file
            for (lat, longitudes) in latitudes {
                for (lon, tec) in longitudes {
                    if *lat == 87.5 {
                        if *lon == -180.0 {
                            assert!((*tec - 9.2).abs() < 1E-3);
                        } else if *lon == -175.0 {
                            assert!((*tec - 9.2).abs() < 1E-3);
                        }
                    } else if *lat == 27.5 {
                        if *lon == -180.0 {
                            assert!((*tec - 21.6).abs() < 1E-3);
                        } else if *lon == -175.0 {
                            assert!((*tec - 21.4).abs() < 1E-3);
                        }
                    } else if *lat == 25.0 {
                        if *lon == -180.0 {
                            assert!((*tec - 23.8).abs() < 1E-3);
                        } else if *lon == -175.0 {
                            assert!((*tec - 23.8).abs() < 1E-3);
                        } else if *lon == -170.0 {
                            assert!((*tec - 23.2).abs() < 1E-3);
                        } else if *lon == -160.0 {
                            assert!((*tec - 21.8).abs() < 1E-3);
                        }
                    }
                }
            }
        }
    }
}
