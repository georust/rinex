use std::io::{BufWriter, Write};

use crate::{
    epoch::format as format_epoch,
    meteo::Record,
    prelude::{FormattingError, Header, RinexType},
};

use itertools::Itertools;

/// Formats Meteo epoch into [BufWriter]
pub fn format<W: Write>(
    w: &mut BufWriter<W>,
    record: &Record,
    header: &Header,
) -> Result<(), FormattingError> {
    let observables = &header
        .meteo
        .as_ref()
        .ok_or(FormattingError::UndefinedObservables)?;

    for t in record.keys().map(|k| k.epoch).unique().sorted() {
        writeln!(
            w,
            " {}",
            format_epoch(t, RinexType::MeteoData, header.version.major)
        )?;
        //// follow header definitions
        //for observable in observables.codes.iter() {
        //    if let Some(observation) = v.iter().filter(|obs| obs.observable == observable).reduce(|k, _| k) {
        //        write!(w, "{:14.13} ", observation.value)?;
        //    } else {
        //        write!(w, "           ")?;
        //    }
        //}
    }

    Ok(())
}
