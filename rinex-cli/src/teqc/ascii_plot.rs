use rinex::*;
use rinex::sv::Sv;
use rinex::epoch::Epoch;
use rinex::channel::Channel;
use rinex::constellation::Constellation;
use rinex::observation::record::LliFlags;

use itertools::Itertools;
use std::convert::TryInto;
use std::collections::BTreeMap;

pub const DEFAULT_X_WIDTH :u32 = 72;

fn copy (x_width: u32, s: &str) -> String {
    let mut result= String::new();
    for _ in 0..x_width {
        result.push_str(s);
    }
    result
}

fn blank (x_width: u32) -> String {
    copy(x_width, " ")
}

fn x_axis (x_width: u32) -> String {
    let mut string = copy(x_width, "-");
    let nb_spacing = 8;
    let spacing_width = x_width / nb_spacing;
    for i in 1..nb_spacing {
        let min : usize = (i*spacing_width).try_into().unwrap();
        let max : usize = (i*spacing_width +1).try_into().unwrap();
        string.replace_range(min..max, "|");
    }
    string.clone()
}

pub fn ascii_plot (x_width: u32, obs_rinex: &Rinex, nav_rinex: Option<Rinex>) -> String {
    if !obs_rinex.is_observation_rinex() {
        return String::new();
    }

    let mut result = String::new();
    let mut elev_angles : BTreeMap<Epoch, BTreeMap<Sv, f64>> = BTreeMap::new();
    if let Some(rnx) = nav_rinex {
        elev_angles = rnx
            .orbits_elevation_angles();
    }
    
    // epochs list
    let mut epochs = obs_rinex.epochs();
    epochs.sort();
    let time_span = epochs[epochs.len()-1].date - epochs[0].date;
    let px_secs = time_span.num_seconds() as u32 / x_width; // nb of secs per px
    let dt_granularity = chrono::TimeDelta::from_std(std::time::Duration::from_secs(px_secs.into())).unwrap();
    
    // list vehicles, on an epoch basis
    let mut vehicles = obs_rinex.space_vehicles();
    vehicles.sort();

    // extract record
    let record = obs_rinex.record
        .as_obs()
        .unwrap();
    
    // print Sv Header / upper axis
    result.push_str(" Sv+");
    result.push_str(&x_axis(x_width));
    result.push_str("+Sv");
    result.push_str("\n");

    for sv in vehicles.iter() {
        // start new line for this svnn 
        result.push_str(&format!("{}{:>2}|", sv.constellation.to_1_letter_code(), sv.prn));
        let mut prev_epoch = epochs[0];
        let mut loss_of_lock = false;
        let mut anti_spoofing = false;
        let mut _code_is_l1 = false;
        let mut _code_is_l2 = false;
        let mut clock_slip = false;
        //let no_obs = true;
        //let mut above_elev = true;
        let mut _epoch_empty = false;
        // "C" : clock slip
        // "I" : iono delay phase slip
        // "-" satellite was above elevation mask, but no data was apparently recorded by the receiver
        // "+" satellite was below elevation mask and a complete set of phase and code data was collected 
        // "^" satellite was below elevation mask and a partial set of phase and code data was collected 
        // "." phase and/or code data for SV is L1 and C/A only & A/S is off; if qc full, satellite was above elevation mask 
        // ":" phase and/or code data for SV is L1 and P1 only & A/S is off; if qc full, satellite was above elevation mask 
        // "~" phase and/or code data for SV is L1, C/A, L2, P2 & A/S is off; if qc full, satellite was above elevation mask 
        // "*" phase and/or code data for SV is L1, P1, L2, P2 & A/S is off; if qc full, satellite was above elevation mask 
        // "," phase and/or code data for SV is L1 and C/A only & A/S is on; if qc full, satellite was above elevation mask 
        // ";" phase and/or code data for SV is L1 and P1 only & A/S is on; if qc full, satellite was above elevation mask 

        // "L" Loss of Lock indicator was set by receiver for L1 and/or L2; detection is turned off with -lli option 
        //  _ satellite between horizon and elevation mask with no data collected by receiver; indicator is turned off with -hor option 
        // (blank) qc lite: no satellite tracked
        // --> browse epochs
        for (epoch, (_, vehicles)) in record.iter() {
            _epoch_empty = true;
            if let Some(map) = vehicles.get(sv) {
                _epoch_empty = false;
                // if we do have ephemeris attached
                // use them to determine elevation angle
                for (e, vvehicles) in elev_angles.iter() {
                    if *e == *epoch {
                        for (vvehicle, _angle) in vvehicles.iter() {
                            if *vvehicle == *sv {
                                //above_elev = *angle > 30.0 ; // TODO
                            }
                        }
                    }
                }
                // determine observable related info:
                // L1/L2 code? LLI flag..
                for (obscode, data) in map.iter() {
                    if let Ok(channel) = Channel::from_observable(Constellation::GPS, obscode) {
                        match channel {
                            Channel::L1 => {
                                _code_is_l1 |= true;        
                            },
                            Channel::L2 => {
                                _code_is_l2 |= true;
                            },
                            _ => {},
                        }
                    }
                    if let Some(lli) = data.lli {
                        loss_of_lock |= lli.intersects(LliFlags::LOCK_LOSS);
                        clock_slip |= lli.intersects(LliFlags::HALF_CYCLE_SLIP);
                        anti_spoofing |= lli.intersects(LliFlags::UNDER_ANTI_SPOOFING);
                    }
                }
            }
            
            let dt = epoch.date - prev_epoch.date;
            if dt >= dt_granularity { // time to place new marker(s)
                let n_markers = dt.num_seconds() / dt_granularity.num_seconds(); // nb of markers to place
                                                // this covers smaller files, with few epochs
                for _ in 0..n_markers {
                    let mut marker = " "; // epoch assumed empty
                    if !_epoch_empty {
                        if loss_of_lock { // loss of lock prevails
                            marker = "L";
                        } else {
                            if clock_slip { // possible clock slip prevails
                                marker = "C";
                            } else { // OBS
                               if anti_spoofing { // A/S ?
                                    marker = "o";
                               } else {
                                    marker = "~"; // normal conditions
                               }
                            }
                        }
                    }
                    result.push_str(marker);
                } // markers
            } // epoch granularity
            prev_epoch = epoch.clone();
        } // epochs browsing
        // --> conclude svnn line
        result.push_str(&format!("|{}{:>2}\n", sv.constellation.to_1_letter_code(), sv.prn));
    }// all vehicles in record

    ///////////////////////////////////////////////////
    // "-dn" line
    // emphasizes missing sv per epoch
    result.push_str("-dn|");
    let mut prev_epoch = epochs[0];
    for (e, (_, vvehicles)) in record.iter() {
        let mut n_missing = 0;
        for sv in vehicles.iter() {
            if !vvehicles.keys().contains(&sv) {
                n_missing += 1;
            }
        }
        let dt = e.date - prev_epoch.date;
        if dt >= dt_granularity { // time to place new marker(s)
            let n_markers = dt.num_seconds() / dt_granularity.num_seconds();
            for _ in 0..n_markers {
                result.push_str(&n_missing.to_string());
            } // markers
        } // epoch granularity
        prev_epoch = e.clone();
    } // epochs browsing
    result.push_str("|-dn\n");
    ///////////////////////////////////////////////////
    
    ///////////////////////////////////////////////////
    // +dn line: ??
    ///////////////////////////////////////////////////
    result.push_str("+dn|");
    let mut prev_epoch = epochs[0];
    for (e, (_, _)) in record.iter() {
        let dt = e.date - prev_epoch.date;
        if dt >= dt_granularity { // time to place new marker(s)
            let n_markers = dt.num_seconds() / dt_granularity.num_seconds();
            for _ in 0..n_markers {
                result.push_str(" ");
            } // markers
        } // epoch granularity
        prev_epoch = e.clone();
    } // epochs browsing
    result.push_str("|+dn\n");

    ///////////////////////////////////////////////////
    // +DD line: emphasize receiver capacity
    //     versus vehicles currently seen
    //      Currently we assume a +12 receiver capacity
    //      which is the default teqc value,
    //      but we should have some interface for the user
    //      to set this value
    ///////////////////////////////////////////////////
    result.push_str("+12|");
    let mut prev_epoch = epochs[0];
    for (e, (_, vehicles)) in record.iter() {
        let n_seen = vehicles.len();
        let dt = e.date - prev_epoch.date;
        if dt >= dt_granularity { // time to place new marker(s)
            let n_markers = dt.num_seconds() / dt_granularity.num_seconds();
            for _ in 0..n_markers {
                result.push_str(&format!("{:x}", std::cmp::min(12, n_seen)));
            } // markers
        } // epoch granularity
        prev_epoch = e.clone();
    } // epochs browsing
    result.push_str("|+12\n");

    ///////////////////////////////////////////////////
    // The "Pos" line records the success or non-success 
    // of calculated code positions for the antenna at the different epochs. 
    // Generally, you should see an o recorded for each bin in which a position calculation was successful.
    ///////////////////////////////////////////////////
    result.push_str("Pos|");
    for (e, (_, _)) in record.iter() {
        let dt = e.date - prev_epoch.date;
        if dt >= dt_granularity { // time to place new marker(s)
            let n_markers = dt.num_seconds() / dt_granularity.num_seconds();
            for _ in 0..n_markers {
                result.push_str("o");
            } // markers
        } // epoch granularity
        prev_epoch = e.clone();
    } // epochs browsing
    result.push_str("|Pos\n");

    ///////////////////////////////////////////////////
    // "Clk" line
    result.push_str("Clk|");
    for (e, (_, _)) in record.iter() {
        let dt = e.date - prev_epoch.date;
        if dt >= dt_granularity { // time to place new marker(s)
            let n_markers = dt.num_seconds() / dt_granularity.num_seconds();
            for _ in 0..n_markers {
                result.push_str(" ");
            } // markers
        } // epoch granularity
        prev_epoch = e.clone();
    } // epochs browsing
    result.push_str("|Clk\n");
    ///////////////////////////////////////////////////

    ///////////////////////////////////////////////////
    // bottom axis
    result.push_str("   +");
    result.push_str(&x_axis(x_width));
    result.push_str("+\n");
    ///////////////////////////////////////////////////

    ///////////////////////////////////////////////////
    // time span reminder
    result.push_str(&epochs[0].date.format("    %H:%M:%S").to_string());
    result.push_str(&blank(x_width - 16));
    result.push_str(&epochs[epochs.len()-1].date.format("%H:%M:%S").to_string());
    result.push_str("\n");
    result.push_str(&epochs[0].date.format("    %Y %b %d").to_string());
    result.push_str(&blank(x_width - 22));
    result.push_str(&epochs[epochs.len()-1].date.format("%Y %b %d").to_string());
    result.push_str("\n");
    ///////////////////////////////////////////////////
    result 
}
