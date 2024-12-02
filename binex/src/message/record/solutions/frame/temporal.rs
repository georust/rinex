//! Monument Geodetic marker specific frames
use crate::{utils::Utils, Error};

#[derive(Debug, Clone, PartialEq)]
pub struct TemporalSolution {
    /// Offset to system time [s]
    pub offset_s: f64,
    /// Drift in system time [s/s]
    pub drift_s_s: Option<f64>,
}

impl TemporalSolution {
    pub(crate) fn encoding_size(&self) -> usize {
        if self.drift_s_s.is_some() {
            16
        } else {
            8
        }
    }
    pub(crate) fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.encoding_size();
        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        let bytes = if big_endian {
            self.offset_s.to_be_bytes()
        } else {
            self.offset_s.to_le_bytes()
        };

        buf[..8].copy_from_slice(&bytes);

        if let Some(drift) = self.drift_s_s {
            let bytes = if big_endian {
                drift.to_be_bytes()
            } else {
                drift.to_le_bytes()
            };
            buf[8..16].copy_from_slice(&bytes);
        }

        Ok(size)
    }

    pub(crate) fn decode_without_drift(big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        if buf.len() < 8 {
            return Err(Error::NotEnoughBytes);
        }
        let offset_s = Utils::decode_f64(big_endian, buf)?;
        Ok(Self {
            offset_s,
            drift_s_s: None,
        })
    }

    pub(crate) fn decode_with_drift(big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        if buf.len() < 16 {
            return Err(Error::NotEnoughBytes);
        }
        let offset_s = Utils::decode_f64(big_endian, buf)?;
        let drift_s_s = Utils::decode_f64(big_endian, &buf[8..])?;
        Ok(Self {
            offset_s,
            drift_s_s: Some(drift_s_s),
        })
    }
}
