use crate::{Cli, Context};
use rinex::*;

/*
 * Basic identification operations
 */
pub fn rinex_identification(ctx: &Context, cli: &Cli) {
    let pretty = cli.pretty();
    let ops = cli.identification_ops();
    identification(&ctx.primary_rinex, pretty, ops.clone());

    if let Some(nav) = &ctx.nav_rinex {
        identification(&nav, pretty, ops.clone());
    }
}

fn identification(rnx: &Rinex, pretty: bool, ops: Vec<&str>) {
    for op in ops {
        if op.eq("header") {
            let content = match pretty {
                true => serde_json::to_string_pretty(&rnx.header).unwrap(),
                false => serde_json::to_string(&rnx.header).unwrap(),
            };
            println!("{}", content);
        } else if op.eq("epochs") {
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
        } else if op.eq("nav-msg") {
            let data = &rnx.navigation_message_types();
            println!("{:?}", data);
        } else if op.eq("anomalies") {
            let data = &rnx.observation_epoch_anomalies();
            println!("{:#?}", data);
        }
    }
}
