//! Buffered Reader wrapper, for efficient data reading
//! and integrated .gz decompression.
use std::fs::File;
use std::io::BufReader; // Seek, SeekFrom};
#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;

#[derive(Debug)]
pub enum BufferedReader {
    /// Readable `RINEX`
    PlainFile(BufReader<File>),
    /// gzip compressed RINEX
    #[cfg(feature = "flate2")]
    GzFile(BufReader<GzDecoder<File>>),
}

impl BufferedReader {
    /// Builds a new BufferedReader for efficient file interation,
    /// with possible .gz decompression
    pub fn new (path: &str) -> std::io::Result<Self> {
        let f = File::open(path)?;
        if path.ends_with(".gz") {
            // --> gzip encoded
            #[cfg(feature = "flate2")] {
                // .gz
                // example : i.gz, .n.gz, .crx.gz 
                Ok(Self::GzFile(BufReader::new(GzDecoder::new(f))))
            }
            #[cfg(not(feature = "flate2"))] {
                panic!(".gz data requires --flate2 feature")
            }
        
        } else if path.ends_with(".Z") {
            panic!(".z decompresion not supported yet, uncompress manually")
        
        } else { // Assumes no extra compression
            Ok(Self::PlainFile(BufReader::new(f)))
        }
    }
/*
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
*/
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
        match self {
            Self::PlainFile(ref mut h) => h.read(buf),
            #[cfg(feature = "flate2")]
            Self::GzFile(ref mut h) => h.read(buf),
        }
    }
}

impl std::io::BufRead for BufferedReader {
    fn fill_buf (&mut self) -> Result<&[u8], std::io::Error> { 
        match self {
            Self::PlainFile(ref mut bufreader) => bufreader.fill_buf(),
            #[cfg(feature = "flate2")]
            Self::GzFile(ref mut bufreader) => bufreader.fill_buf(),
        }
    }
    fn consume (&mut self, s: usize) { 
        match self {
            Self::PlainFile(ref mut bufreader) => bufreader.consume(s),
            #[cfg(feature = "flate2")]
            Self::GzFile(ref mut bufreader) => bufreader.consume(s),
        }
    }
}
