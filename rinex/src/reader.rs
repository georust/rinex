//! Generic Buffered Reader
use std::io::BufReader;
#[cfg(feature = "with-gzip")]
use flate2::read::GzDecoder;

#[derive(Debug)]
pub enum BufferedReader {
    /// Readable `RINEX`
    PlainFile(BufReader<std::fs::File>),
    // /// Hatana Compressed RINEX
    // Hatanaka(BufReader<std::fs::File>),
    /// gzip compressed RINEX
    #[cfg(feature = "with-gzip")]
    GzFile(BufReader<GzDecoder<std::fs::File>>),
    // /// zlib compressed RINEX
    // #[cfg(feature = "with-gzip")]
    // ZlibFile(BufReader<ZlibDecoder<std::fs::File>>),
    // /// gzip + Hatanaka compressed RINEX
    // #[cfg(feature = "with-gzip")]
    // GzHatanakaFile(BufReader<GzDecoder<std::fs::File>>),
    // /// zlib + Hatanaka compressed RINEX
    // #[cfg(feature = "with-gzip")]
    // ZlibHatanakaFile(BufReader<ZlibDecoder<std::fs::File>>),
}

impl BufferedReader {
    pub fn new (path: &str, hatanaka: bool) -> std::io::Result<BufferedReader> {
        let f = std::fs::File::open(path)?;
        if path.ends_with(".gz") {
            // --> gzip encoded
            #[cfg(feature = "with-gzip")] {
                Ok(Self::GzFile(BufReader::new(GzDecoder::new(f))))
            }
            #[cfg(not(feature = "with-gzip"))] {
                panic!("gzip compressed data require the --with-gzip build feature")
            }
        
        } else if path.ends_with(".Z") { // unix/lz4??
            panic!(".Z file not supported yet, uncompress manuallyl first")
        
        } else { // Assumes uncompressed file
            Ok(Self::PlainFile(BufReader::new(f)))
        }
    }
}

impl std::io::Read for BufferedReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> { 
        match self {
            Self::PlainFile(h) => h.read(buf),
            #[cfg(feature = "with-gzip")]
            Self::GzFile(h) => h.read(buf),
        }
    }
}

impl std::io::BufRead for BufferedReader {
    fn fill_buf (&mut self) -> Result<&[u8], std::io::Error> { 
        match self {
            Self::PlainFile(h) => h.fill_buf(),
            #[cfg(feature = "with-gzip")]
            Self::GzFile(h) => h.fill_buf(),
        }
    }
    fn consume (&mut self, s: usize) { 
        match self {
            Self::PlainFile(h) => h.consume(s),
            #[cfg(feature = "with-gzip")]
            Self::GzFile(h) => h.consume(s),
        }
    }
}
