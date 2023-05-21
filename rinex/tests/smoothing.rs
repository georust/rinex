#[cfg(test)]
mod test {
    use rinex::{
        //header::*,
        //observation::*,
        observation::Record,
        prelude::*,
        processing::*,
    };
    use std::str::FromStr;
    fn testbench(filter_name: &str, expected: Vec<(&str, &str, Vec<f64>)>, rec: &Record) {
        for (sv, code, dataset) in expected {
            let sv_to_test = Sv::from_str(sv).unwrap();
            let code_to_test = code.to_string();
            for (index, ((epoch, _), (_, svs))) in rec.iter().enumerate() {
                for (sv, observables) in svs {
                    if *sv == sv_to_test {
                        for (observable, observation) in observables {
                            if observable.to_string() == code_to_test {
                                let expected = dataset.get(index).unwrap();
                                assert!((observation.obs - *expected).abs() < 1E-5,
									"{} filter test failed for \"{}\":\"{}\" @ {} - expecting {} got {}", filter_name, sv, observable, epoch, *expected, observation.obs);
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn v3_duth0630_hatch_filter() {
        let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();
        let record = rinex.record.as_obs().unwrap();
        let filter = Filter::from_str("smooth:hatch").unwrap();

        let _filtered = record.filter(filter);

        let results: Vec<(&str, &str, Vec<f64>)> = vec![(
            "g01",
            "C1C",
            vec![
                (20243517.560 - 106380411.418) + 106380411.418,
                (20805393.080 - 109333085.615 + 20243517.560 - 106380411.418) / 2.0 + 109333085.615,
                (21653418.260 - 113789485.670 + 20805393.080 - 109333085.615 + 20243517.560
                    - 106380411.418)
                    / 3.0
                    + 113789485.670,
            ],
        )];
        testbench("hatch", results, &record);
    }
}
