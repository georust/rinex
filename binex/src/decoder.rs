use log::{debug, error};

use crate::{constants::Constants, message::Message, utils::Utils, Error};

/// BINEX Stream Decoder. Use this structure to decode all messages streamed
/// on a readable interface.
/// ```
/// use std::fs::File;
/// use binex::prelude::{Decoder, Error};
///
/// // Create the Decoder:
/// //  * works from our local source
/// //  * needs to be mutable due to iterating process
/// let mut fd = File::open("../test_resources/BIN/mfle20190130.bnx")
///     .unwrap();
///
/// let mut decoder = Decoder::new(fd);
///
/// // Consume data stream
/// loop {
///     match decoder.next() {
///         Some(Ok(msg)) => {
///             
///         },
///         Some(Err(e)) => match e {
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
///         None => {
///             // reacehed of stream!
///             break;
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
            debug!("parsing: rd={}/wr={}", self.rd_ptr, self.wr_ptr);
            debug!("workbuf: {:?}", &self.buffer[self.rd_ptr..]);

            match Message::decode(&self.buffer[self.rd_ptr..]) {
                Ok(msg) => {
                    self.rd_ptr += msg.encoding_size();
                    return Some(Ok(msg));
                },
                Err(Error::NoSyncByte) => {
                    // no SYNC in entire buffer
                    // => reset & re-read
                    error!(".decode no-sync");
                    self.rd_ptr = 0;
                    self.wr_ptr = 0;
                    self.buffer.clear();
                    break;
                },
                Err(e) => {
                    error!(".decode error: {}", e);
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
                error!("i/o error: {}", e);
                Some(Err(Error::IoError(e)))
            },
        }
    }
}
