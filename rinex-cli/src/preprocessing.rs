use log::error;
use std::str::FromStr;

use crate::Cli;
use rinex::prelude::RnxContext;
use rinex::preprocessing::*;

pub fn preprocess(ctx: &mut RnxContext, cli: &Cli) {
    // GNSS filters
    let mut gnss_filters = Vec::<&str>::new();

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

    for filter in gnss_filters {
        let filter = Filter::from_str(filter).unwrap(); // cannot fail
        if let Some(inner) = ctx.observation_mut() {
            inner.filter_mut(filter.clone());
        }
        if let Some(inner) = ctx.brdc_navigation_mut() {
            inner.filter_mut(filter.clone());
        }
        if let Some(inner) = ctx.clock_mut() {
            inner.filter_mut(filter.clone());
        }
        if let Some(_inner) = ctx.sp3_mut() {
            //TODO
        }
    }

    for filt_str in cli.preprocessing() {
        /*
         * Apply all preprocessing filters
         */
        if let Ok(filter) = Filter::from_str(filt_str) {
            if let Some(ref mut inner) = ctx.observation_mut() {
                inner.filter_mut(filter.clone());
            }
            if let Some(ref mut inner) = ctx.brdc_navigation_mut() {
                inner.filter_mut(filter.clone());
            }
            if let Some(ref mut inner) = ctx.meteo_mut() {
                inner.filter_mut(filter.clone());
            }
            if let Some(ref mut inner) = ctx.clock_mut() {
                inner.filter_mut(filter.clone());
            }
            trace!("applied filter \"{}\"", filt_str);
        } else {
            error!("invalid filter description \"{}\"", filt_str);
        }
    }
}
