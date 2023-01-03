#[cfg(test)]
mod test {
    use rinex::{
		prelude::*,
		header::*,
		observation::*,
		observation::Record,
		processing::*,
	};
    use std::str::FromStr;
	use std::collections::HashMap;
	fn testbench(test: &str, expected: Vec<(&str, &str, f64)>, data: &HashMap<Sv, HashMap<Observable, f64>>) {
		for (sv, code, value) in expected {
			let sv_to_test = Sv::from_str(sv)
				.unwrap();
			let code_to_test = code.to_string();
			for (sv, observables) in data {
				if *sv == sv_to_test {
					for (observable, data) in observables {
						if observable.to_string() == code_to_test {
							assert!(
								(value - *data).abs() < 1.0E-3, 
								"{} test failed for \"{}\":\"{}\" - expecting {} got {}", test, sv, observable, value, data);
						}
					}
				}
			}
		}
	}
	#[test]
	fn stats_v3_duth0630() {
        let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
			.unwrap();
		let record = rinex.record.as_obs().unwrap();
			
		/*
		G01 C1C 20805393.080 20243517.560 21653418.260 
		G01 L1C 106380411.418  109333085.615 113789485.670
		G03 C1C 20619020.680 20425456.580 20410261.460
		G03 L1C 108353702.797 107336517.682 107256666.317
		R23 D1C -2835.609 -3870.441 -4464.453  
		R23 S1C 50.000 48.000 44.250
		R24 D1C 865.820 -700.187 -2188.113
		R24 S1C 51.000 51.250 51.000
		*/
		let (_, min) = record.min();
		let results: Vec<(&str, &str, f64)> = vec![
			("g01", "C1C", 20243517.560),
			("g01", "L1C", 106380411.418),
			("g03", "C1C", 20410261.460),
			("g03", "L1C", 107256666.317),
			("R23", "D1C", -4464.453),
			("R23", "S1C", 44.250),
			("R24", "D1C", -2188.113),
			("R24", "S1C", 51.000),
		];
		testbench("min()", results, &min);
		
		/*
		G01 C1C 20805393.080 20243517.560 21653418.260 
		G01 L1C 106380411.418  109333085.615 113789485.670
		G03 C1C 20619020.680 20425456.580 20410261.460
		G03 L1C 108353702.797 107336517.682 107256666.317
		R23 D1C -2835.609 -3870.441 -4464.453  
		R23 S1C 50.000 48.000 44.250
		R24 D1C 865.820 -700.187 -2188.113
		R24 S1C 51.000 51.250 51.000
		*/
		let (_, max) = record.max();
		let results: Vec<(&str, &str, f64)> = vec![
			("g01", "C1C", 21653418.260),
			("g01", "L1C", 113789485.670),
			("g03", "C1C", 20619020.680),
			("g03", "L1C", 108353702.797),
			("R23", "D1C", -2835.609),
			("R23", "S1C", 50.000),
			("R24", "D1C", 865.820),
			("R24", "S1C", 51.250),
		];
		testbench("max()", results, &max);
		
		/*
		G01 C1C 20805393.080 20243517.560 21653418.260 
		G01 L1C 106380411.418  109333085.615 113789485.670
		G03 C1C 20619020.680 20425456.580 20410261.460
		G03 L1C 108353702.797 107336517.682 107256666.317
		R23 D1C -2835.609 -3870.441 -4464.453  
		R23 S1C 50.000 48.000 44.250
		R24 D1C 865.820 -700.187 -2188.113
		R24 S1C 51.000 51.250 51.000
		*/
		let (_, mean) = record.mean();
		let results: Vec<(&str, &str, f64)> = vec![
			("g01", "C1C", 20900776.3),
			("g01", "L1C", 109834327.5676),
			("g03", "C1C", 20484912.9066),
			("g03", "L1C", 107648962.265333),
			("R23", "D1C", -3723.501),
			("R23", "S1C", 47.4166),
			("R24", "D1C", -674.16),
			("R24", "S1C", 51.0833),
		];
		testbench("mean()", results, &mean);
		
		/*
		G01 C1C 20805393.080 20243517.560 21653418.260 
		G01 L1C 106380411.418  109333085.615 113789485.670
		G03 C1C 20619020.680 20425456.580 20410261.460
		G03 L1C 108353702.797 107336517.682 107256666.317
		R23 D1C -2835.609 -3870.441 -4464.453  
		R23 S1C 50.000 48.000 44.250
		R24 D1C 865.820 -700.187 -2188.113
		R24 S1C 51.000 51.250 51.000
		*/
		let (_, stddev) = record.stddev();
		let results: Vec<(&str, &str, f64)> = vec![
			("g01", "C1C", 335852309972.1992_f64.sqrt()),
			("g01", "L1C", 9274685292831.4397042266_f64.sqrt()),
			("g03", "C1C", 9030929379.51475556_f64.sqrt()),
			("g03", "L1C", 249392315235.63520555555566_f64.sqrt()),
			("R23", "D1C", 452984.477856_f64.sqrt()),
			("R23", "S1C", 5.68055_f64.sqrt()),
			("R24", "D1C", 1554756.49711266666666666667_f64.sqrt()),
			("R24", "S1C", 0.01388_f64.sqrt()),
		];
		testbench("stddev()", results, &stddev);
		
		/*
		G01 C1C 20805393.080 20243517.560 21653418.260 
		G01 L1C 106380411.418  109333085.615 113789485.670
		G03 C1C 20619020.680 20425456.580 20410261.460
		G03 L1C 108353702.797 107336517.682 107256666.317
		R23 D1C -2835.609 -3870.441 -4464.453  
		R23 S1C 50.000 48.000 44.250
		R24 D1C 865.820 -700.187 -2188.113
		R24 S1C 51.000 51.250 51.000
		*/
		let der = record.derivative();
		/*let results: Vec<(&str, &str, Vec<f64>)> = vec![
			("g01", "C1C", vec![
			("g01", "L1C", 
			("g03", "C1C", 
			("g03", "L1C", 
			("R23", "D1C", 
			("R23", "S1C", 
			("R24", "D1C", 
			("R24", "S1C", 
		];
		testbench("derivative()", results, &der);*/
	}
}
