use std::io::Read;

use log::{debug, error};

use crate::{constants::Constants, message::MessageID, utils::Utils, Error, Message};

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
    /// Minimal buffer len for [Self::State]
    next_read: usize,
    /// Internal buffer
    buffer: Vec<u8>,
    /// Endianness used when encoding current message,
    /// defined by SYNC byte
    big_endian: bool,
    /// Stored [MessageID]
    mid: MessageID,
    /// Stored Message Length
    mlen: usize,
    /// Internal [State]
    pub(crate) state: State,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Self {
            ptr: 0,
            reader,
            big_endian: true,
            next_read: 128,
            state: State::default(),
            buffer: [0; 128].to_vec(),
            mid: MessageID::default(),
            mlen: 0,
        }
    }
}

impl<R: Read> Iterator for Decoder<R> {
    type Item = Result<Message, Error>;
    /// Parse next message in stream
    fn next(&mut self) -> Option<Self::Item> {
        let size = self.buffer.len();
        let avail = size - self.ptr;
        let mut next_state = self.state;
        let mut keep_parsing = avail >= self.next_read; // more bytes than needed

        // process internal buffer
        while keep_parsing || self.ptr < size {
            // FSM
            match self.state {
                State::Synchronizing => {
                    // locate SYNC in buffer
                    for i in self.ptr..size {
                        match self.buffer[i] {
                            Constants::FWDSYNC_BE_STANDARD_CRC => {
                                debug!("SYNC: forward big endian stream");
                                self.ptr = i;
                                next_state = State::MID;
                                self.next_read = 4; // MessageID needs 4 bytes
                                self.big_endian = true;
                                break;
                            },
                            Constants::FWDSYNC_LE_ENHANCED_CRC => {
                                self.big_endian = false;
                                error!("little endianness: not supported yet");
                                error!("enhanced crc: not supported yet");
                            },
                            Constants::FWDSYNC_LE_STANDARD_CRC => {
                                self.big_endian = false;
                                error!("forward little endian stream: not supported yet");
                            },
                            Constants::FWDSYNC_BE_ENHANCED_CRC => {
                                self.big_endian = true;
                                error!("enhanced crc: not supported yet");
                            },
                            Constants::REVSYNC_LE_ENHANCED_CRC => {
                                self.big_endian = false;
                                error!("little endianness: not supported yet");
                                error!("reversed streams: not supported yet");
                                error!("enhanced crc: not supported yet");
                            },
                            Constants::REVSYNC_BE_ENHANCED_CRC => {
                                self.big_endian = true;
                                error!("enhanced crc: not supported yet");
                                error!("reversed streams: not supported yet");
                            },
                            _ => {}, // invalid
                        }
                    }

                    // SYNC byte still not found: increment pointer
                    if next_state != State::MID {
                        self.ptr += size;
                    }
                },
                State::MID => {
                    // MID must follow SYNC byte.
                    let mid = Utils::decode_bnxi(
                        &self.buffer[self.ptr..4], //TODO consume 4 bytes
                        self.big_endian,
                    ) as u8;
                    self.mid = MessageID::from(mid);
                    self.next_read = 4;
                    next_state = State::MLength;
                },
                State::MLength => {
                    let len = Utils::decode_bnxi(
                        &self.buffer[self.ptr..4], // TODO consume 4 bytes
                        self.big_endian,
                    );
                    self.mlen = len as usize;
                },
                State::Message => {},
            }

            // update to buffer size
            // keeps parsing as long as we still have enough bytes
            let avail = size - self.ptr;
            keep_parsing = avail >= self.next_read;
        } // keep_parsing

        let ret = match (self.state, next_state) {
            (State::Synchronizing, State::Synchronizing) => {
                // still searching for SYNC byte
                None
            },
            (State::Synchronizing, State::MID) => {
                // SYNC byte found
                None
            },
            (State::MID, State::Synchronizing) => {
                // MID byte not found
                None
            },
            (State::MID, State::MID) => {
                // should have located SOF right away (on read success)
                // abort: invalid stream content
                next_state = State::Synchronizing;
                Some(Err(Error::InvalidStartofStream))
            },
            // invalid combinations
            (State::MLength, State::Synchronizing)
            | (State::Synchronizing, State::MLength)
            | (State::Synchronizing, State::Message)
            | (State::MID, State::MLength)
            | (State::MID, State::Message)
            | (State::MLength, State::MLength)
            | (State::MLength, State::MID)
            | (State::MLength, State::Message)
            | (State::Message, State::Synchronizing)
            | (State::Message, State::MID)
            | (State::Message, State::Message)
            | (State::Message, State::MLength) => None,
        };

        // update state and return
        self.state = next_state;

        // read some bytes & append to internal buffer
        match self.reader.read(&mut self.buffer) {
            Ok(size) => {
                if size == 0 {
                    return None; // End of Stream
                }
            },
            Err(e) => {
                error!("i/o error: {}", e);
                return Some(Err(Error::IoError(e)));
            },
        }
        ret
    }
}
