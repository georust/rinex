// use log::{debug, error};
use std::io::{Error as IoError, Read};

#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;

use crate::{message::Message, Error};

enum Reader<R: Read> {
    Plain(R),
    #[cfg(feature = "flate2")]
    Compressed(GzDecoder<R>),
}

impl<R: Read> From<R> for Reader<R> {
    fn from(r: R) -> Reader<R> {
        Self::Plain(r)
    }
}

#[cfg(feature = "flate2")]
impl<R: Read> From<GzDecoder<R>> for Reader<R> {
    fn from(r: GzDecoder<R>) -> Reader<R> {
        Self::Compressed(r)
    }
}

impl<R: Read> Read for Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        match self {
            Self::Plain(r) => r.read(buf),
            Self::Compressed(r) => r.read(buf),
        }
    }
}

/// [BINEX] Stream Decoder. Use this structure to decode all messages streamed
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
    reader: Reader<R>,
    /// Buffer read pointer
    rd_ptr: usize,
    /// Internal buffer
    buffer: Vec<u8>,
}

impl<R: Read> Decoder<R> {
    /// Creates a new BINEX [Decoder] from [R] readable interface,
    /// ready to parse incoming bytes.
    pub fn new(reader: R) -> Self {
        Self {
            rd_ptr: 1024,
            reader: reader.into(),
            buffer: [0; 1024].to_vec(),
        }
    }

    #[cfg(feature = "flate2")]
    /// Creates a new Compressed BINEX stream [Decoder] from [R] readable
    /// interface, that must stream Gzip encoded bytes.
    pub fn new_gzip(reader: R) -> Self {
        Self {
            rd_ptr: 1024,
            buffer: [0; 1024].to_vec(),
            reader: GzDecoder::new(reader).into(),
        }
    }
}

impl<R: Read> Iterator for Decoder<R> {
    type Item = Result<Message, Error>;
    /// Parse next message contained in stream
    fn next(&mut self) -> Option<Self::Item> {
        // parse internal buffer
        while self.rd_ptr < 1024 {
            println!("parsing: rd={}/wr={}", self.rd_ptr, 1024);
            //println!("workbuf: {:?}", &self.buffer[self.rd_ptr..]);

            match Message::decode(&self.buffer[self.rd_ptr..]) {
                Ok(msg) => {
                    self.rd_ptr += msg.encoding_size();
                    return Some(Ok(msg));
                },
                Err(Error::NoSyncByte) => {
                    // no SYNC in entire buffer
                    println!(".decode no-sync");
                    // prepare for next read
                    self.rd_ptr = 1024;
                    //self.buffer.clear();
                    self.buffer = [0; 1024].to_vec();
                },
                Err(_) => {
                    // decoding error: unsupported message
                    // Keep iterating the buffer
                    self.rd_ptr += 1;
                },
            }
        }

        // read data: fill in buffer
        match self.reader.read_exact(&mut self.buffer) {
            Ok(_) => {
                println!("OK GOT SOMETHING: {:?}", self.buffer);
                self.rd_ptr = 0;
                // Exit and prepare for next Iter
                Some(Err(Error::NotEnoughBytes))
            },
            Err(e) => {
                //println!("i/o error: {}", e);
                // Some(Err(Error::IoError(e)))
                None // EOS
            },
        }
    }
}
