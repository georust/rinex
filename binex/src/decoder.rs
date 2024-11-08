// use log::{debug, error};
use std::io::{Error as IoError, Read};

#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;

// use log::warn;

use crate::prelude::{ClosedSourceElement, Error, Message, StreamElement};

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

/// BINEX Stream Decoder. Use this structure to decode a serie
/// of [StreamElement]s streamed over any [Read]able interface.
pub struct Decoder<'a, R: Read> {
    /// Write pointer
    wr_ptr: usize,
    /// Read pointer
    rd_ptr: usize,
    /// Reached EOS
    eos: bool,
    /// Internal buffer. Buffer is sized to fully contain
    /// the "worst case" open source [Message].
    buf: [u8; 4096],
    /// [R]
    reader: Reader<R>,
    /// Reference to past [ClosedSourceElement] (if any)
    past_element: Option<ClosedSourceElement<'a>>,
}

impl<'a, R: Read> Decoder<'a, R> {
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
    /// let mut decoder = Decoder::new(fd);
    ///
    /// // Consume data stream
    /// loop {
    ///     match decoder.next() {
    ///         Some(Ok(msg)) => {
    ///             // do something
    ///         },
    ///         Some(Err(e)) => match e {
    ///             Error::IoError => {
    ///                 // any I/O error should be handled
    ///                 // and user should react accordingly,
    ///                 break;
    ///             },
    ///             Error::ReversedStream => {
    ///                 // this library is currently limited:
    ///                 //  - reversed streams are not supported yet
    ///                 //  - little endian streams are not supported yet
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
            buf: [0; 4096],
            past_element: None,
            reader: reader.into(),
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
    ///             Error::IoError => {
    ///                 // any I/O error should be handled
    ///                 // and user should react accordingly,
    ///                 break;
    ///             },
    ///             Error::ReversedStream => {
    ///                 // this library is currently limited:
    ///                 //  - reversed streams are not supported yet
    ///                 //  - little endian streams are not supported yet
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
            buf: [0; 4096],
            past_element: None,
            reader: GzDecoder::new(reader).into(),
        }
    }
}

impl<'a, R: Read> Iterator for Decoder<'a, R> {
    type Item = Result<StreamElement<'a>, Error>;

    /// Parse next [StreamElement] contained in this BINEX stream.
    fn next(&mut self) -> Option<Self::Item> {
        // always try to fill internal buffer
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

                // terminates possible [ClosedSourceElement] serie
                self.past_element = None;

                Some(Ok(msg.into()))
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
                    Error::NonSupportedMesssage(mlen) => {
                        self.rd_ptr += mlen;

                        if self.rd_ptr > 4096 {
                            self.rd_ptr = 0;
                            self.wr_ptr = 0;
                        }

                        if self.eos == true {
                            // consumed everything and EOS has been reached
                            return None;
                        }
                    },
                    Error::IncompleteMessage(mlen) => {
                        // decoded partial valid frame
                        if self.rd_ptr + mlen > 4096 {
                            // frame would not fit in buffer:
                            // abort: we do not support that scenario.
                            // This should never happen anyway: internal buffer should be sized correctly.
                            self.buf = [0; 4096];
                            self.wr_ptr = 0;
                            self.rd_ptr = 0;
                            return Some(Err(Error::TooLargeInternalLimitation));
                        } else {
                            // preserved content (shift left)
                            // and permit the refill that will conclude this message
                            self.buf.copy_within(self.rd_ptr.., 0);

                            self.wr_ptr -= self.rd_ptr;
                            self.rd_ptr = 0;
                            return Some(Err(Error::IncompleteMessage(mlen)));
                        }
                    },
                    Error::ClosedSourceMessage(closed_source) => {
                        // determine whether
                        // - this element is self sustained (ie., fully described by this meta)
                        // - the followup of previous elements
                        // - or the last element of a serie
                        if self.rd_ptr + closed_source.size < 4096 {
                            // content is fully wrapped in buffer: expose as is
                            // self.past_element = Some(ClosedSourceElement {
                            //     provider: meta.provider,
                            //     size: meta.mlen,
                            //     total: meta.mlen,
                            //     raw: self.buf[self.rd_ptr..self.rd_ptr +meta.mlen],
                            // });
                        } else {
                            // content is not fully wrapped up here;
                            // initiate or continue a serie of undisclosed element
                        }
                        return Some(Err(Error::IncompleteMessage(closed_source.size)));
                    },
                    Error::UnknownMessage => {
                        // panic!("unknown message\nrd_ptr={}\nbuf={:?}", self.rd_ptr, &self.buf[self.rd_ptr-1..self.rd_ptr+4]);
                        self.rd_ptr += 1;
                    },
                    _ => {
                        // bad content that does not look like valid BINEX.
                        // This is very inefficient. If returned error would increment
                        // the internal pointer, we could directly move on to next interesting bytes.
                        self.rd_ptr += 1;
                    },
                }
                Some(Err(e))
            },
        }
    }
}
