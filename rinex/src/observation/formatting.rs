//! OBS RINEX formatting
use crate::{
    epoch::format as epoch_format,
    prelude::{ClockObservation, Constellation, Header, ObsKey, RinexType, SignalObservation},
};

#[cfg(feature = "log")]
use log::error;

/// Formats one epoch according to standard definitions.
/// ## Inputs
/// - major: RINEX revision major
/// - key: [ObsKey]
/// - clock: possible [ClockObservation]
/// - signals: [SignalObservation]s
/// ## Returns
/// - formatter string according to standard specifications
pub fn fmt_observations(
    major: u8,
    header: &Header,
    key: &ObsKey,
    clock: Option<&ClockObservation>,
    signals: Vec<&SignalObservation>,
) -> String {
    if major < 3 {
        fmt_observations_v2(header, key, clock, signals)
    } else {
        fmt_observations_v3(header, key, clock, signals)
    }
}

fn fmt_observations_v3(
    header: &Header,
    key: &ObsKey,
    clock: Option<&ClockObservation>,
    signals: Vec<&SignalObservation>,
) -> String {
    const EMPTY_FIELD: &str = "                ";
    let mut lines = String::with_capacity(128);

    let observables = &header
        .obs
        .as_ref()
        .expect("bad rinex: not observable definitions")
        .codes;

    let unique_sv = signals.iter().map(|sig| sig.sv).collect::<Vec<_>>();

    let num_sat = unique_sv.len();

    lines.push_str(&format!(
        "> {}  {} {:2}",
        epoch_format(key.epoch, RinexType::ObservationData, 3),
        key.epoch,
        num_sat,
    ));

    if let Some(clock) = clock {
        lines.push_str(&format!("{:13.4}", clock.offset_s));
    }

    lines.push('\n');

    for (nth_sv, sv) in unique_sv.iter().enumerate() {
        // retrieve observable specs
        let observables = if sv.constellation.is_sbas() {
            observables.get(&Constellation::SBAS)
        } else {
            observables.get(&sv.constellation)
        };

        if observables.is_none() {
            #[cfg(feature = "log")]
            error!("{}: missing obs specs", sv);
            continue;
        }

        lines.push_str(&sv.to_string());

        let observables = observables.unwrap();

        for observable in observables {
            if let Some(signal) = signals
                .iter()
                .filter(|sig| sig.sv == *sv && &sig.observable == observable)
                .reduce(|k, _| k)
            {
                lines.push_str(&format!("{:14.3}", signal.value));
                if let Some(lli) = signal.lli {
                    lines.push_str(&format!("{}", lli.bits()));
                } else {
                    lines.push(' ');
                }
                if let Some(snr) = signal.snr {
                    lines.push_str(&format!("{:x}", snr));
                } else {
                    lines.push(' ');
                }
            } else {
                lines.push_str(EMPTY_FIELD);
            }
        }

        if nth_sv < num_sat - 1 {
            lines.push('\n');
        }
    }
    lines
}

fn fmt_observations_v2(
    header: &Header,
    key: &ObsKey,
    clock: Option<&ClockObservation>,
    signals: Vec<&SignalObservation>,
) -> String {
    const NUM_SV_PER_LINE: usize = 12;
    const OBSERVATION_PER_LINE: usize = 5;
    const END_OF_LINE_PADDING: &str = "\n                                ";

    let mut lines = String::with_capacity(128);

    let observables = &header
        .obs
        .as_ref()
        .expect("bad rinex: not observable definitions")
        .codes;

    let unique_sv = signals.iter().map(|sig| sig.sv).collect::<Vec<_>>();

    let num_sat = unique_sv.len();

    lines.push_str(&format!(
        " {}  {} {:2}",
        epoch_format(key.epoch, RinexType::ObservationData, 2),
        key.flag,
        num_sat,
    ));

    // format the systems line
    let mut index = 0;
    for (nth_sv, sv) in unique_sv.iter().enumerate() {
        if index == NUM_SV_PER_LINE {
            index = 0;
            if nth_sv == 12 {
                // first line: append Clock (if any)
                if let Some(clock) = clock {
                    // push clock offsets
                    lines.push_str(&format!(" {:9.1}", clock.offset_s));
                }
            }

            lines.push_str(END_OF_LINE_PADDING);
        }

        lines.push_str(&format!(" {:x}", sv));
        index += 1;
    }

    for (nth_sv, sv) in unique_sv.iter().enumerate() {
        // retrieve header specs
        let observables = if sv.constellation.is_sbas() {
            observables.get(&Constellation::SBAS)
        } else {
            observables.get(&sv.constellation)
        };

        if observables.is_none() {
            #[cfg(feature = "log")]
            error!("{}: missing obs specs", sv.constellation);
            continue;
        }

        let observables = observables.unwrap();

        for (nth_obs, observable) in observables.iter().enumerate() {
            if nth_obs % OBSERVATION_PER_LINE == 0 {
                lines.push('\n');
            }
            if let Some(signal) = signals
                .iter()
                .filter(|sig| sig.sv == *sv && &sig.observable == observable)
                .reduce(|k, _| k)
            {
                lines.push_str(&format!("{:14.3}", signal.value));
                if let Some(lli) = signal.lli {
                    lines.push_str(&format!("{:x}", lli.bits()));
                } else {
                    lines.push(' ');
                }
                if let Some(snr) = signal.snr {
                    lines.push_str(&format!("{:x}", snr));
                } else {
                    lines.push(' ');
                }
            } else {
            }
        }
    }
    lines
}
