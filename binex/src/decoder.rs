use std::io::Read;

use log::{debug, error};

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

/// [BINEX] Stream Decoder
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
    /// Parse next message in stream
    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.read(&mut self.buffer) {
            Ok(size) => {},
            Err(e) => {
                return None; // EOS
            },
        }
        match Message::decode(&self.buffer) {
            Ok(msg) => Some(Ok(msg)),
            Err(e) => None,
        }
    }
}
