//! RINEX compression module

use crate::{
    epoch::epoch_decompose as epoch_decomposition,
    error::FormattingError,
    hatanaka::{NumDiff, TextDiff},
    observation::{HeaderFields, Record},
    prelude::{Observable, SV},
    BufWriter,
};

use std::{collections::HashMap, io::Write};

use itertools::Itertools;

pub type Compressor = CompressorExpert<5>;

pub struct CompressorExpert<const M: usize> {
    /// True when epoch descriptor should be compressed.
    /// True on first epoch.
    epoch_compression: bool,
    /// Readable Epoch being compressed
    epoch_buf: String,
    /// Readable flags being compressed
    flags_buf: String,
    /// Epoch [TextDiff]
    epoch_diff: TextDiff,
    /// Flags kernel, per SV
    flags_diff: HashMap<SV, TextDiff>,
    /// Flag textdiff
    /// Compression kernels (per SV and signal)
    sv_kernels: HashMap<(SV, Observable), NumDiff<M>>,
    // /// Clock [NumDiff]
    // clock_diff: NumDiff<M>,
}

impl<const M: usize> Default for CompressorExpert<M> {
    fn default() -> Self {
        Self {
            epoch_compression: true,
            epoch_diff: TextDiff::new(""),
            epoch_buf: String::with_capacity(128),
            flags_buf: String::with_capacity(128),
            sv_kernels: HashMap::with_capacity(8),
            flags_diff: HashMap::with_capacity(8),
        }
    }
}

impl<const M: usize> CompressorExpert<M> {
    /// Format [Record] using mutable [CompressorExpert].
    /// Compressed bytes are dumped in mutable [BufWriter].
    /// This permits the RNX2CRX compression ops.
    pub fn format<W: Write>(
        &mut self,
        w: &mut BufWriter<W>,
        record: &Record,
        header: &HeaderFields,
    ) -> Result<(), FormattingError> {
        for (k, v) in record.iter() {
            // kernel reset on phase lock loss ?
            //if k.flags.intersects(LliFlags::LOCK_LOSS || LliFlags::HALF_CYCLE_SLIP) {
            //}

            let (y, m, d, hh, mm, ss, ns) = epoch_decomposition(k.epoch);

            // form unique SV list
            let svnn = v
                .signals
                .iter()
                .map(|sig| sig.sv)
                .unique()
                .sorted()
                .collect::<Vec<_>>();

            self.epoch_buf = format!(
                "{:04} {:02} {:02} {:04} {:04} {:04}.{:07} {}{:3}      ",
                y,
                m,
                d,
                hh,
                mm,
                ss,
                ns / 100,
                k.flag,
                svnn.len(),
            );

            // Append each SV to epoch description
            for sv in svnn.iter() {
                self.epoch_buf.push_str(&format!("{:x}", sv));
            }

            // Epoch compression
            if self.epoch_compression {
                writeln!(w, "&{}", self.epoch_buf)?;
            } else {
                let compressed = self.epoch_diff.compress(&self.epoch_buf);
                writeln!(w, " {}", compressed)?;
            }

            if let Some(clk) = v.clock {
                // TODO: this needs its own compressor
                write!(w, "{:14.3}\n", clk.offset_s)?;
            } else {
                // No clock: BLANKed line
                write!(w, "{}", '\n')?;
            }

            // For each SV
            for sv in svnn.iter() {
                // Following header specs
                let observables = header
                    .codes
                    .get(&sv.constellation)
                    .ok_or(FormattingError::MissingObservableDefinition)?;

                for observable in observables.iter() {
                    if let Some(signal) = v
                        .signals
                        .iter()
                        .filter(|sig| sig.sv == *sv && &sig.observable == observable)
                        .reduce(|k, _| k)
                    {
                        let quantized = (signal.value * 1000.0).round() as i64;

                        // retrieve or build compression kernel
                        if let Some((_, sv_kernel)) = self
                            .sv_kernels
                            .iter_mut()
                            .filter(|((sv, obs), _)| *sv == signal.sv && obs == &signal.observable)
                            .reduce(|k, _| k)
                        {
                            let compressed = sv_kernel.compress(quantized);
                            write!(w, "{}", compressed)?;
                        } else {
                            // first encounter: build kernel
                            let kernel = NumDiff::<M>::new(quantized, 3);
                            self.sv_kernels
                                .insert((signal.sv, signal.observable.clone()), kernel);

                            write!(w, "{}&{:14.3}", 3, signal.value)?;
                        }

                        if let Some(lli) = signal.lli {
                            self.flags_buf.push_str(&format!("{}", lli.bits() as u8));
                        }

                        if let Some(snr) = signal.snr {
                            self.flags_buf.push_str(&format!("{}", snr as u8));
                        }
                    } else {
                        // BLANK is a single ' '
                        write!(w, "{}", ' ')?;
                        self.flags_buf.push_str("  ");
                    }
                }

                // Flags compression
                if let Some(flags_kernel) = self.flags_diff.get_mut(&sv) {
                    let compressed = flags_kernel.compress(&self.flags_buf);
                    write!(w, "{}", compressed)?;
                } else {
                    let kernel = TextDiff::new(&self.flags_buf);
                    self.flags_diff.insert(*sv, kernel);
                    write!(w, "{}", self.flags_buf)?;
                }

                self.flags_buf.clear();
            }

            self.epoch_compression = false;
        }
        Ok(())
    }
}
