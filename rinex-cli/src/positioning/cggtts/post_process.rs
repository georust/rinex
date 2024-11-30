//! CGGTTS track formation and post processing
use crate::cli::Context;
use cggtts::prelude::*;
use cggtts::Coordinates;
use clap::ArgMatches;
use std::io::Write;

/*
 * CGGTTS file generation and solutions post processing
 */
pub fn post_process(
    ctx: &Context,
    tracks: &Vec<Track>,
    matches: &ArgMatches,
) -> std::io::Result<()> {
    /*
     * CGGTTS formation and customization
     */
    let obs_data = ctx.data.observation_data().unwrap(); // infaillible at this point

    // receiver customization
    let rcvr = match &obs_data.header.rcvr {
        Some(rcvr) => Rcvr {
            manufacturer: String::from("XX"),
            model: rcvr.model.clone(),
            serial_number: rcvr.sn.clone(),
            year: 0,
            release: rcvr.firmware.clone(),
        },
        None => Rcvr::default(),
    };
    // LAB/Agency customization
    let lab = if let Some(custom) = matches.get_one::<String>("lab") {
        custom.to_string()
    } else {
        let stem = Context::context_stem(&ctx.data);
        if let Some(index) = stem.find('_') {
            stem[..index].to_string()
        } else {
            "LAB".to_string()
        }
    };

    let mut cggtts = CGGTTS::default()
        .station(&lab)
        .nb_channels(1) // TODO: improve this ?
        .receiver(rcvr.clone())
        .ims(rcvr.clone()) // TODO : improve this ?
        .apc_coordinates({
            // TODO: wrong, coordinates should be expressed in ITRF: need some conversion
            let (x, y, z) = ctx.rx_ecef.unwrap(); // infallible at this point
            Coordinates { x, y, z }
        })
        .reference_frame("WGS84") //TODO: ITRF
        .reference_time({
            if let Some(utck) = matches.get_one::<String>("utck") {
                ReferenceTime::UTCk(utck.clone())
            } else if let Some(clock) = matches.get_one::<String>("clock") {
                ReferenceTime::Custom(clock.clone())
            } else {
                ReferenceTime::Custom("Unknown".to_string())
            }
        })
        .comments(&format!(
            "rinex-cli v{} - https://georust.org",
            env!("CARGO_PKG_VERSION")
        ));

    for track in tracks {
        cggtts.tracks.push(track.clone());
    }

    let mut fd = ctx.workspace.create_file(&cggtts.filename());
    write!(fd, "{}", cggtts)?;
    Ok(())
}
