use log::{debug, error};
use std::io::Read;

#[cfg(feature = "flate2")]
use flate2::GzDecoder;

use crate::{constants::Constants, message::Message, utils::Utils, Error};

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum State {
    /// Read I/O: needs more byte
    #[default]
    Read,
    /// Parse internal buffer: consuming data
    Parsing,
}

/// [BINEX] Stream Decoder. Use this structure to decode all messages streamed
/// on a readable interface.
/// ```
/// use std::fs::File;
/// use binex::prelude::{Decoder, Error};
///
/// // Create the Decoder
/// //  * this one works from a local source
/// //  * decoder must be mutable
/// let mut fd = File::open("../test_resources/BIN/cres_20080526.bin")
///     .unwrap();
/// let mut decoder = Decoder::new(fd);
///
/// // Iterate the data stream
/// while let Some(ret) = decoder.next() {
///     match ret {
///         Ok(msg) => {
///             
///         },
///         Err(e) => match e {
///             Error::IoError(e) => {
///                 // any I/O error should be handled
///                 // and user should react accordingly,
///             },
///             Error::ReversedStream | Error::LittleEndianStream => {
///                 // this library is currently limited:
///                 //  - reversed streams are not supported yet
///                 //  - little endian streams are not supported yet
///             },
///             Error::InvalidStartofStream => {
///                 // other errors give meaningful information
///             },
///             _ => {},
///         },
///     }
/// }
/// ```
pub struct Decoder<R: Read> {
    /// [R]
    reader: R,
    /// Buffer read pointer
    rd_ptr: usize,
    /// Buffer write pointer
    wr_ptr: usize,
    /// Internal buffer
    buffer: Vec<u8>,
    /// Current Message being decoded to handle
    /// case when [Message] appears in the middle of two succesive .read()
    msg: Message,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Self {
            rd_ptr: 0,
            wr_ptr: 0,
            reader,
            msg: Default::default(),
            buffer: [0; 1024].to_vec(),
        }
    }
}

impl<R: Read> Iterator for Decoder<R> {
    type Item = Result<Message, Error>;
    /// Parse next message contained in stream
    fn next(&mut self) -> Option<Self::Item> {
        // parse internal buffer
        while self.rd_ptr < self.wr_ptr {
            println!("parsing: rd={}/wr={}", self.rd_ptr, self.wr_ptr);
            println!("workbuf: {:?}", &self.buffer[self.rd_ptr..]);

            match Message::decode(&self.buffer[self.rd_ptr..]) {
                Ok(msg) => {
                    self.rd_ptr += msg.encoding_size();
                    return Some(Ok(msg));
                },
                Err(Error::NoSyncByte) => {
                    // no SYNC in entire buffer
                    // => reset & re-read
                    println!(".decode no-sync");
                    self.rd_ptr = 0;
                    self.wr_ptr = 0;
                    self.buffer.clear();
                    break;
                },
                Err(e) => {
                    println!(".decode error: {}", e);
                    self.rd_ptr = 0;
                    self.wr_ptr = 0;
                    self.buffer.clear();
                    break;
                },
            }
        }

        // read data: fill in buffer
        match self.reader.read(&mut self.buffer) {
            Ok(size) => {
                if size == 0 {
                    None // EOS
                } else {
                    self.wr_ptr += size;
                    Some(Err(Error::NotEnoughBytes))
                }
            },
            Err(e) => {
                println!("i/o error: {}", e);
                Some(Err(Error::IoError(e)))
            },
        }
    }
}
