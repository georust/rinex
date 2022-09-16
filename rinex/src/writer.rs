//! Generic Buffered Writer, for efficient record production,
//! with integrated optionnal .gz compression
use std::fs::File;
use std::io::BufWriter; // Seek, SeekFrom};
#[cfg(feature = "flate2")]
use flate2::{Compression, write::GzEncoder};

#[derive(Debug)]
pub enum WriterWrapper {
    /// Readable `RINEX`
    PlainFile(BufWriter<File>),
    /// gzip compressed RINEX
    #[cfg(feature = "flate2")]
    GzFile(BufWriter<GzEncoder<File>>),
}

pub struct BufferedWriter {
    /// internal writer,
    writer: WriterWrapper,
}

impl BufferedWriter {
	/// Opens given file for efficient buffered write operation
	/// with possible .gz compression
    pub fn new (path: &str) -> std::io::Result<Self> {
        let f = std::fs::File::create(path)?;
        if path.ends_with(".gz") {
            // --> .gz compression
            #[cfg(feature = "flate2")] {
                // .gz
                // example : i.gz, .n.gz, .crx.gz 
                Ok(Self {
                    writer: WriterWrapper::GzFile( // compression lvl 6 seems to be the optimal standard
						BufWriter::new(GzEncoder::new(f, Compression::new(6)))), 
                })
            }
            #[cfg(not(feature = "flate2"))] {
                panic!(".gz data requires --flate2 feature")
            }
        
        } else if path.ends_with(".Z") {
            panic!(".z compression is not supported yet, compress manually")
        
        } else { // Assumes no extra compression
            Ok(Self {
                writer: WriterWrapper::PlainFile(BufWriter::new(f)),
            })
        }
    }
}

impl std::io::Write for BufferedWriter {
	fn write (&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
		match self.writer {
			WriterWrapper::PlainFile(ref mut writer) => writer.write(buf),
    		#[cfg(feature = "flate2")]
			WriterWrapper::GzFile(ref mut writer) => writer.write(buf),
		}
	}
	fn flush (&mut self) -> Result<(), std::io::Error> {
		match self.writer {
			WriterWrapper::PlainFile(ref mut writer) => writer.flush(),
    		#[cfg(feature = "flate2")]
			WriterWrapper::GzFile(ref mut writer) => writer.flush(),
		}
	}
}
