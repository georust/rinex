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
/// on a [Read]able interface. M represents the internal buffer depth:
/// * the larger M the less constrain on the I/O interface (less frequent access)
/// * but the larger the (initial) memory allocation
pub struct Decoder<const M: usize, R: Read> {
    /// Internal state
    state: State,
    /// Write pointer
    wr_ptr: usize,
    /// Read pointer
    rd_ptr: usize,
    /// Reached EOS
    eos: bool,
    /// Internal buffer
    buf: [u8; M],
    /// [R]
    reader: Reader<R>,
}

impl<const M: usize, R: Read> Decoder<M, R> {
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
    /// // Two generics: with M the internal buffer depth
    /// let mut decoder = Decoder::<1024, File>::new(fd);
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
            eos: false,
            rd_ptr: 0,
            wr_ptr: 0,
            buf: [0; M],
            reader: reader.into(),
            state: State::default(),
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
    /// // two generics: with M the internal buffer depth
    /// let mut decoder = Decoder::<1024, File>::new(fd);
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
            eos: false,
            rd_ptr: 0,
            wr_ptr: 0,
            buf: [0; M],
            state: State::default(),
            reader: GzDecoder::new(reader).into(),
        }
    }
}

impl<const M: usize, R: Read> Iterator for Decoder<M, R> {
    type Item = Result<Message, Error>;
    /// Parse next message contained in stream
    fn next(&mut self) -> Option<Self::Item> {
        // always try to fill in buffer
        let size = self.reader.read(&mut self.buf[self.wr_ptr..]).ok()?;
        self.wr_ptr += size;
        //println!("wr_ptr={}", self.wr_ptr);

        if size == 0 {
            self.eos = true;
        }

        // try to consume one message
        match Message::decode(&self.buf[self.rd_ptr..]) {
            Ok(msg) => {
                // one message fully decoded
                //  - increment pointer
                //  - expose to user
                self.rd_ptr += msg.encoding_size();
                Some(Ok(msg))
            },
            Err(e) => {
                match e {
                    Error::NoSyncByte => {
                        // buffer does not even contain the sync byte:
                        // we can safely discard everything
                        self.wr_ptr = 0;
                        self.rd_ptr = 0;
                        if self.eos == true {
                            // consumed everything and EOS has been reached
                            return None;
                        }
                    },
                    Error::IncompleteMessage(mlen) => {
                        // buffer does not contain the entire message
                        // preserve content and shift: to permit refilling the buffer
                        // two cases:
                        if mlen + 2 < M {
                            // - if that message would fit in buffer, shift and prepare to refill for completion
                            self.wr_ptr -= self.rd_ptr;
                            self.buf.copy_within(self.rd_ptr.., 0);
                            return Some(Err(Error::IncompleteMessage(mlen)));
                        } else {
                            // - or, we don't support messages that do not fit in the local buffer (yet)
                            self.buf = [0; M];
                            self.wr_ptr = 0;
                            self.rd_ptr = 0;
                            return Some(Err(Error::NonSupportedMesssage));
                        }
                    },
                    _ => {
                        self.rd_ptr += 1;
                    },
                }
                Some(Err(e))
            },
        }
    }
}
