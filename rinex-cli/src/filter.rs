use rinex::{*,
    observation::LliFlags,
};

fn args_to_lli_mask (args: &str) -> Option<LliFlags> {
    if let Ok(u) = u8::from_str_radix(args.trim(), 10) {
        LliFlags::from_bits(u)
    } else {
        None
    }
}

/// Efficient RINEX content filter
pub fn apply_filters(rnx: &mut Rinex, ops: Vec<(&str, &str)>) {
    for (op, args) in ops.iter() {
        if op.eq(&"lli-mask") {
            if let Some(mask) = args_to_lli_mask(args) {
                rnx
                    .observation_lli_and_mask_mut(mask);
            } else {
                println!("invalid LLI mask value \"{}\"", args);
            }
        }
    }
}
