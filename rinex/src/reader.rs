//! Buffered Reader wrapper, for efficient data reading
//! and integrated .gz decompression.
#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;

use std::io::{BufRead, Error as IoError, Read};

/// [BufferedReader] is an interface reader that adapts to all our usecases.
/// RINEX formats are Line Termination based and oftentimes compressed.
/// The Hatanaka compression scheme was developped specifically for RINEX Observation format (heavy files).
/// Gzip decompression is also natively supported in case it was compiled.
/// Hatanaka + Gzip compression is used in most RINEX Observation production contexts.
/// [BufferedReader] allows seamless RINEX iteration by providing [BufRead] implementation in all cases.
#[derive(Debug)]
pub enum BufferedReader<R: Read> {
    /// Readable data
    Plain(R),
    /// Gzip compressed data (non readable)
    #[cfg(feature = "flate2")]
    Gz(GzDecoder<R>),
}

impl<R: Read> BufferedReader<R> {
    pub fn plain(r: R) -> Self {
        Self::Plain(r)
    }
    #[cfg(feature = "flate2")]
    pub fn gzip(r: R) -> Self {
        Self::Gz(GzDecoder::new(r))
    }
}

impl<R: Read> Read for BufferedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        match self {
            Self::Plain(r) => r.read(buf),
            #[cfg(feature = "flate2")]
            Self::Gz(r) => r.read(buf),
        }
    }
}

/// Providing [BufRead] implementation, facilitates the file consideration a lot.
impl<R: Read> BufRead for BufferedReader<R> {
    fn fill_buf(&mut self) -> Result<&[u8], IoError> {
        match self {
            Self::Plain(r) => r.fill_buf(),
            #[cfg(feature = "flate2")]
            Self::GzFile(r) => r.fill_buf(),
        }
    }
    fn consume(&mut self, s: usize) {
        match self {
            Self::Plain(r) => r.consume(s),
            #[cfg(feature = "flate2")]
            Self::GzFile(r) => r.consume(s),
        }
    }
}
