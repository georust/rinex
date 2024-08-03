use log::error;
use std::str::FromStr;

use crate::Cli;
use rinex_qc::prelude::{Filter, QcContext, Repair, RepairTrait};

pub fn preprocess(ctx: &mut QcContext, cli: &Cli) {
    // GNSS filters
    let mut gnss_filters = Vec::<&str>::new();
    /*
     * Special teqc like filters
     * Design one filter per specs
     */
    if cli.gps_filter() {
        gnss_filters.push("!=GPS");
        info!("GPS filtered out");
    }
    if cli.glo_filter() {
        gnss_filters.push("!=GLO");
        info!("Glonass filtered out");
    }
    if cli.gal_filter() {
        gnss_filters.push("!=Gal");
        info!("Galileo filtered out");
    }
    if cli.bds_filter() {
        gnss_filters.push("!=BDS");
        info!("BeiDou filtered out");
    }
    if cli.bds_geo_filter() {
        gnss_filters.push(">C05;<C55");
        info!("BeiDou GEO filtered out");
    }
    if cli.sbas_filter() {
        gnss_filters.push("!=geo");
        info!("SBAS filtered out..");
    }
    if cli.qzss_filter() {
        gnss_filters.push("!=QZSS");
        info!("QZSS filtered out");
    }
    if cli.irnss_filter() {
        gnss_filters.push("!=INRSSN");
        info!("IRNSS/LNAV filtered out");
    }

    for filter in gnss_filters {
        let filter = Filter::from_str(filter).unwrap();
        ctx.filter_mut(&filter);
    }

    for filt_str in cli.preprocessing() {
        /*
         * Apply other -P filter specs
         */
        if let Ok(filter) = Filter::from_str(filt_str) {
            ctx.filter_mut(&filter);
            trace!("applied filter \"{}\"", filt_str);
        } else {
            error!("invalid filter description \"{}\"", filt_str);
        }
    }

    if cli.zero_repair() {
        info!("repairing zero values..");
        ctx.repair_mut(Repair::Zero);
    }
}
