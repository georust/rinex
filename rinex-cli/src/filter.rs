use crate::{Cli, Context};
use log::{error, trace};
use rinex::{observation::*, prelude::*};

fn args_to_lli_mask(args: &str) -> Option<LliFlags> {
    if let Ok(u) = u8::from_str_radix(args.trim(), 10) {
        LliFlags::from_bits(u)
    } else {
        None
    }
}

pub fn apply_gnss_filters(ctx: &mut Context, cli: &Cli) {
/*
    if cli.gps_filter() {
        ctx.primary_rinex.retain_constellation_mut(vec![
            Constellation::Glonass,
            Constellation::BeiDou,
            Constellation::Galileo,
            Constellation::QZSS,
            Constellation::Geo,
            Constellation::SBAS(Augmentation::Unknown),
        ]);
        if let Some(ref mut nav) = ctx.nav_rinex {
            nav.retain_constellation_mut(vec![
                Constellation::Glonass,
                Constellation::BeiDou,
                Constellation::Galileo,
                Constellation::QZSS,
                Constellation::Geo,
                Constellation::SBAS(Augmentation::Unknown),
            ]);
        }
        trace!("-G filter");
    }
    if cli.glo_filter() {
        ctx.primary_rinex.retain_constellation_mut(vec![
            Constellation::GPS,
            Constellation::BeiDou,
            Constellation::Galileo,
            Constellation::QZSS,
            Constellation::Geo,
            Constellation::SBAS(Augmentation::Unknown),
        ]);
        if let Some(ref mut nav) = ctx.nav_rinex {
            nav.retain_constellation_mut(vec![
                Constellation::GPS,
                Constellation::BeiDou,
                Constellation::Galileo,
                Constellation::QZSS,
                Constellation::Geo,
                Constellation::SBAS(Augmentation::Unknown),
            ]);
        }
        trace!("-R filter");
    }
    if cli.bds_filter() {
        ctx.primary_rinex.retain_constellation_mut(vec![
            Constellation::GPS,
            Constellation::Glonass,
            Constellation::Galileo,
            Constellation::QZSS,
            Constellation::Geo,
            Constellation::SBAS(Augmentation::Unknown),
        ]);
        if let Some(ref mut nav) = ctx.nav_rinex {
            nav.retain_constellation_mut(vec![
                Constellation::GPS,
                Constellation::Glonass,
                Constellation::Galileo,
                Constellation::QZSS,
                Constellation::Geo,
                Constellation::SBAS(Augmentation::Unknown),
            ]);
        }
        trace!("-C filter");
    }
    if cli.sbas_filter() {
        ctx.primary_rinex.retain_constellation_mut(vec![
            Constellation::GPS,
            Constellation::Glonass,
            Constellation::BeiDou,
            Constellation::Galileo,
            Constellation::QZSS,
        ]);
        if let Some(ref mut nav) = ctx.nav_rinex {
            nav.retain_constellation_mut(vec![
                Constellation::GPS,
                Constellation::Glonass,
                Constellation::BeiDou,
                Constellation::Galileo,
                Constellation::QZSS,
            ]);
        }
        trace!("-S filter");
    }
    if cli.gal_filter() {
        ctx.primary_rinex.retain_constellation_mut(vec![
            Constellation::GPS,
            Constellation::Glonass,
            Constellation::BeiDou,
            Constellation::QZSS,
            Constellation::Geo,
            Constellation::SBAS(Augmentation::Unknown),
        ]);
        if let Some(ref mut nav) = ctx.nav_rinex {
            nav.retain_constellation_mut(vec![
                Constellation::GPS,
                Constellation::Glonass,
                Constellation::BeiDou,
                Constellation::QZSS,
                Constellation::Geo,
                Constellation::SBAS(Augmentation::Unknown),
            ]);
        }
        trace!("-E filter");
    }
    if cli.qzss_filter() {
        ctx.primary_rinex.retain_constellation_mut(vec![
            Constellation::GPS,
            Constellation::Glonass,
            Constellation::BeiDou,
            Constellation::Galileo,
            Constellation::Geo,
            Constellation::SBAS(Augmentation::Unknown),
        ]);
        if let Some(ref mut nav) = ctx.nav_rinex {
            nav.retain_constellation_mut(vec![
                Constellation::GPS,
                Constellation::Glonass,
                Constellation::BeiDou,
                Constellation::Galileo,
                Constellation::Geo,
                Constellation::SBAS(Augmentation::Unknown),
            ]);
        }
        trace!("-J filter");
    }
    */
}

pub fn apply_filters(ctx: &mut Context, cli: &Cli) {
/*
    let ops = cli.filter_ops();
    for (op, args) in ops.iter() {
        if op.eq(&"lli-mask") {
            if let Some(mask) = args_to_lli_mask(args) {
                ctx.primary_rinex.observation_lli_and_mask_mut(mask);
                trace!("lli mask applied");
            } else {
                error!("invalid lli mask \"{}\"", args);
            }
        }
    }
*/
}

//TODO
pub fn elevation_mask_filter(ctx: &mut Context, cli: &Cli) {
/*    if let Some(mask) = cli.elevation_mask() {
        if let Some(ref mut nav) = ctx.nav_rinex {
            nav.elevation_mask_mut(mask, ctx.ground_position)
        }
    }*/
}
