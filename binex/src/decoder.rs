// use log::{debug, error};
use std::io::{Error as IoError, Read};

#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;

use crate::{message::Message, Error};
use log::warn;

/// Abstraction for Plain or Compressed [R]
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
#[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
impl<R: Read> From<GzDecoder<R>> for Reader<R> {
    fn from(r: GzDecoder<R>) -> Reader<R> {
        Self::Compressed(r)
    }
}

impl<R: Read> Read for Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        match self {
            Self::Plain(r) => r.read(buf),
            #[cfg(feature = "flate2")]
            Self::Compressed(r) => r.read(buf),
        }
    }
}

/// Decoder FSM
#[derive(Debug, Copy, Clone, Default, PartialEq)]
enum State {
    /// Everything is OK we're consuming data
    #[default]
    Parsing,
    /// Partial frame is found in internal Buffer.
    /// We need a secondary read to complete this message
    IncompleteMessage,
    /// Partial frame was found in internal Buffer.
    /// But the total expected payload exceeds our internal buffer capacity.
    /// [Decoder] is currently limited to parsing [Message] that fits
    /// in the buffer entirely. This may not apply to very length (> 1 MB) messages
    /// which is the case of signal observations for example - that we do not support at the moment.
    /// In this case, we proceed to trash (consume the Input interface), complete the message
    /// we do not know how to interprate & move on to next message.
    IncompleteTrashing,
}

/// [BINEX] Stream Decoder. Use this structure to decode all messages streamed
/// on a readable interface.
pub struct Decoder<R: Read> {
    /// Internal state
    state: State,
    /// Read pointer
    rd_ptr: usize,
    /// Internal buffer
    buffer: Vec<u8>,
    /// [R]
    reader: Reader<R>,
    /// Used when partial frame is saved within Buffer
    size_to_complete: usize,
}

impl<R: Read> Decoder<R> {
    /// Creates a new BINEX [Decoder] from [R] readable interface,
    /// ready to parse incoming bytes.
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
    ///             // do something
    ///         },
    ///         Some(Err(e)) => match e {
    ///             Error::IoError(e) => {
    ///                 // any I/O error should be handled
    ///                 // and user should react accordingly,
    ///                 break;
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
    ///             // End of stream!
    ///             break;
    ///         },
    ///     }
    /// }
    /// ```
    pub fn new(reader: R) -> Self {
        Self {
            rd_ptr: 1024,
            size_to_complete: 0,
            reader: reader.into(),
            state: State::default(),
            buffer: [0; 1024].to_vec(),
        }
    }

    #[cfg(feature = "flate2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
    /// Creates a new Compressed BINEX stream [Decoder] from [R] readable
    /// interface, that must stream Gzip encoded bytes.
    /// ```
    /// use std::fs::File;
    /// use binex::prelude::{Decoder, Error};
    ///
    /// // Create the Decoder:
    /// //  * works from our local source
    /// //  * needs to be mutable due to iterating process
    /// let mut fd = File::open("../test_resources/BIN/mfle20200105.bnx.gz")
    ///     .unwrap();
    ///
    /// let mut decoder = Decoder::new(fd);
    ///
    /// // Consume data stream
    /// loop {
    ///     match decoder.next() {
    ///         Some(Ok(msg)) => {
    ///             // do something
    ///         },
    ///         Some(Err(e)) => match e {
    ///             Error::IoError(e) => {
    ///                 // any I/O error should be handled
    ///                 // and user should react accordingly,
    ///                 break;
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
    ///             // End of stream!
    ///             break;
    ///         },
    ///     }
    /// }
    /// ```
    pub fn new_gzip(reader: R) -> Self {
        Self {
            rd_ptr: 1024,
            size_to_complete: 0,
            state: State::default(),
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
        while self.rd_ptr < 1024 && self.state == State::Parsing {
            //println!("parsing: rd={}/wr={}", self.rd_ptr, 1024);
            //println!("workbuf: {:?}", &self.buffer[self.rd_ptr..]);

            match Message::decode(&self.buffer[self.rd_ptr..]) {
                Ok(msg) => {
                    // one message fully decoded
                    //   - increment pointer so we can move on to the next
                    //   - and expose to User.
                    self.rd_ptr += msg.encoding_size();
                    return Some(Ok(msg));
                },
                Err(Error::IncompleteMessage(mlen)) => {
                    //print!("INCOMPLETE: rd_ptr={}/mlen={}", self.rd_ptr, mlen);
                    // buffer contains partial message

                    // [IF] mlen (size to complete) fits in self.buffer
                    self.size_to_complete = mlen - self.rd_ptr;
                    if self.size_to_complete < 1024 {
                        // Then next .read() will complete this message
                        // and we will then be able to complete the parsing.
                        // Shift current content (rd_ptr=>0) and preserve then move on to Reading.
                        self.buffer.copy_within(self.rd_ptr..1024, 0);
                        self.state = State::IncompleteMessage;
                    } else {
                        // OR
                        // NB: some messages can be very lengthy (some MB)
                        // especially the signal sampling that we do not support yet.
                        // In this case, we simply trash the remaning amount of bytes,
                        // message is lost and we move on to the next SYNC
                        warn!("library limitation: unprocessed message");
                        self.state = State::IncompleteTrashing;
                        //println!("need to trash {} bytes", self.size_to_complete);
                    }
                },
                Err(Error::NoSyncByte) => {
                    // no SYNC in entire buffer
                    //println!(".decode no-sync");
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
                match self.state {
                    State::Parsing => {},
                    State::IncompleteMessage => {
                        // complete frame, move on to parsing
                        self.state = State::Parsing;
                    },
                    State::IncompleteTrashing => {
                        if self.size_to_complete == 0 {
                            // trashed completely.
                            self.state = State::Parsing;
                            //println!("back to parsing");
                        } else {
                            if self.size_to_complete < 1024 {
                                //println!("shiting {} bytes", self.size_to_complete);

                                // discard remaning bytes from buffer
                                // and move on to parsing to analyze new content
                                self.buffer.copy_within(self.size_to_complete.., 0);
                                self.state = State::Parsing;
                                //println!("back to parsing");
                            } else {
                                self.size_to_complete =
                                    self.size_to_complete.saturating_add_signed(-1024);
                                //println!("size to trash: {}", self.size_to_complete);
                            }
                        }
                    },
                }
                // read success
                self.rd_ptr = 0; // reset pointer & prepare for next Iter
                Some(Err(Error::NotEnoughBytes))
            },
            Err(_) => {
                None // EOS
            },
        }
    }
}
