//! Observation RINEX decimation
use crate::{observation::Record, prelude::Epoch};
use qc_traits::{DecimationFilter, DecimationFilterType};

pub(crate) fn decim_mut(rec: &mut Record, decim: &DecimationFilter) {
    if decim.item.is_some() {
        todo!("targetted decimation not supported yet");
    }
    match decim.filter {
        DecimationFilterType::Modulo(r) => {
            let mut i = 0;
            rec.retain(|_, _| {
                let retained = (i % r) == 0;
                i += 1;
                retained
            });
        },
        DecimationFilterType::Duration(interval) => {
            let mut last_retained = Option::<Epoch>::None;
            rec.retain(|k, _| {
                if let Some(last) = last_retained {
                    let dt = k.epoch - last;
                    if dt >= interval {
                        last_retained = Some(k.epoch);
                        true
                    } else {
                        false
                    }
                } else {
                    last_retained = Some(k.epoch);
                    true // always retain 1st epoch
                }
            });
        },
    }
}
