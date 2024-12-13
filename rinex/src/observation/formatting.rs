//! OBS RINEX formatting
use crate::{
    epoch::format as epoch_format,
    observation::Record,
    observation::{ClockObservation, ObsKey},
    prelude::{Header, RinexType, SV},
    FormattingError,
};

use std::io::{BufWriter, Write};

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
pub fn format<W: Write>(
    w: &mut BufWriter<W>,
    record: &Record,
    header: &Header,
) -> Result<(), FormattingError> {
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
        writeln!(
            w,
            "> {}  {} {:2}  {:13.4}",
            epoch_format(k.epoch, RinexType::ObservationData, 3),
            k.flag,
            numsat,
            clock.offset_s,
        )?;
    } else {
        writeln!(
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
    let observables = &header
        .obs
        .as_ref()
        .ok_or(FormattingError::UndefinedObservables)?
        .codes;

    for (k, v) in record.iter() {
        // retrieve unique sv list
        let sv_list = v
            .signals
            .iter()
            .map(|k| k.sv)
            .unique()
            .sorted()
            .collect::<Vec<_>>();

        // encode new epoch
        format_epoch_v3(w, k, &sv_list, v.clock)?;

        // by sorted SV
        for sv in sv_list.iter() {
            // following header definition
            let observables = observables
                .get(&sv.constellation)
                .ok_or(FormattingError::MissingObservableDefinition)?;

            for observable in observables.iter() {
                if let Some(observation) = v
                    .signals
                    .iter()
                    .filter(|sig| sig.observable == *observable)
                    .reduce(|k, _| k)
                {
                    write!(w, "{:14.13}", observation.value)?;
                } else {
                    // Blanking
                    write!(w, "             ")?;
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
    const NUM_SV_PER_LINE: usize = 12;
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

    for (nth, sv) in sv_list.iter().enumerate() {
        write!(w, "{:x}", sv)?;
        if nth % NUM_SV_PER_LINE == NUM_SV_PER_LINE - 1 {
            write!(w, "{}", '\n')?;
        }
    }

    Ok(())
}

fn format_v2<W: Write>(
    w: &mut BufWriter<W>,
    header: &Header,
    record: &Record,
) -> Result<(), FormattingError> {
    const OBSERVATIONS_PER_LINE: usize = 5;

    let observables = &header
        .obs
        .as_ref()
        .ok_or(FormattingError::UndefinedObservables)?
        .codes;

    for (k, v) in record.iter() {
        // retrieve unique sv list
        let sv_list = v
            .signals
            .iter()
            .map(|k| k.sv)
            .unique()
            .sorted()
            .collect::<Vec<_>>();

        // encode new epoch
        format_epoch_v2(w, k, &sv_list, v.clock)?;

        // by sorted SV
        for sv in sv_list.iter() {
            // following header definition
            let observables = observables
                .get(&sv.constellation)
                .ok_or(FormattingError::MissingObservableDefinition)?;

            for (nth, observable) in observables.iter().enumerate() {
                if let Some(observation) = v
                    .signals
                    .iter()
                    .filter(|sig| sig.observable == *observable)
                    .reduce(|k, _| k)
                {
                    write!(w, "{:14.13}", observation.value)?;
                } else {
                    // Blanking
                    write!(w, "{:14.13}", " ")?;
                }
                if nth % OBSERVATIONS_PER_LINE == OBSERVATIONS_PER_LINE - 1 {
                    write!(w, "{}", '\n')?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {

    use super::{format_epoch_v2, format_epoch_v3};

    use crate::prelude::{
        obs::{EpochFlag, ObsKey},
        Epoch, SV,
    };

    use std::io::BufWriter;
    use std::str::FromStr;

    use crate::tests::formatting::Utf8Buffer;

    #[test]
    fn test_fmt_epoch_v2() {
        let mut buf = BufWriter::new(Utf8Buffer::new(1024));

        let key = ObsKey {
            flag: EpochFlag::Ok,
            epoch: Epoch::from_str("2021-01-01T00:00:00 GPST").unwrap(),
        };

        let sv_list = [
            SV::from_str("G01").unwrap(),
            SV::from_str("G02").unwrap(),
            SV::from_str("G03").unwrap(),
            SV::from_str("G04").unwrap(),
            SV::from_str("G05").unwrap(),
            SV::from_str("G06").unwrap(),
            SV::from_str("G07").unwrap(),
            SV::from_str("G08").unwrap(),
            SV::from_str("G09").unwrap(),
            SV::from_str("G10").unwrap(),
        ];

        format_epoch_v2(&mut buf, &key, &sv_list, None).unwrap();

        let content = buf.into_inner().unwrap().to_ascii_utf8();

        assert_eq!(
            content,
            " 17  1  1  0  0  0.0000000  0 10G03G08G14G16G22G23G26G27G31G32\n",
        );
    }

    #[test]
    fn test_fmt_epoch_v3() {
        let mut buf = BufWriter::new(Utf8Buffer::new(1024));

        let key = ObsKey {
            flag: EpochFlag::Ok,
            epoch: Epoch::from_str("2021-01-01T00:00:00 GPST").unwrap(),
        };

        let sv_list = [
            SV::from_str("G01").unwrap(),
            SV::from_str("G02").unwrap(),
            SV::from_str("G03").unwrap(),
            SV::from_str("G04").unwrap(),
            SV::from_str("G05").unwrap(),
            SV::from_str("G06").unwrap(),
            SV::from_str("G07").unwrap(),
            SV::from_str("G08").unwrap(),
            SV::from_str("G09").unwrap(),
            SV::from_str("G10").unwrap(),
        ];

        format_epoch_v2(&mut buf, &key, &sv_list, None).unwrap();

        let content = buf.into_inner().unwrap().to_ascii_utf8();

        assert_eq!(
            content,
            " 2021  1  1  0  0  0.0000000  0 10G03G08G14G16G22G23G26G27G31G32\n",
        );
    }
}
