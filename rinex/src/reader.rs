//! Generic Buffered Reader, for efficient record iteration,
//! with powerful internal Hatanaka / Gz decompression.
use std::io::{BufReader, Seek, SeekFrom};
use crate::hatanaka::Decompressor;
#[cfg(feature = "with-gzip")]
use flate2::read::GzDecoder;


#[derive(Debug)]
pub enum ReaderWrapper {
    /// Readable `RINEX`
    PlainFile(BufReader<std::fs::File>),
    /// gzip compressed RINEX
    #[cfg(feature = "with-gzip")]
    GzFile(BufReader<GzDecoder<std::fs::File>>),
    // /// zlib compressed RINEX
    // #[cfg(feature = "with-gzip")]
    // ZlibFile(BufReader<ZlibDecoder<std::fs::File>>),
}

pub struct BufferedReader {
    /// Internal reader,
    /// supports Plain RINEX, CRINEX, .gz
    reader: ReaderWrapper,
    /// Internal struct in case of CRINEX decompression 
    decompressor: Option<Decompressor>,
}

impl BufferedReader {
    /// Builds a new BufferedReader for efficient file interation,
    /// with possible .gz and .gz + hatanaka decompression
    pub fn new (path: &str) -> std::io::Result<Self> {
        let f = std::fs::File::open(path)?;
        if path.ends_with(".gz") {
            // --> gzip encoded
            #[cfg(feature = "with-gzip")] {
                // .gz
                // example : i.gz, .n.gz, .crx.gz 
                Ok(Self {
                    reader: ReaderWrapper::GzFile(BufReader::new(GzDecoder::new(f))),
                    decompressor: None,
                })
            }
            #[cfg(not(feature = "with-gzip"))] {
                panic!("gzip compressed data require the --with-gzip build feature")
            }
        
        } else if path.ends_with(".Z") { // unix/lz4??
            panic!(".Z file not supported yet, uncompress manuallyl first")
        
        } else { // Assumes no extra compression
            Ok(Self {
                reader: ReaderWrapper::PlainFile(BufReader::new(f)),
                decompressor: None,
            })
        }
    }
    /// Enhances self for hatanaka internal decompression,
    /// preserves inner pointer state
    pub fn with_hatanaka (&self, m: usize) -> std::io::Result<Self> {
        match &self.reader {
            ReaderWrapper::PlainFile(bufreader) => {
                let inner = bufreader.get_ref();
                let fd = inner.try_clone()?; // preserves pointer
                Ok(BufferedReader {
                    reader: ReaderWrapper::PlainFile(BufReader::new(fd)),
                    decompressor: Some(Decompressor::new(m)),
                })
            },
            #[cfg(feature = "with-gzip")]
            ReaderWrapper::GzFile(bufreader) => {
                let inner = bufreader.get_ref().get_ref();
                let fd = inner.try_clone()?; // preserves pointer
                Ok(BufferedReader {
                    reader: ReaderWrapper::GzFile(BufReader::new(GzDecoder::new(fd))),
                    decompressor: Some(Decompressor::new(m)),
                })
            },
        }
    }
    /// Modifies inner file pointer position
    pub fn seek (&mut self, pos: SeekFrom) -> Result<u64, std::io::Error> {
        match self.reader {
            ReaderWrapper::PlainFile(ref mut bufreader) => bufreader.seek(pos),
            #[cfg(feature = "with-gzip")]
            ReaderWrapper::GzFile(ref mut bufreader) => bufreader.seek(pos),
        }
    }
/*
    /// rewind filer inner pointer, to offset = 0 
    pub fn rewind (&mut self) -> Result<(), std::io::Error> {
        match self.reader {
            ReaderWrapper::PlainFile(ref mut bufreader) => bufreader.rewind(),
            #[cfg(feature = "with-gzip")]
            ReaderWrapper::GzFile(ref mut bufreader) => bufreader.rewind(),
        }
    }
*/
}

impl std::io::Read for BufferedReader {
    fn read (&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> { 
        match self.reader {
            ReaderWrapper::PlainFile(ref mut h) => h.read(buf),
            #[cfg(feature = "with-gzip")]
            ReaderWrapper::GzFile(ref mut h) => h.read(buf),
        }
    }
}

impl std::io::BufRead for BufferedReader {
    fn fill_buf (&mut self) -> Result<&[u8], std::io::Error> { 
        match self.reader {
            ReaderWrapper::PlainFile(ref mut bufreader) => bufreader.fill_buf(),
            #[cfg(feature = "with-gzip")]
            ReaderWrapper::GzFile(ref mut bufreader) => bufreader.fill_buf(),
        }
    }
    
    fn consume (&mut self, s: usize) { 
        match self.reader {
            ReaderWrapper::PlainFile(ref mut bufreader) => bufreader.consume(s),
            #[cfg(feature = "with-gzip")]
            ReaderWrapper::GzFile(ref mut bufreader) => bufreader.consume(s),
        }
    }
}
