use crate::{
    //parser::{parse_duration, parse_epoch},
    Cli,
    Context,
};
use log::error;
use rinex::filter;
use rinex::preprocessing::*;
use std::str::FromStr;

pub fn preprocess(ctx: &mut Context, cli: &Cli) {
    // quick GNSS filter
    if cli.gps_filter() {
        let gnss_filter = filter!("!=gps");
        ctx.primary_rinex.filter_mut(gnss_filter.clone());
        if let Some(ref mut nav) = ctx.nav_rinex {
            nav.filter_mut(gnss_filter.clone());
        }
        trace!("applied -G filter");
    }
    if cli.glo_filter() {
        let gnss_filter = filter!("!=glo");
        ctx.primary_rinex.filter_mut(gnss_filter.clone());
        if let Some(ref mut nav) = ctx.nav_rinex {
            nav.filter_mut(gnss_filter.clone());
        }
        trace!("applied -R filter");
    }
    if cli.gal_filter() {
        let gnss_filter = filter!("!=gal");
        ctx.primary_rinex.filter_mut(gnss_filter.clone());
        if let Some(ref mut nav) = ctx.nav_rinex {
            nav.filter_mut(gnss_filter.clone());
        }
        trace!("applied -E filter");
    }
    if cli.bds_filter() {
        let gnss_filter = filter!("!=bds");
        ctx.primary_rinex.filter_mut(gnss_filter.clone());
        if let Some(ref mut nav) = ctx.nav_rinex {
            nav.filter_mut(gnss_filter.clone());
        }
        trace!("applied -C filter");
    }
    if cli.sbas_filter() {
        let gnss_filter = filter!("!=geo");
        ctx.primary_rinex.filter_mut(gnss_filter.clone());
        if let Some(ref mut nav) = ctx.nav_rinex {
            nav.filter_mut(gnss_filter.clone());
        }
        trace!("applied -S filter");
    }
    if cli.qzss_filter() {
        let gnss_filter = filter!("!=qzss");
        ctx.primary_rinex.filter_mut(gnss_filter.clone());
        if let Some(ref mut nav) = ctx.nav_rinex {
            nav.filter_mut(gnss_filter.clone());
        }
        trace!("applied -J filter");
    }
    for filt_str in cli.preprocessing() {
        if let Ok(filt) = Filter::from_str(filt_str) {
            ctx.primary_rinex.filter_mut(filt.clone());
            if let Some(ref mut nav) = ctx.nav_rinex {
                nav.filter_mut(filt.clone());
            }
            trace!("applied filter \"{}\"", filt_str);
        } else {
            error!("invalid filter description \"{}\"", filt_str);
        }
    }
}
