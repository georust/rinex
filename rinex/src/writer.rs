//! Generic Buffered Writer, for efficient record production,
//! with integrated optionnal .gz compression
use std::io::{BufWrite, BufWriter}; // Seek, SeekFrom};

#[cfg(feature = "flate2")]
use flate2::{bufwrite::GzEncoder, Compression};

/// [BufferedWriter] is an Output abstraction to propose seamless
/// Gzip, Readable and Hatanaka compressed content streaming.
#[derive(Debug)]
pub enum BufferedWriter<BW: BufWrite> {
    /// Readable stream
    Plain(BufWriter<File>),
    /// Gzip compressed stream (non readable)
    #[cfg(feature = "flate2")]
    Gz(BufWriter<GzEncoder<BW>>),
}

impl<BW: BufWrite> BufferedWriter<BW> {
    pub fn plain(w: BW) -> Self {
        Self::Plain(w)
    }
    #[cfg(feature = "flate2")]
    pub fn gzip(w: BW) -> Self {
        Self::Gz(BufWriter::new(GzEncoder::new(w)))
    }
}

impl Write for BufferedWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        match self {
            Self::Plain(ref mut writer) => writer.write(buf),
            #[cfg(feature = "flate2")]
            Self::Gz(ref mut writer) => writer.write(buf),
        }
    }
    fn flush(&mut self) -> Result<(), std::io::Error> {
        match self.writer {
            Self::Plain(ref mut writer) => writer.flush(),
            #[cfg(feature = "flate2")]
            Self::Gz(ref mut writer) => writer.flush(),
        }
    }
}
