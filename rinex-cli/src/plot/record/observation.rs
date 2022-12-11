use crate::plot::{generate_markers, Context};
use plotly::{
    common::{Marker, MarkerSymbol, Mode, Visible},
    Scatter,
};
use rinex::{navigation::*, observation::*, prelude::*, *};
use std::collections::HashMap;

macro_rules! code2physics {
    ($code: expr) => {
        if is_phase_carrier_obs_code!($code) {
            "Phase".to_string()
        } else if is_doppler_obs_code!($code) {
            "Doppler".to_string()
        } else if is_sig_strength_obs_code!($code) {
            "Signal Strength".to_string()
        } else {
            "Pseudo Range".to_string()
        }
    };
}

/*
 * Plots given Observation RINEX content
 */
pub fn plot_observation(ctx: &mut Context, record: &observation::Record, nav_ctx: &Option<Rinex>) {
    if let Some(nav) = nav_ctx {
        //enhanced_plot(record, nav);
        basic_plot(ctx, record);
    } else {
        basic_plot(ctx, record);
    }
}

pub fn basic_plot(ctx: &mut Context, record: &observation::Record) {
    let mut clk_offset: Vec<(String, f64)> = Vec::new();
    // dataset
    //  per physics, per carrier signal (symbol)
    //      per vehicule (color map)
    //      bool: loss of lock - CS emphasis
    //      x: sampling timestamp,
    //      y: observation (raw),
    let mut dataset: HashMap<String, HashMap<u8, HashMap<Sv, Vec<(bool, String, f64)>>>> =
        HashMap::new();
    for ((epoch, _flag), (clock_offset, vehicules)) in record {
        if let Some(value) = clock_offset {
            clk_offset.push((epoch.to_string(), *value));
        }

        for (sv, observations) in vehicules {
            for (observation, data) in observations {
                let p_code = &observation[0..1];
                let c_code = &observation[1..2]; // carrier code
                let c_code = u8::from_str_radix(c_code, 10).expect("failed to parse carrier code");

                let physics = code2physics!(p_code);
                let y = data.obs;
                let cycle_slip = match data.lli {
                    Some(lli) => lli.intersects(LliFlags::LOCK_LOSS),
                    _ => false,
                };

                if let Some(data) = dataset.get_mut(&physics) {
                    if let Some(data) = data.get_mut(&c_code) {
                        if let Some(data) = data.get_mut(&sv) {
                            data.push((cycle_slip, epoch.to_string(), y));
                        } else {
                            data.insert(*sv, vec![(cycle_slip, epoch.to_string(), y)]);
                        }
                    } else {
                        let mut map: HashMap<Sv, Vec<(bool, String, f64)>> = HashMap::new();
                        map.insert(*sv, vec![(cycle_slip, epoch.to_string(), y)]);
                        data.insert(c_code, map);
                    }
                } else {
                    let mut map: HashMap<Sv, Vec<(bool, String, f64)>> = HashMap::new();
                    map.insert(*sv, vec![(cycle_slip, epoch.to_string(), y)]);
                    let mut mmap: HashMap<u8, HashMap<Sv, Vec<(bool, String, f64)>>> =
                        HashMap::new();
                    mmap.insert(c_code, map);
                    dataset.insert(physics.to_string(), mmap);
                }
            }
        }
    }

    if clk_offset.len() > 0 {
        ctx.add_cartesian2d_plot("Receiver Clock Offset", "Clock Offset [s]");
        let data_x: Vec<String> = clk_offset.iter().map(|(k, _)| k.clone()).collect();
        let data_y: Vec<f64> = clk_offset.iter().map(|(_, v)| *v).collect();
        let trace = Scatter::new(data_x, data_y)
            .mode(Mode::LinesMarkers)
            .marker(Marker::new().symbol(MarkerSymbol::TriangleUp))
            .name("Clk Offset");
        ctx.add_trace(trace);
    }
    /*
     * 1 plot per physics
     */
    for (physics, carriers) in dataset {
        let y_label = match physics.as_str() {
            "Phase" => "Carrier cycles",
            "Doppler" => "Doppler Shifts",
            "Signal Strength" => "Power [dB]",
            "Pseudo Range" => "Pseudo Range",
            _ => unreachable!(),
        };
        ctx.add_cartesian2d_plot(&format!("{} Observations", physics), y_label);
        // one symbol per carrier
        let markers = generate_markers(carriers.len());
        for (index, (carrier, vehicules)) in carriers.iter().enumerate() {
            for (sv, data) in vehicules {
                let data_x: Vec<String> = data.iter().map(|(cs, e, _y)| e.clone()).collect();
                let data_y: Vec<f64> = data.iter().map(|(cs, _e, y)| *y).collect();
                let trace = Scatter::new(data_x, data_y)
                    .mode(Mode::Markers)
                    .marker(Marker::new().symbol(markers[index].clone()))
                    .visible({
                        if index < 1 {
                            Visible::True
                        } else {
                            Visible::LegendOnly
                        }
                    })
                    .name(&format!("{}(L{})", sv, carrier));
                ctx.add_trace(trace);
            }
        }
    }
}
