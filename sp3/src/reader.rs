//! Buffered Reader wrapper, for efficient data reading
//! and integrated .gz decompression.
#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::BufReader; // Seek, SeekFrom};

#[derive(Debug)]
pub enum BufferedReader {
    /// Readable (plain) file
    PlainFile(BufReader<File>),
    /// gzip compressed RINEX
    #[cfg(feature = "flate2")]
    GzFile(BufReader<GzDecoder<File>>),
}

impl BufferedReader {
    /// Builds a new BufferedReader for efficient file interation,
    /// with possible .gz decompression
    pub fn new(path: &str) -> std::io::Result<Self> {
        let f = File::open(path)?;
        if path.ends_with(".gz") {
            // --> gzip encoded
            #[cfg(feature = "flate2")]
            {
                Ok(Self::GzFile(BufReader::new(GzDecoder::new(f))))
            }
            #[cfg(not(feature = "flate2"))]
            {
                panic!(".gz data requires --flate2 feature")
            }
        } else if path.ends_with(".Z") {
            panic!(".z decompresion is not supported: uncompress manually")
        } else {
            // Assumes no extra compression
            Ok(Self::PlainFile(BufReader::new(f)))
        }
    }
}

impl std::io::Read for BufferedReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        match self {
            Self::PlainFile(ref mut h) => h.read(buf),
            #[cfg(feature = "flate2")]
            Self::GzFile(ref mut h) => h.read(buf),
        }
    }
}

impl std::io::BufRead for BufferedReader {
    fn fill_buf(&mut self) -> Result<&[u8], std::io::Error> {
        match self {
            Self::PlainFile(ref mut bufreader) => bufreader.fill_buf(),
            #[cfg(feature = "flate2")]
            Self::GzFile(ref mut bufreader) => bufreader.fill_buf(),
        }
    }
    fn consume(&mut self, s: usize) {
        match self {
            Self::PlainFile(ref mut bufreader) => bufreader.consume(s),
            #[cfg(feature = "flate2")]
            Self::GzFile(ref mut bufreader) => bufreader.consume(s),
        }
    }
}
