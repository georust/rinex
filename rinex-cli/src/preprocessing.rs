use log::error;
use std::str::FromStr;

use crate::Cli;
use rinex::prelude::RnxContext;
use rinex::preprocessing::*;

pub fn preprocess(ctx: &mut RnxContext, cli: &Cli) {
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
    if cli.irnss_filter() {
        gnss_filters.push("!=irnss");
        trace!("applying -I filter..");
    }

    for filt in gnss_filters {
        let filt = Filter::from_str(filt).unwrap(); // cannot fail
        if let Some(ref mut obs) = ctx.obs_data_mut() {
            obs.filter_mut(filt.clone());
        }
        if let Some(ref mut nav) = ctx.nav_data_mut() {
            nav.filter_mut(filt.clone());
        }
    }

    for filt_str in cli.preprocessing() {
        /*
         * TODO
         * special case : apply to specific file format only
         */
        let only_obs = filt_str.starts_with("obs:");
        let only_met = filt_str.starts_with("met:");
        let only_nav = filt_str.starts_with("nav:");
        let special_prefix = only_obs || only_met || only_nav;
        let offset: usize = match special_prefix {
            true => 4, // "obs:","met:",..,"ion:"
            false => 0,
        };
        if let Ok(filt) = Filter::from_str(&filt_str[offset..]) {
            if let Some(ref mut data) = ctx.obs_data_mut() {
                data.filter_mut(filt.clone());
            }
            if let Some(ref mut data) = ctx.meteo_data_mut() {
                data.filter_mut(filt.clone());
            }
            if let Some(ref mut data) = ctx.nav_data_mut() {
                data.filter_mut(filt.clone());
            }
            if let Some(ref mut data) = ctx.ionex_data_mut() {
                data.filter_mut(filt.clone());
            }
            trace!("applied filter \"{}\"", filt_str);
        } else {
            error!("invalid filter description \"{}\"", filt_str);
        }
    }
}
