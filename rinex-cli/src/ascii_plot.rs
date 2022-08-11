use rinex::Rinex;
use rinex::sv::Sv;
use rinex::epoch::Epoch;
use rinex::channel::Channel;
use rinex::constellation::Constellation;
use rinex::observation::record::LliFlags;

use std::collections::BTreeMap;

pub const DEFAULT_X_WIDTH :u32 = 72;

fn copy (x_width: u32, s: &str) -> String {
    let mut result= String::new();
    for _ in 0..x_width {
        result.push_str(s);
    }
    result
}

fn x_axis (x_width: u32) -> String {
    copy(x_width, "-")
}

fn blank (x_width: u32) -> String {
    copy(x_width, " ")
}

pub fn ascii_plot (x_width: u32, obs_rinex: &Rinex, nav_rinex: Option<Rinex>) -> String {
    if !obs_rinex.is_observation_rinex() {
        return String::new();
    }

    let mut result = String::new();
    let mut elev_angles : BTreeMap<Epoch, BTreeMap<Sv, f64>> = BTreeMap::new();
    if let Some(rnx) = nav_rinex {
        elev_angles = rnx.elevation_angles();
    }
    
    // epochs list
    let epochs = obs_rinex.epochs();
    let time_span = epochs[epochs.len()-1].date - epochs[0].date;
    
    // list vehicules, on an epoch basis
    let mut vehicules = obs_rinex.space_vehicules();
    vehicules.sort();

    // extract record
    let record = obs_rinex.record
        .as_obs()
        .unwrap();
    let px_secs = time_span.num_seconds() as u32 / x_width; // nb of secs per px
    let granularity_dt = chrono::Duration::from_std(std::time::Duration::from_secs(px_secs.into())).unwrap();
    
    // print Sv Header / upper axis
    result.push_str(" SV+");
    result.push_str(&x_axis(x_width));
    result.push_str("\n");

    for sv in vehicules.iter() {
        // start new line for this svnn 
        result.push_str(&format!("{}{:>2}|", sv.constellation.to_1_letter_code(), sv.prn));
        let mut prev_epoch = epochs[0];
        let mut loss_of_lock = false;
        let mut anti_spoofing = false;
        let mut code_is_l1 = false;
        let mut code_is_l2 = false;
        let mut clock_slip = false;
        //let no_obs = true;
        //let mut above_elev = true;
        let mut has_data = false;
        let mut data_missing = false;
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
        // "o" phase and/or code data for SV is L1, C/A, L2, P2 & A/S is on; if qc full, satellite was above elevation mask 
        // "y" phase and/or code data for SV is L1, P1, L2, P2 & A/S is on; if qc full, satellite was above elevation mask 
        // "L" Loss of Lock indicator was set by receiver for L1 and/or L2; detection is turned off with -lli option 
        //  _ satellite between horizon and elevation mask with no data collected by receiver; indicator is turned off with -hor option 
        // (blank) qc lite: no satellite tracked
        // --> browse epochs
        for (epoch, (_, vehicules)) in record.iter() {
            if let Some(map) = vehicules.get(sv) {
                has_data |= true;
                // if we do have ephemeris attached
                // use them to determine elevation angle
                for (e, vvehicules) in elev_angles.iter() {
                    if *e == *epoch {
                        for (vvehicule, _angle) in vvehicules.iter() {
                            if *vvehicule == *sv {
                                //above_elev = *angle > 30.0 ; // TODO
                            }
                        }
                    }
                }
                for (obscode, data) in map.iter() {
                    if let Ok(channel) = Channel::from_observable(Constellation::GPS, obscode) {
                        match channel {
                            Channel::L1 => {
                                code_is_l1 |= true;        
                            },
                            Channel::L2 => {
                                code_is_l2 |= true;
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
            } else {
                data_missing |= true;
            }
            
            if epoch.date - prev_epoch.date >= granularity_dt {
                // end of time granularity
                // make decision of which marker to use
                let mut marker = " ";
                if !has_data {
                    // period completely empty
                    marker = " "
                } else {
                    if data_missing {
                        // period was partial
                        marker = "^"

                    } else {
                        // complete period
                        if loss_of_lock { // loss of lock prevails
                            marker = "L";
                        } else {
                            if clock_slip { // clock slip prevails
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
                }
                result.push_str(marker);
                prev_epoch = epoch.clone();
            } // marker granularity
        } // OBS record browsing
        // conclude svnn line
        result.push_str(&format!("|{}{:>2}\n", sv.constellation.to_1_letter_code(), sv.prn));
    }// all vehicules in record

    // final, "Clk" line
    result.push_str("Clk|");
    result.push_str("|Clk\n");
    
    // bottom axis
    result.push_str("   +");
    result.push_str(&x_axis(x_width));
    result.push_str("\n");

    // time span reminder
    result.push_str(&epochs[0].date.format("    %H:%M:%S").to_string());
    result.push_str(&blank(x_width - 16));
    result.push_str(&epochs[epochs.len()-1].date.format("%H:%M:%S").to_string());
    result.push_str("\n");
    result.push_str(&epochs[0].date.format("    %Y %b %d").to_string());
    result.push_str(&blank(x_width - 22));
    result.push_str(&epochs[epochs.len()-1].date.format("%Y %b %d").to_string());
    result.push_str("\n");

    result 
}
