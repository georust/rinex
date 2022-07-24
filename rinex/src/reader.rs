//! Generic Buffered Reader
use std::io::{prelude::*, BufReader};

#[cfg(feature = "with-gzip")]
use flate2::read::{GzDecoder, ZlibDecoder};

#[derive(Debug)]
pub enum BufferedReader {
    File(BufReader<std::fs::File>),
    #[cfg(feature = "with-gzip")]
    GzFile(BufReader<GzDecoder<std::fs::File>>),
}

impl BufferedReader {
    pub fn new (path: &str) -> std::io::Result<BufferedReader> {
        let f = std::fs::File::open(path)?;
        if path.ends_with(".gz") {
            // --> gzip encoded
            if !cfg!(feature = "with-gzip") {
                panic!("gzip compressed data require the --with-gzip build feature")
            }
            Ok(Self::GzFile(BufReader::new(GzDecoder::new(f))))
        //} else if path.ends_with(".Z") {
            // --> zlib encoded
        } else {
            // assume uncompress or hatanaka compressed
            Ok(Self::File(BufReader::new(f)))
        }
    }
}

impl std::io::Read for BufferedReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> { 
        match self {
            Self::File(h) => h.read(buf),
            Self::GzFile(h) => h.read(buf),
        }
    }
}

impl std::io::BufRead for BufferedReader {
    fn fill_buf (&mut self) -> Result<&[u8], std::io::Error> { 
        match self {
            Self::File(h) => h.fill_buf(),
            Self::GzFile(h) => h.fill_buf(),
        }
    }
    fn consume (&mut self, s: usize) { 
        match self {
            Self::File(h) => h.consume(s),
            Self::GzFile(h) => h.consume(s),
        }
    }
}
