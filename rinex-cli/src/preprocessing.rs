use log::error;
use std::str::FromStr;

use crate::Cli;
use rinex::preprocessing::*;
use rinex::quality::QcContext;

pub fn preprocess(ctx: &mut QcContext, cli: &Cli) {
    // GNSS filters
    let mut gnss_filters: Vec<&str> = Vec::new();

    if cli.gps_filter() {
        gnss_filters.push("!=gps");
        trace!("applying -G filter..");
    }
    if cli.glo_filter() {
        gnss_filters.push("!=glo");
        trace!("applying -R filter..");
    }
    if cli.gal_filter() {
        gnss_filters.push("!=gal");
        trace!("applying -E filter..");
    }
    if cli.bds_filter() {
        gnss_filters.push("!=bds");
        trace!("applying -C filter..");
    }
    if cli.sbas_filter() {
        gnss_filters.push("!=geo");
        trace!("applying -S filter..");
    }
    if cli.qzss_filter() {
        gnss_filters.push("!=qzss");
        trace!("applying -J filter..");
    }

    for filt in gnss_filters {
        let filt = Filter::from_str(filt).unwrap(); // cannot fail
        ctx.primary_data_mut().filter_mut(filt.clone());
        if let Some(ref mut nav) = ctx.navigation_data_mut() {
            nav.filter_mut(filt.clone());
        }
    }

    for filt_str in cli.preprocessing() {
        if let Ok(filt) = Filter::from_str(filt_str) {
            ctx.primary_data_mut().filter_mut(filt.clone());
            if let Some(ref mut nav) = ctx.navigation_data_mut() {
                nav.filter_mut(filt.clone());
            }
            trace!("applied filter \"{}\"", filt_str);
        } else {
            error!("invalid filter description \"{}\"", filt_str);
        }
    }
}
