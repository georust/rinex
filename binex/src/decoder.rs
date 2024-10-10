use std::io::Read;

use log::{debug, error};

use crate::{constants::Constants, Error, Message};

#[derive(Debug, Copy, Clone, Default)]
pub enum State {
    /// Searching for SYNC byte marking start of stream
    #[default]
    Synchronizing = 0,
    /// SOF should follow SYNC byte
    SOF = 1,
}

/// [BINEX] Stream Decoder
pub struct Decoder<R: Read> {
    pub(crate) state: State,
    reader: R,
    ptr: usize,
    next_read: usize,
    buffer: Vec<u8>,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            ptr: 0,
            next_read: 128,
            state: State::default(),
            buffer: [0; 128].to_vec(),
        }
    }
}

impl<R: Read> Iterator for Decoder<R> {
    type Item = Result<Message, Error>;
    /// Parse next message in stream
    fn next(&mut self) -> Option<Self::Item> {
        // read some bytes
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

        let current = self.ptr;
        let size = self.buffer.len();
        let avail = size - current;
        let mut next_state = self.state;

        if avail < self.next_read {
            // not enough data compared to what we'd like
            // Re-iterate, maintain required data & current state
            return Some(Err(Error::NotEnoughBytes));
        }

        match self.state {
            State::Synchronizing => {
                // obtain SYNC byte from I/O interface
                for i in current..size {
                    match self.buffer[i] {
                        Constants::FWDSYNC_BE_STANDARD_CRC => {
                            debug!("forward big endian stream");
                            next_state = State::SOF;

                            // request more data if need be
                            if size < 4 {
                                self.next_read = 4;
                            }
                        },
                        Constants::FWDSYNC_LE_STANDARD_CRC
                        | Constants::FWDSYNC_LE_ENHANCED_CRC
                        | Constants::REVSYNC_LE_ENHANCED_CRC
                        | Constants::REVSYNC_BE_ENHANCED_CRC
                        | Constants::FWDSYNC_BE_ENHANCED_CRC
                        | Constants::FWDSYNC_LE_ENHANCED_CRC => {
                            // Little Endian + Enhanced CRC + Reversed
                            // not supported yet
                            error!("non supported format");
                        },
                        _ => {}, // invalid
                    }
                }
            },
            State::SOF => {
                // SOF should follow SYNC byte
                if avail < 4 {
                    // not enough data, SOF not available yet
                    self.next_read = 4;
                    return None;
                }
                let sof = u32::from_be_bytes([
                    self.buffer[self.ptr],
                    self.buffer[self.ptr + 1],
                    self.buffer[self.ptr + 2],
                    self.buffer[self.ptr + 3],
                ]);
            },
        }

        let ret = match (self.state, next_state) {
            (State::Synchronizing, State::Synchronizing) => {
                // still searching for SYNC byte
                None
            },
            (State::Synchronizing, State::SOF) => {
                // SYNC byte found
                None
            },
            (State::SOF, State::Synchronizing) => {
                // SOF byte not found
                None
            },
            (State::SOF, State::SOF) => {
                // should have located SOF right away (on read success)
                // abort: invalid stream content
                next_state = State::Synchronizing;
                Some(Err(Error::InvalidStartofStream))
            },
            // invalid combinations
        };

        // update state and return
        self.state = next_state;
        ret
    }
}
