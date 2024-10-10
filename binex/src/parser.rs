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

/// [BINEX] Stream Parser
pub struct Parser<R: Read> {
    pub(crate) state: State,
    reader: R,
    ptr: usize,
    next_read: usize,
    buffer: Vec<u8>,
}

impl<R: Read> Parser<R> {
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

impl<R: Read> Iterator for Parser<R> {
    type Item = Result<Message, Error>;
    /// Parse next message in stream
    fn next(&mut self) -> Option<Self::Item> {
        // read some bytes
        match self.reader.read(&mut self.buffer) {
            Ok(size) => {
                if size < self.next_read {
                    // not enough bytes compared to what we currently need
                    // reiterate, preserve desired size
                    return Some(Err(Error::NotEnoughBytes));
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

        match self.state {
            State::Synchronizing => {
                // obtain SYNC byte from I/O interface
                if avail < 1 {
                    // not enough data
                    return None;
                }

                for i in current..size {
                    if self.buffer[i] == Constants::BNXSYNC2 {
                        debug!("sync byte found");
                        next_state = State::SOF;
                        // ahead: prepare to read more data if need be
                        if size < 4 {
                            self.next_read = 4;
                        }
                        break;
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
