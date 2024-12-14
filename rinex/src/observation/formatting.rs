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
        if nth > 0 && (nth % NUM_SV_PER_LINE) == 0 {
            write!(w, "                                             ")?;
        }
        write!(w, "{:x}", sv)?;
        if nth < numsat - 1 && nth % NUM_SV_PER_LINE == NUM_SV_PER_LINE - 1 {
            write!(w, "{}", '\n')?;
        }
        if nth == numsat - 1 {
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
        let key = ObsKey {
            flag: EpochFlag::Ok,
            epoch: Epoch::from_str("2017-01-01T00:00:00 GPST").unwrap(),
        };

        let sv_list = [
            SV::from_str("G03").unwrap(),
            SV::from_str("G08").unwrap(),
            SV::from_str("G14").unwrap(),
            SV::from_str("G16").unwrap(),
            SV::from_str("G22").unwrap(),
            SV::from_str("G23").unwrap(),
            SV::from_str("G26").unwrap(),
            SV::from_str("G27").unwrap(),
            SV::from_str("G31").unwrap(),
            SV::from_str("G32").unwrap(),
        ];

        let mut buf = BufWriter::new(Utf8Buffer::new(1024));
        format_epoch_v2(&mut buf, &key, &sv_list, None).unwrap();

        let content = buf.into_inner().unwrap().to_ascii_utf8();
        assert_eq!(
            content,
            " 17  1  1  0  0  0.0000000  0 10G03G08G14G16G22G23G26G27G31G32\n",
        );

        let sv_list = [
            SV::from_str("G07").unwrap(),
            SV::from_str("G23").unwrap(),
            SV::from_str("G26").unwrap(),
            SV::from_str("G20").unwrap(),
            SV::from_str("G21").unwrap(),
            SV::from_str("G18").unwrap(),
            SV::from_str("R24").unwrap(),
            SV::from_str("R09").unwrap(),
            SV::from_str("G08").unwrap(),
            SV::from_str("G27").unwrap(),
            SV::from_str("G10").unwrap(),
            SV::from_str("G16").unwrap(),
            SV::from_str("R18").unwrap(),
            SV::from_str("G13").unwrap(),
            SV::from_str("R01").unwrap(),
            SV::from_str("R16").unwrap(),
            SV::from_str("R17").unwrap(),
            SV::from_str("G15").unwrap(),
            SV::from_str("R02").unwrap(),
            SV::from_str("R15").unwrap(),
        ];

        let mut buf = BufWriter::new(Utf8Buffer::new(1024));
        format_epoch_v2(&mut buf, &key, &sv_list, None).unwrap();

        let content = buf.into_inner().unwrap().to_ascii_utf8();
        assert_eq!(
            content,
            " 17  1  1  0  0  0.0000000  0 20G07G23G26G20G21G18R24R09G08G27G10G16
                                             R18G13R01R16R17G15R02R15\n",
        );

        let sv_list = [
            SV::from_str("G07").unwrap(),
            SV::from_str("G23").unwrap(),
            SV::from_str("G26").unwrap(),
            SV::from_str("G20").unwrap(),
            SV::from_str("G21").unwrap(),
            SV::from_str("G18").unwrap(),
            SV::from_str("R24").unwrap(),
            SV::from_str("R09").unwrap(),
            SV::from_str("G08").unwrap(),
            SV::from_str("G27").unwrap(),
            SV::from_str("G10").unwrap(),
            SV::from_str("G16").unwrap(),
            SV::from_str("R18").unwrap(),
        ];

        let mut buf = BufWriter::new(Utf8Buffer::new(1024));
        format_epoch_v2(&mut buf, &key, &sv_list, None).unwrap();

        let content = buf.into_inner().unwrap().to_ascii_utf8();
        assert_eq!(
            content,
            " 17  1  1  0  0  0.0000000  0 13G07G23G26G20G21G18R24R09G08G27G10G16
                                             R18\n",
        );

        let sv_list = [
            SV::from_str("G07").unwrap(),
            SV::from_str("G23").unwrap(),
            SV::from_str("G26").unwrap(),
            SV::from_str("G20").unwrap(),
            SV::from_str("G21").unwrap(),
            SV::from_str("G18").unwrap(),
            SV::from_str("R24").unwrap(),
            SV::from_str("R09").unwrap(),
            SV::from_str("G08").unwrap(),
            SV::from_str("G27").unwrap(),
            SV::from_str("G10").unwrap(),
            SV::from_str("G16").unwrap(),
            SV::from_str("R18").unwrap(),
            SV::from_str("G13").unwrap(),
            SV::from_str("R01").unwrap(),
            SV::from_str("R16").unwrap(),
            SV::from_str("R17").unwrap(),
            SV::from_str("G15").unwrap(),
            SV::from_str("R02").unwrap(),
            SV::from_str("R15").unwrap(),
            SV::from_str("C01").unwrap(),
            SV::from_str("C02").unwrap(),
            SV::from_str("C03").unwrap(),
            SV::from_str("C04").unwrap(),
        ];

        let mut buf = BufWriter::new(Utf8Buffer::new(1024));
        format_epoch_v2(&mut buf, &key, &sv_list, None).unwrap();

        let content = buf.into_inner().unwrap().to_ascii_utf8();
        assert_eq!(
            content,
            " 17  1  1  0  0  0.0000000  0 24G07G23G26G20G21G18R24R09G08G27G10G16
                                             R18G13R01R16R17G15R02R15C01C02C03C04\n",
        );

        let sv_list = [
            SV::from_str("G07").unwrap(),
            SV::from_str("G23").unwrap(),
            SV::from_str("G26").unwrap(),
            SV::from_str("G20").unwrap(),
            SV::from_str("G21").unwrap(),
            SV::from_str("G18").unwrap(),
            SV::from_str("R24").unwrap(),
            SV::from_str("R09").unwrap(),
            SV::from_str("G08").unwrap(),
            SV::from_str("G27").unwrap(),
            SV::from_str("G10").unwrap(),
            SV::from_str("G16").unwrap(),
            SV::from_str("R18").unwrap(),
            SV::from_str("G13").unwrap(),
            SV::from_str("R01").unwrap(),
            SV::from_str("R16").unwrap(),
            SV::from_str("R17").unwrap(),
            SV::from_str("G15").unwrap(),
            SV::from_str("R02").unwrap(),
            SV::from_str("R15").unwrap(),
            SV::from_str("C01").unwrap(),
            SV::from_str("C02").unwrap(),
            SV::from_str("C03").unwrap(),
            SV::from_str("C04").unwrap(),
            SV::from_str("C05").unwrap(),
        ];

        let mut buf = BufWriter::new(Utf8Buffer::new(1024));
        format_epoch_v2(&mut buf, &key, &sv_list, None).unwrap();

        let content = buf.into_inner().unwrap().to_ascii_utf8();
        assert_eq!(
            content,
            " 17  1  1  0  0  0.0000000  0 25G07G23G26G20G21G18R24R09G08G27G10G16
                                             R18G13R01R16R17G15R02R15C01C02C03C04
                                             C05\n",
        );
    }

    #[test]
    fn test_fmt_epoch_v3() {
        let key = ObsKey {
            flag: EpochFlag::Ok,
            epoch: Epoch::from_str("2021-01-01T00:00:00 GPST").unwrap(),
        };

        let sv_list = [
            SV::from_str("G01").unwrap(),
            SV::from_str("G02").unwrap(),
            SV::from_str("G03").unwrap(),
            SV::from_str("G04").unwrap(),
        ];

        let mut buf = BufWriter::new(Utf8Buffer::new(1024));
        format_epoch_v3(&mut buf, &key, &sv_list, None).unwrap();
        let content = buf.into_inner().unwrap().to_ascii_utf8();
        assert_eq!(content, "> 2021 01 01 00 00  0.0000000  0  4\n",);

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

        let mut buf = BufWriter::new(Utf8Buffer::new(1024));
        format_epoch_v3(&mut buf, &key, &sv_list, None).unwrap();
        let content = buf.into_inner().unwrap().to_ascii_utf8();
        assert_eq!(content, "> 2021 01 01 00 00  0.0000000  0 10\n",);
    }
}
