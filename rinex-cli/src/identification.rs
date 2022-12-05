use rinex::*;

/// Basic file identification
pub fn basic_identification(rnx: &Rinex, ops: Vec<&str>, pretty: bool) {
    for op in ops {
        if op.eq("header") {
            let content = match pretty {
                true => serde_json::to_string_pretty(&rnx.header).unwrap(),
                false => serde_json::to_string(&rnx.header).unwrap(),
            };
            println!("{}", content);
        } else if op.eq("epoch") {
            let data: Vec<String> = rnx.epochs().iter().map(|e| e.to_string()).collect();
            let content = match pretty {
                true => serde_json::to_string_pretty(&data).unwrap(),
                false => serde_json::to_string(&data).unwrap(),
            };
            println!("{}", content);
        } else if op.eq("sv") {
            let data = &rnx.space_vehicules();
            let content = match pretty {
                true => serde_json::to_string_pretty(data).unwrap(),
                false => serde_json::to_string(data).unwrap(),
            };
            println!("{}", content);
        } else if op.eq("sv-epoch") {
            let data = &rnx.space_vehicules_per_epoch();
            let content = match pretty {
                true => serde_json::to_string_pretty(data).unwrap(),
                false => serde_json::to_string(data).unwrap(),
            };
            println!("{}", content);
        } else if op.eq("observables") {
            let data = &rnx.observables();
            let content = match pretty {
                true => serde_json::to_string_pretty(data).unwrap(),
                false => serde_json::to_string(data).unwrap(),
            };
            println!("{}", content);
        } else if op.eq("constellations") {
            let data = &rnx.list_constellations();
            let content = match pretty {
                true => serde_json::to_string_pretty(data).unwrap(),
                false => serde_json::to_string(data).unwrap(),
            };
            println!("{}", content);
        } else if op.eq("ssi-range") {
            let data = &rnx.observation_ssi_minmax();
            let content = match pretty {
                true => serde_json::to_string_pretty(data).unwrap(),
                false => serde_json::to_string(data).unwrap(),
            };
            println!("{}", content);
        } else if op.eq("orbits") {
            let data = &rnx.observables();
            let content = match pretty {
                true => serde_json::to_string_pretty(data).unwrap(),
                false => serde_json::to_string(data).unwrap(),
            };
            println!("{}", content);
        } else if op.eq("elevation") {
            let data = &rnx.orbits_elevation_angles();
            let content = match pretty {
                true => serde_json::to_string_pretty(data).unwrap(),
                false => serde_json::to_string(data).unwrap(),
            };
            println!("{}", content);
        } else if op.eq("nav-msg") {
            let data = &rnx.navigation_message_types();
            println!("{:?}", data);
        }
    }
}
