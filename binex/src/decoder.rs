use std::io::Read;
use log::{debug, error};

#[cfg(feature = "flate2")]
use flate2::GzDecoder;

use crate::{constants::Constants, message::Message, utils::Utils, Error};

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum State {
    /// Searching for SYNC byte, defining a start of stream
    #[default]
    Synchronizing = 0,
    /// Message ID byte follows SYNC byte and is BNXI encoded
    MID = 1,
    /// Message length follows MID bytes and is BNXI encoded
    MLength = 2,
    /// Message content follows MLength bytes
    Message = 3,
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
    ptr: usize,
    /// Internal buffer
    buffer: Vec<u8>,
    /// Current Message being decoded
    msg: Message,
    /// Internal [State]
    state: State,
    /// Minimal number of bytes for current [State]
    next_read: usize,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Self {
            ptr: 0,
            reader,
            next_read: 128,
            state: State::default(),
            msg: Default::default(),
            buffer: [0; 128].to_vec(),
        }
    }
}

impl<R: Read> Iterator for Decoder<R> {
    type Item = Result<Message, Error>;
    /// Parse next message contained in stream
    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.read(&mut self.buffer[self.ptr..]) {
            Ok(size) => {
                if size == 0 {
                    return None; // marks the EOS
                } else {
                    self.ptr += size;
                }
            },
            Err(e) => {
                return Some(Err(Error::IoError(e)));
            },
        }

        match Message::decode(&self.buffer) {
            Ok(msg) => Some(Ok(msg)),
            Err(e) => {
                println!("decoding error: {}", e);
                self.buffer.clear();
                self.ptr = 0;
                None
            },
        }
    }
}
