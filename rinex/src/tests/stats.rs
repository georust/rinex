#[cfg(test)]
mod test {
    use crate::*;
    use crate::{observation::*, prelude::*};
    use std::str::FromStr;
    fn run_test(rinex: &Rinex, ops: &str, expected: Vec<(Sv, Vec<(Observable, f64)>)>) {
        let (clk_stats, stats) = match ops {
            "min" => rinex.min(),
            "max" => rinex.max(),
            "mean" => rinex.mean(),
            _ => panic!("unknown test"),
        };
        assert!(clk_stats.is_none()); // always none on our test pool

        for (sv, fields) in expected {
            let sv_data = stats.get(&sv);
            assert!(
                sv_data.is_some(),
                "{}",
                format!("\"{}\" stats missing for vehicle \"{}\"", ops, sv)
            );
            let sv_data = sv_data.unwrap();
            for (observable, expected) in fields {
                let data = sv_data.get(&observable);
                assert!(
                    data.is_some(),
                    "{}",
                    format!(
                        "\"{}\" stats missing for vehicle \"{}\" - \"{}\"",
                        ops, sv, observable
                    )
                );
                let data = data.unwrap();
                let e = *data - expected;
                assert!(
                    e.abs() < 1E-5,
                    "{}",
                    format!(
                        "\"{}\" test failed for \"{}\" - \"{}\", expecting {} got {}",
                        ops, sv, observable, expected, *data
                    )
                );
            }
        }
    }
    fn run_obsv_test(rinex: &Rinex, ops: &str, expected: Vec<(Observable, f64)>) {
        let stats = match ops {
            "min" => rinex.min_observable(),
            "max" => rinex.max_observable(),
            "mean" => rinex.mean_observable(),
            _ => panic!("unknown test"),
        };

        for (observable, expected) in expected {
            let results = stats.get(&observable);
            assert!(
                results.is_some(),
                "{}",
                format!(
                    "\"{}\" observable stats results missing for observable \"{}\"",
                    ops, observable
                )
            );
            let results = results.unwrap();
            let err = *results - expected;
            assert!(
                err.abs() < 1E-5,
                "{}",
                format!(
                    "\"{}\" observable test failed for - \"{}\", expecting {} - got {}",
                    ops, observable, expected, results
                )
            );
        }
    }
    #[test]
    fn stats() {
        let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();

        let expected: Vec<(Sv, Vec<(Observable, f64)>)> = vec![
            (
                sv!("g01"),
                vec![
                    (
                        observable!("c1c"),
                        (20243517.560 + 20805393.080 + 21653418.260) / 3.0,
                    ),
                    (observable!("d1c"), (-1242.766 - 2193.055 - 2985.516) / 3.0),
                    (observable!("s1c"), (51.25 + 50.75 + 49.5) / 3.0),
                ],
            ),
            (
                sv!("g03"),
                vec![
                    (
                        observable!("c1c"),
                        (20619020.680 + 20425456.580 + 20410261.460) / 3.0,
                    ),
                    (observable!("d1c"), (852.785 + 328.797 - 244.031) / 3.0),
                    (observable!("s1c"), (50.75 + 51.0 + 51.0) / 3.0),
                ],
            ),
        ];
        run_test(&rinex, "mean", expected);

        let expected: Vec<(Sv, Vec<(Observable, f64)>)> = vec![
            (
                sv!("g01"),
                vec![
                    (observable!("c1c"), 20243517.560),
                    (observable!("d1c"), -2985.516),
                    (observable!("s1c"), 49.5),
                ],
            ),
            (
                sv!("g03"),
                vec![
                    (observable!("c1c"), 20410261.460),
                    (observable!("d1c"), -244.031),
                    (observable!("s1c"), 50.75),
                ],
            ),
        ];
        run_test(&rinex, "min", expected);

        let expected: Vec<(Sv, Vec<(Observable, f64)>)> = vec![
            (
                sv!("g01"),
                vec![
                    (observable!("c1c"), 21653418.260),
                    (observable!("d1c"), -1242.766),
                    (observable!("s1c"), 51.25),
                ],
            ),
            (
                sv!("g03"),
                vec![
                    (observable!("c1c"), 20619020.680),
                    (observable!("d1c"), 852.785),
                    (observable!("s1c"), 51.0),
                ],
            ),
        ];
        run_test(&rinex, "max", expected);

        /* *********************************************
         * STATS FOR ALL SV / ALL OBS : meteo compatible
         **********************************************/
        /*let expected : Vec<(Observable, f64)> = vec![
            (observable!("c1c"), (20243517.560 + 20619020.680 + 21542633.500 + 24438727.980 + 22978068.560 + 23460759.840 + 21923317.180 + 23434790.440 + 22401985.340 + 24991723.280 + 19727826.340 + 23171275.620 + 20662538.580 + 23450513.820 +  23044984.180 + 22909354.040 + 20116780.920 + 19708379.260 + 20805393.080 + 20425456.580 + 20887001.400 + 23371156.300 + 23031543.660 + 23117350.280 + 22726604.680 + 24425563.640 + 22689941.780 + 19677287.000 + 22265147.080 + 21462395.740 + 23740237.340 + 22432243.520 + 21750541.080 + 21199384.320 + 19680274.400 + 21653418.260 + 20410261.460 + 20488105.720 + 23647940.540 + 22436978.380 + 23392660.200 + 23154069.760 + 23689895.760 + 25161827.280 + 23333751.720 + 19831816.600 + 21490078.220 + 22328018.960 + 22235350.560 + 20915624.780 + 22543866.020 + 20147683.700) / 52.0),
        ];
        run_obsv_test(&rinex, "mean", expected);*/

        /*
         * Test on METEO RINEX
         */
        let rinex = Rinex::from_file("../test_resources/MET/V2/clar0020.00m").unwrap();
        let expected: Vec<(Observable, f64)> = vec![(
            observable!("PR"),
            (970.5
                + 970.4
                + 970.4
                + 970.3
                + 970.2
                + 970.3
                + 970.1
                + 970.3
                + 970.2
                + 970.2
                + 972.1
                + 972.3
                + 972.4
                + 972.6
                + 972.8
                + 973.1
                + 973.3
                + 973.4
                + 973.6
                + 973.6
                + 973.7
                + 973.7
                + 973.5
                + 973.6
                + 973.6
                + 973.6
                + 973.6
                + 973.5
                + 973.5
                + 973.4
                + 973.3
                + 973.2
                + 972.9
                + 972.8
                + 972.7
                + 972.5
                + 972.4
                + 972.4
                + 972.2
                + 972.2
                + 972.2
                + 972.3
                + 972.2
                + 972.2
                + 972.1
                + 972.2
                + 972.1
                + 972.2
                + 972.2
                + 972.2
                + 972.1
                + 972.2
                + 972.2
                + 972.3
                + 972.3
                + 972.5
                + 972.5)
                / 57.0,
        )];
        run_obsv_test(&rinex, "mean", expected);

        let expected: Vec<(Observable, f64)> = vec![(
            observable!("TD"),
            (10.7
                + 10.6
                + 10.4
                + 10.3
                + 10.1
                + 9.9
                + 9.8
                + 9.5
                + 9.6
                + 9.6
                + 8.4
                + 10.1
                + 10.1
                + 10.3
                + 10.2
                + 10.3
                + 10.6
                + 10.8
                + 11.1
                + 11.2
                + 11.4
                + 11.5
                + 11.8
                + 11.8
                + 11.8
                + 12.7
                + 12.7
                + 13.2
                + 13.5
                + 13.0
                + 13.5
                + 13.9
                + 14.0
                + 14.7
                + 15.0
                + 14.8
                + 15.6
                + 15.1
                + 14.7
                + 14.8
                + 14.9
                + 14.9
                + 15.9
                + 15.6
                + 16.2
                + 15.6
                + 15.5
                + 15.9
                + 15.8
                + 15.7
                + 15.7
                + 15.6
                + 15.4
                + 14.9
                + 14.6
                + 14.3
                + 14.2)
                / 57.0,
        )];
        run_obsv_test(&rinex, "mean", expected);

        let expected: Vec<(Observable, f64)> = vec![(
            observable!("HR"),
            (71.4
                + 72.2
                + 72.9
                + 72.9
                + 75.0
                + 76.7
                + 78.1
                + 79.7
                + 80.5
                + 79.1
                + 70.7
                + 65.3
                + 66.4
                + 68.0
                + 68.4
                + 68.6
                + 66.8
                + 65.4
                + 63.3
                + 62.7
                + 60.0
                + 57.2
                + 56.0
                + 54.9
                + 52.2
                + 49.3
                + 46.3
                + 43.8
                + 44.8
                + 42.7
                + 41.2
                + 40.2
                + 37.9
                + 36.4
                + 37.3
                + 36.1
                + 32.7
                + 31.6
                + 36.1
                + 35.7
                + 34.0
                + 32.6
                + 29.5
                + 28.0
                + 26.7
                + 25.5
                + 25.0
                + 30.7
                + 25.0
                + 23.5
                + 27.7
                + 28.2
                + 28.4
                + 30.9
                + 33.3
                + 33.3
                + 33.2)
                / 57.0,
        )];
        run_obsv_test(&rinex, "mean", expected);
    }
}
