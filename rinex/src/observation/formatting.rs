//! OBS RINEX formatting
use crate::{
    epoch::format as epoch_format, observation::Record, prelude::{SV, ClockObservation, Header, ObsKey, RinexType}, FormattingError
};

use std::io::{Write, BufWriter};

#[cfg(feature = "log")]
use log::error;

use itertools::Itertools;

/// Formats one epoch according to standard definitions.
/// ## Inputs
/// - major: RINEX revision major
/// - key: [ObsKey]
/// - clock: possible [ClockObservation]
/// - signals: [SignalObservation]s
/// ## Returns
/// - formatter string according to standard specifications
pub fn format<W: Write>(w: &mut BufWriter<W>, record: &Record, header: &Header) -> Result<(), FormattingError> {
    if header.version.major < 3 {
        format_v2(w, header, record)
    } else {
        format_v3(w, header, record)
    }
}

fn format_epoch_v3<W: Write>(
    w: &mut BufWriter<W>,
    k: &ObsKey,
    sv_list: &[SV],
    clock: Option<ClockObservation>,
) -> Result<(), FormattingError> {
    
    let numsat = sv_list.len();

    if let Some(clock) = clock {
        write!(
            w, 
            "> {}  {} {:2}",
            epoch_format(k.epoch, RinexType::ObservationData, 3),
            k.flag,
            numsat,
        )?;
    } else {
        write!(
            w,
            "> {}  {} {:2}",
            epoch_format(k.epoch, RinexType::ObservationData, 3),
            k.flag,
            numsat,
        )?;
    }

    Ok(())
}

fn format_v3<W: Write>(
    w: &mut BufWriter<W>,
    header: &Header,
    record: &Record,
) -> Result<(), FormattingError> {

    const NUM_SV_PER_LINE: usize = 12;
    const OBSERVATIONS_PER_LINE: usize = 5;
    const END_OF_LINE_PADDING: &str = "\n                                ";

    let observables = &header
        .obs
        .as_ref()
        .ok_or(FormattingError::UndefinedObservables)?
        .codes;

    for (k, v) in record.iter() {

        // retrieve unique sv list
        let sv_list = v.signals
            .iter()
            .map(|k| k.sv)
            .unique()
            .sorted()
            .collect::<Vec<_>>();

        // encode new epoch
        format_epoch_v3(w, k,  &sv_list, v.clock)?;

        // by sorted SV     
        for sv in sv_list.iter() {

            // following header definition
            let observables = observables.get(&sv.constellation)
                .ok_or(FormattingError::MissingObservableDefinition)?;
            
            for observable in observables.iter() {
                if let Some(observation) = v.signals
                    .iter()
                    .filter(|sig| sig.observable == *observable)
                    .reduce(|k, _| k)
                {
                    
                } else {
                    // Blanking
                }
            }
        }
    }
    Ok(())
}

fn format_epoch_v2<W: Write>(
    w: &mut BufWriter<W>,
    k: &ObsKey,
    sv_list: &[SV],
    clock: Option<ClockObservation>,
) -> Result<(), FormattingError> {
    
    let numsat = sv_list.len();

    if let Some(clock) = clock {
        write!(
            w,
            " {}  {} {:2}",
            epoch_format(k.epoch, RinexType::ObservationData, 2),
            k.flag,
            numsat,
        )?;
    } else {
        write!(
            w,
            " {}  {} {:2}",
            epoch_format(k.epoch, RinexType::ObservationData, 2),
            k.flag,
            numsat,
        )?;
    }

    Ok(())
}

fn format_v2<W: Write>(
    w: &mut BufWriter<W>,
    header: &Header,
    record: &Record,
) -> Result<(), FormattingError> {

    const NUM_SV_PER_LINE: usize = 12;
    const OBSERVATIONS_PER_LINE: usize = 5;
    const END_OF_LINE_PADDING: &str = "\n                                ";

    let observables = &header
        .obs
        .as_ref()
        .ok_or(FormattingError::UndefinedObservables)?
        .codes;

    for (k, v) in record.iter() {

        // retrieve unique sv list
        let sv_list = v.signals
            .iter()
            .map(|k| k.sv)
            .unique()
            .sorted()
            .collect::<Vec<_>>();

        // encode new epoch
        format_epoch_v2(w, k,  &sv_list, v.clock)?;

        // by sorted SV     
        for sv in sv_list.iter() {

            // following header definition
            let observables = observables.get(&sv.constellation)
                .ok_or(FormattingError::MissingObservableDefinition)?;
            
            for observable in observables.iter() {
                if let Some(observation) = v.signals
                    .iter()
                    .filter(|sig| sig.observable == *observable)
                    .reduce(|k, _| k)
                {
                    
                } else {
                    // Blanking
                }
            }
        }
    }
    Ok(())
}
