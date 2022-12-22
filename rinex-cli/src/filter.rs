/*use crate::{Cli, Context};
use log::{error, trace};
use rinex::{observation::*, prelude::*};

fn args_to_lli_mask(args: &str) -> Option<LliFlags> {
    if let Ok(u) = u8::from_str_radix(args.trim(), 10) {
        LliFlags::from_bits(u)
    } else {
        None
    }
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
*/
