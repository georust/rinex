//! Generic Buffered Reader, for efficient record iteration,
//! with powerful internal Hatanaka / Gz decompression.
use std::io::{BufReader}; // Seek, SeekFrom};
use crate::hatanaka::Decompressor;
#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;


#[derive(Debug)]
pub enum ReaderWrapper {
    /// Readable `RINEX`
    PlainFile(BufReader<std::fs::File>),
    /// gzip compressed RINEX
    #[cfg(feature = "flate2")]
    GzFile(BufReader<GzDecoder<std::fs::File>>),
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
            #[cfg(feature = "flate2")] {
                // .gz
                // example : i.gz, .n.gz, .crx.gz 
                Ok(Self {
                    reader: ReaderWrapper::GzFile(BufReader::new(GzDecoder::new(f))),
                    decompressor: None,
                })
            }
            #[cfg(not(feature = "flate2"))] {
                panic!("gzip compressed data require the --flate2 build feature")
            }
        
        } else if path.ends_with(".Z") {
            panic!(".z compressed files not supported yet, uncompress manually")
        
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
            #[cfg(feature = "flate2")]
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
/*
    /// Modifies inner file pointer position
    pub fn seek (&mut self, pos: SeekFrom) -> Result<u64, std::io::Error> {
        match self.reader {
            ReaderWrapper::PlainFile(ref mut bufreader) => bufreader.seek(pos),
            #[cfg(feature = "flate2")]
            ReaderWrapper::GzFile(ref mut bufreader) => bufreader.seek(pos),
        }
    }
    /// rewind filer inner pointer, to offset = 0 
    pub fn rewind (&mut self) -> Result<(), std::io::Error> {
        match self.reader {
            ReaderWrapper::PlainFile(ref mut bufreader) => bufreader.rewind(),
            #[cfg(feature = "flate2")]
            ReaderWrapper::GzFile(ref mut bufreader) => bufreader.rewind(),
        }
    }
*/
}

impl std::io::Read for BufferedReader {
    fn read (&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> { 
        match self.reader {
            ReaderWrapper::PlainFile(ref mut h) => h.read(buf),
            #[cfg(feature = "flate2")]
            ReaderWrapper::GzFile(ref mut h) => h.read(buf),
        }
    }
}

impl std::io::BufRead for BufferedReader {
    fn fill_buf (&mut self) -> Result<&[u8], std::io::Error> { 
        match self.reader {
            ReaderWrapper::PlainFile(ref mut bufreader) => bufreader.fill_buf(),
            #[cfg(feature = "flate2")]
            ReaderWrapper::GzFile(ref mut bufreader) => bufreader.fill_buf(),
        }
    }
    
    fn consume (&mut self, s: usize) { 
        match self.reader {
            ReaderWrapper::PlainFile(ref mut bufreader) => bufreader.consume(s),
            #[cfg(feature = "flate2")]
            ReaderWrapper::GzFile(ref mut bufreader) => bufreader.consume(s),
        }
    }
}
