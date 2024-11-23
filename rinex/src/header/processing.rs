use crate::header::Header;
use qc_traits::{FilterItem, MaskFilter, MaskOperand};

fn header_mask_eq(hd: &mut Header, _item: &FilterItem) {}

pub(crate) fn header_mask_mut(hd: &mut Header, f: &MaskFilter) {
    match f.operand {
        MaskOperand::Equals => header_mask_eq(hd, &f.item),
        MaskOperand::NotEquals => {},
        MaskOperand::GreaterThan => {},
        MaskOperand::GreaterEquals => {},
        MaskOperand::LowerThan => {},
        MaskOperand::LowerEquals => {},
    }
    if let Some(obs) = &mut hd.obs {
        obs.mask_mut(f);
    }
    if let Some(met) = &mut hd.meteo {
        met.mask_mut(f);
    }
    if let Some(ionex) = &mut hd.ionex {
        ionex.mask_mut(f);
    }
    if let Some(doris) = &mut hd.doris {
        doris.mask_mut(f);
    }
}
