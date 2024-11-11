//! Buffered Reader wrapper, for efficient data reading and seamless decompression.
#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;

use crate::hatanaka::DecompressorExpert;

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
    CRINEX(BufReader<DecompressorExpert<M, R>>),
    /// Seamless Gzip Hatanaka compressed stream (non readable)
    #[cfg(feature = "flate2")]
    GzCRINEX(BufReader<DecompressorExpert<M, GzDecoder<R>>>),
}

impl<const M: usize, R: Read> BufferedReader<M, R> {
    pub fn plain(r: R) -> Self {
        Self::Plain(BufReader::new(r))
    }
    pub fn crinex(r: R) -> Self {
        Self::CRINEX(BufReader::new(DecompressorExpert::new(r)))
    }
    #[cfg(feature = "flate2")]
    pub fn gzip(r: R) -> Self {
        Self::Gz(BufReader::new(GzDecoder::new(r)))
    }
    #[cfg(feature = "flate2")]
    pub fn gzip_crinex(r: R) -> Self {
        Self::GzCRINEX(BufReader::new(DecompressorExpert::new(GzDecoder::new(r))))
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

#[cfg(test)]
mod test {
    use crate::reader::BufferedReader;
    use std::fs::File;
    use std::io::BufRead;
    #[test]
    fn plain_reader() {
        let fp = File::open("../test_resources/OBS/V2/AJAC3550.21O").unwrap();

        let reader = BufferedReader::<6, _>::plain(fp);
        let lines = reader.lines();

        let mut nth = 1;
        for line in lines {
            if let Ok(line) = line {
                if nth == 1 {
                    assert_eq!(line, "     2.11           OBSERVATION DATA    M (MIXED)           RINEX VERSION / TYPE");
                } else if nth == 2 {
                    assert_eq!(
                        line,
                        "HEADER CHANGED BY EPN CB ON 2021-12-28                      COMMENT"
                    );
                } else if nth == 3 {
                    assert_eq!(
                        line,
                        "TO BE CONFORM WITH THE INFORMATION IN                       COMMENT"
                    );
                } else if nth == 33 {
                    assert_eq!(
                        line,
                        "                                                            END OF HEADER"
                    );
                }
                nth += 1;
            }
        }
        assert_eq!(nth, 300);
    }

    #[test]
    fn crinex_reader() {
        let fp = File::open("../test_resources/CRNX/V1/AJAC3550.21D").unwrap();

        let reader = BufferedReader::<6, _>::crinex(fp);
        let lines = reader.lines();

        let mut nth = 1;
        for line in lines {
            match line {
                Ok(line) => {
                    println!("[{}] \"{}\"", nth, line);
                    if nth == 1 {
                        assert_eq!(line, "1.0                 COMPACT RINEX FORMAT                    CRINEX VERS   / TYPE");
                    } else if nth == 2 {
                        assert_eq!(line, "                    RNX2CRX ver.4.0.7                       28-Dec-21 00:17     CRINEX PROG / DATE");
                    } else if nth == 3 {
                        assert_eq!(line, "     2.11           OBSERVATION DATA    M (MIXED)           RINEX VERSION / TYPE");
                    } else if nth == 4 {
                        assert_eq!(
                            line,
                            "HEADER CHANGED BY EPN CB ON 2021-12-28                      COMMENT"
                        );
                    } else if nth == 5 {
                        assert_eq!(
                            line,
                            "TO BE CONFORM WITH THE INFORMATION IN                       COMMENT"
                        );
                    } else if nth == 35 {
                        assert_eq!(line, "                                                            END OF HEADER");
                    }
                },
                Err(e) => {
                    println!("error= {}", e);
                },
            }
            nth += 1;
        }
        assert!(nth > 35); // File Body left out !
        assert_eq!(nth, 303); // Missing some data (total) !
    }
}
