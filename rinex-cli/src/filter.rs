use crate::Cli;
use log::trace;
use rinex::{observation::*, prelude::*};

fn args_to_lli_mask(args: &str) -> Option<LliFlags> {
    if let Ok(u) = u8::from_str_radix(args.trim(), 10) {
        LliFlags::from_bits(u)
    } else {
        None
    }
}

pub fn apply_gnss_filters(cli: &Cli, rnx: &mut Rinex) {
    if cli.gps_filter() {
        rnx.retain_constellation_mut(vec![
            Constellation::Glonass,
            Constellation::BeiDou,
            Constellation::Galileo,
            Constellation::QZSS,
            Constellation::Geo,
            Constellation::SBAS(Augmentation::Unknown),
        ]);
    }
    if cli.glo_filter() {
        rnx.retain_constellation_mut(vec![
            Constellation::GPS,
            Constellation::BeiDou,
            Constellation::Galileo,
            Constellation::QZSS,
            Constellation::Geo,
            Constellation::SBAS(Augmentation::Unknown),
        ]);
    }
    if cli.bds_filter() {
        trace!("-C filter");
        rnx.retain_constellation_mut(vec![
            Constellation::GPS,
            Constellation::Glonass,
            Constellation::Galileo,
            Constellation::QZSS,
            Constellation::Geo,
            Constellation::SBAS(Augmentation::Unknown),
        ]);
    }
    if cli.sbas_filter() {
        trace!("-S filter");
        rnx.retain_constellation_mut(vec![
            Constellation::GPS,
            Constellation::Glonass,
            Constellation::BeiDou,
            Constellation::Galileo,
            Constellation::QZSS,
        ]);
    }
    if cli.gal_filter() {
        rnx.retain_constellation_mut(vec![
            Constellation::GPS,
            Constellation::Glonass,
            Constellation::BeiDou,
            Constellation::QZSS,
            Constellation::Geo,
            Constellation::SBAS(Augmentation::Unknown),
        ]);
    }
    if cli.qzss_filter() {
        rnx.retain_constellation_mut(vec![
            Constellation::GPS,
            Constellation::Glonass,
            Constellation::BeiDou,
            Constellation::Galileo,
            Constellation::Geo,
            Constellation::SBAS(Augmentation::Unknown),
        ]);
    }
}

/// Efficient RINEX content filter
pub fn apply_filters(rnx: &mut Rinex, ops: Vec<(&str, &str)>) {
    // Apply other filtering operations, if any
    for (op, args) in ops.iter() {
        if op.eq(&"lli-mask") {
            if let Some(mask) = args_to_lli_mask(args) {
                rnx.observation_lli_and_mask_mut(mask);
            } else {
                println!("invalid LLI mask value \"{}\"", args);
            }
        }
    }
}
