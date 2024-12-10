use std::io::{BufWriter, Write};

use crate::{
    epoch::format as format_epoch,
    meteo::{MeteoKey, Record},
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

    for epoch in record.keys().map(|k| k.epoch).unique().sorted() {
        writeln!(
            w,
            " {}",
            format_epoch(epoch, RinexType::MeteoData, header.version.major)
        )?;

        // follow header definitions
        for observable in observables.codes.iter() {
            let key = MeteoKey {
                epoch,
                observable: observable.clone(),
            };

            if let Some(observation) = record.get(&key) {
                write!(w, "{:14.13} ", observation)?;
            } else {
                write!(w, "           ")?;
            }
        }
    }

    Ok(())
}
