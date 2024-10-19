//! Buffered Reader wrapper, for efficient data reading and seamless decompression.
#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;

use crate::hatanaka::Decompressor;

use std::io::{BufRead, BufReader, Error as IoError, Read};

/// [BufferedReader] is an efficient BufRead implementer from any [Read]able interface.
/// It provides seamless Gzip and CRINEX decompression on any [Read]able interface.
/// This greatly facilitates the Parsing process, by providing [BufRead] implementation
/// for all scenarios.
pub enum BufferedReader<const M: usize, R: Read> {
    /// Readable stream
    Plain(BufReader<R>),
    /// Seamless Gzip compressed stream (non readable)
    #[cfg(feature = "flate2")]
    Gz(BufReader<GzDecoder<R>>),
    // Seamless Hatanaka compressed stream (non readable)
    CRINEX(BufReader<Decompressor<M, R>>),
    /// Seamless Gzip Hatanaka compressed stream (non readable)
    #[cfg(feature = "flate2")]
    GzCRINEX(BufReader<Decompressor<M, GzDecoder<R>>>),
}

impl<const M: usize, R: Read> BufferedReader<M, R> {
    pub fn plain(r: R) -> Self {
        Self::Plain(BufReader::new(r))
    }
    pub fn crinex(r: R) -> Self {
        Self::CRINEX(BufReader::new(Decompressor::new(r)))
    }
    #[cfg(feature = "flate2")]
    pub fn gzip(r: R) -> Self {
        Self::Gz(BufReader::new(GzDecoder::new(r)))
    }
    #[cfg(feature = "flate2")]
    pub fn gzip_crinex(r: R) -> Self {
        Self::GzCRINEX(BufReader::new(Decompressor::new(GzDecoder::new(r))))
    }
}

impl<const M: usize, R: Read> Read for BufferedReader<M, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        match self {
            Self::Plain(ref mut r) => r.read(buf),
            Self::CRINEX(ref mut r) => r.read(buf),
            #[cfg(feature = "flate2")]
            Self::Gz(ref mut r) => r.read(buf),
            #[cfg(feature = "flate2")]
            Self::GzCRINEX(ref mut r) => r.read(buf),
        }
    }
}

/**
 * Providing BufRead implementation for all types of streams
 * allows .lines() Iteration to become available
 * whatever the type of streams,
 * which facilitates de Parsing process for all types of streams.
 */
impl<const M: usize, R: Read> BufRead for BufferedReader<M, R> {
    fn fill_buf(&mut self) -> Result<&[u8], IoError> {
        match self {
            Self::Plain(r) => r.fill_buf(),
            #[cfg(feature = "flate2")]
            Self::Gz(r) => r.fill_buf(),
            Self::CRINEX(r) => r.fill_buf(),
            #[cfg(feature = "flate2")]
            Self::GzCRINEX(r) => r.fill_buf(),
        }
    }
    fn consume(&mut self, s: usize) {
        match self {
            Self::Plain(r) => r.consume(s),
            #[cfg(feature = "flate2")]
            Self::Gz(r) => r.consume(s),
            Self::CRINEX(r) => r.consume(s),
            #[cfg(feature = "flate2")]
            Self::GzCRINEX(r) => r.consume(s),
        }
    }
}
