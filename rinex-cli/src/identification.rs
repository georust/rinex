use rinex::*;

/// Basic file identification
pub fn basic_identification(rnx: &Rinex, ops: Vec<&str>, pretty: bool) {
    for op in ops {
        println!("op: {}", op);
        if op.eq("header") {
            let content = match pretty {
                true => serde_json::to_string_pretty(&rnx.header)
                    .unwrap(),
                false => serde_json::to_string(&rnx.header)
                    .unwrap(),
            };
            println!("{}", content);
        
        } else if op.eq("epochs") {
            let data = &rnx.epochs();
            let content = match pretty {
                true => serde_json::to_string_pretty(data).unwrap(),
                false => serde_json::to_string(data).unwrap(),
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
        
        } else if op.eq("ssi-sv-range") {
            let data = &rnx.observation_ssi_sv_minmax();
            let content = match pretty {
                true => serde_json::to_string_pretty(data).unwrap(),
                false => serde_json::to_string(data).unwrap(),
            };
            println!("{}", content);
        
        } else if op.eq("cycle-slip") {
            let data = &rnx.observation_cycle_slip_epochs();
            let content = match pretty {
                true => serde_json::to_string_pretty(data).unwrap(),
                false => serde_json::to_string(data).unwrap(),
            };
            println!("{}", content);

        } else if op.eq("clock-offset") {
            let data = &rnx.observation_clock_offsets();
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

        } else if op.eq("clock-bias") {
            let data = &rnx.navigation_clock_biases();
            let content = match pretty {
                true => serde_json::to_string_pretty(data).unwrap(),
                false => serde_json::to_string(data).unwrap(),
            };
            println!("{}", content);

        } else if op.eq("gaps") {
            let data = &rnx.data_gaps();
            let content = match pretty {
                true => serde_json::to_string_pretty(data).unwrap(),
                false => serde_json::to_string(data).unwrap(),
            };
            println!("{}", content);

        } else if op.eq("largest-gap") {
            let data = &rnx.largest_data_gap_duration();
            println!("{:?}", data);
        
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
        
        } else if op.eq("lock-loss") {
            let data = &rnx.observation_epoch_lock_loss();
            println!("{:?}", data);
        }
    }
}
