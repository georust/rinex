// use log::{debug, error};
use std::io::{
    Result as IoResult,
    //Error as IoError,
    Write,
};

#[cfg(feature = "flate2")]
use flate2::{write::GzEncoder, Compression as GzCompression};

/// Abstraction for Plain or Compressed [R]
enum Writer<W: Write> {
    Plain(W),
    #[cfg(feature = "flate2")]
    Compressed(GzEncoder<W>),
}

impl<W: Write> From<W> for Writer<W> {
    fn from(w: W) -> Writer<W> {
        Self::Plain(w)
    }
}

#[cfg(feature = "flate2")]
impl<W: Write> From<GzEncoder<W>> for Writer<W> {
    fn from(w: GzEncoder<W>) -> Writer<W> {
        Self::Compressed(w)
    }
}

impl<W: Write> Write for Writer<W> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        match self {
            Self::Plain(w) => w.write(buf),
            #[cfg(feature = "flate2")]
            Self::Compressed(w) => w.write(buf),
        }
    }
    fn flush(&mut self) -> IoResult<()> {
        match self {
            Self::Plain(w) => w.flush(),
            #[cfg(feature = "flate2")]
            Self::Compressed(w) => w.flush(),
        }
    }
}

/// [BINEX] Stream Encoder.
pub struct Encoder<W: Write> {
    /// [W]
    writer: Writer<W>,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer: writer.into(),
        }
    }
    #[cfg(feature = "flate2")]
    pub fn new_gzip(writer: W, compression_level: u32) -> Self {
        Self {
            writer: GzEncoder::new(writer, GzCompression::new(compression_level)).into(),
        }
    }
}
