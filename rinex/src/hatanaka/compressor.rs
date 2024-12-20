//! RINEX compression module

use std::io::Write;

use crate::{
    epoch::epoch_decompose as epoch_decomposition,
    error::FormattingError,
    hatanaka::{Error, NumDiff, TextDiff},
    observation::{HeaderFields, Record},
    prelude::{Constellation, SV},
    BufWriter,
};

use itertools::Itertools;

pub type Compressor = CompressorExpert<5>;

pub struct CompressorExpert<const M: usize> {
    epoch_reinit: bool,
    /// Epoch being compressed
    epoch_buf: String,
    /// Epoch [TextDiff]
    epoch_diff: TextDiff,
    /// Clock [NumDiff]
    clock_diff: NumDiff<M>,
}

impl<const M: usize> Default for CompressorExpert<M> {
    fn default() -> Self {
        Self {
            epoch_reinit: true,
            epoch_buf: String::with_capacity(128),
            epoch_diff: TextDiff::new(""),
            clock_diff: NumDiff::<M>::new(0, 3),
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
            let (y, m, d, hh, mm, ss, nanos) = epoch_decomposition(k.epoch);

            // form unique SV list
            let svnn = v
                .signals
                .iter()
                .map(|sig| sig.sv)
                .unique()
                .collect::<Vec<_>>();

            self.epoch_buf.clear();

            self.epoch_buf = format!(
                "{:04} {:02} {:02} {:04} {:04} {:04}.0000000  {}{:3}      ",
                y,
                m,
                d,
                hh,
                mm,
                ss,
                k.flag,
                svnn.len(),
            );

            if self.epoch_reinit {
                write!(w, "> ")?;
                writeln!(w, "{}", self.epoch_buf)?;
            } else {
                let compressed = self.epoch_diff.compress(&self.epoch_buf);
                writeln!(w, "{}", compressed)?;
            }

            if let Some(clk) = v.clock {
                write!(w, "{}", '\n')?;
            } else {
                write!(w, "{}", '\n')?;
            }

            for sv in svnn.iter() {
                // following header specs
                if let Some(observables) = header.codes.get(&sv.constellation) {
                    for observable in observables.iter() {
                        if let Some(observation) = v
                            .signals
                            .iter()
                            .filter(|sig| sig.sv == *sv && &sig.observable == observable)
                            .reduce(|k, _| k)
                        {
                            write!(w, "3&")?;
                            write!(w, "1111111111 ")?;
                        } else {
                            // BLANK
                            write!(w, "{}", ' ')?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
