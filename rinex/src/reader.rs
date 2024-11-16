//! Buffered Reader wrapper, for efficient data reading and seamless decompression.
#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;

use crate::hatanaka::DecompressorExpert;

use std::io::{BufRead, BufReader, Error as IoError, Read};

const BUF_SIZE: usize = 8192;

/// [Reader] is dedicated to efficient RINEX input parsing.
/// Whether they are local files or not. It implements efficient
/// internal buffering that is specifically scaled for RINEX content.
/// It is solely here to provide [BufRead] implementation on [Read]able interfaces.
pub struct Reader<R: Read> {
    /// Internal buffer, which needs to store up to two complete lines.
    /// Since the objective here is to always provide a complete Line (without \n termination),
    /// and be consistent with std::Lines iterator.
    /// RINEX lines are short, and this will be more than enough.
    buf: [u8; BUF_SIZE],
    rd_ptr: usize,
    wr_ptr: usize,
    reader: R,
}

impl<R: Read> Reader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            wr_ptr: 0,
            rd_ptr: 0,
            buf: [0; BUF_SIZE],
        }
    }
}

impl<R: Read> Read for Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // always try to pull new content, if we can
        let avail = BUF_SIZE - self.wr_ptr;
        if avail > 0 {
            let size = self.reader.read(&mut self.buf[self.wr_ptr..])?;
            self.wr_ptr += size;
        }

        // try to provide new content if possible
        let buf_len = buf.len();

        let avail = BUF_SIZE - self.rd_ptr;
        if avail < buf_len {
            // we have less than buffer can accept:
            // return everything we have
            buf[..avail].copy_from_slice(&self.buf[self.rd_ptr..]);
            Ok(avail)
        } else {
            // we have more than buffer can accept
            // return only what it can accept, avoiding overflow
            buf.copy_from_slice(&self.buf[self.rd_ptr..self.rd_ptr + buf_len]);
            Ok(buf_len)
        }
    }
}

impl<R: Read> BufRead for Reader<R> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        let avail = BUF_SIZE - self.wr_ptr;
        if avail > 0 {
            let size = self.reader.read(&mut self.buf[self.wr_ptr..])?;
            self.wr_ptr += size;
        }
        Ok(&self.buf[self.rd_ptr..])
    }
    fn consume(&mut self, s: usize) {
        let avail = 8192 - self.rd_ptr;
        if avail >= s {
            self.rd_ptr += s;
        } else {
            self.rd_ptr = 8192;
        }
    }
}

#[cfg(test)]
mod test {
    use super::Reader;
    use std::fs::File;
    use std::io::BufRead;

    #[test]
    fn lines_iter() {
        let mut last_passed = false;

        let fd = File::open("../test_resources/OBS/V3/DUTH0630.22O").unwrap();
        let mut reader = Reader::new(fd);

        for (nth, line) in reader.lines().enumerate() {
            if let Ok(line) = line {
                if nth == 0 {
                    assert_eq!(line, "     2.11           OBSERVATION DATA    M (MIXED)           RINEX VERSION / TYPE");
                } else if nth == 1 {
                    assert_eq!(
                        line,
                        "HEADER CHANGED BY EPN CB ON 2021-12-28                      COMMENT"
                    );
                } else if nth == 34 {
                    assert_eq!(
                        line,
                        "                                                            END OF HEADER"
                    );
                } else if nth == 89 {
                    assert_eq!(line, "R24  20147683.700   107738728.87108     -2188.113          51.000    20147688.700    83796794.50808     -1701.871          48.500");
                } else {
                    if nth > 89 {
                        panic!("producing more than expected!");
                    }
                }
            }
        }
    }
}
