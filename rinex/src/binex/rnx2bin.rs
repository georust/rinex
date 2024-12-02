//! RINEX to BINEX serialization
use std::io::Write;
use crate::prelude::Rinex;
use binex::prelude::{Encoder, Message};

/// RNX2BIN can serialize a [Rinex]
/// into a stream of BINEX [Message]s
pub struct RNX2BIN<&'a> {
    /// [Rinex] that was previously collected
    rinex: &'a Rinex,
    /// State to consume the header
    state: State,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum State {
    /// Describes the RINEX format, Constellation and revision
    #[default]
    VersionType, 
    /// Describes the RINEX original author, agency and operator
    ProgRunBy,
    /// Describes all comments originally written in header section
    HeaderComments,
    /// Done parsing Header: streaming record content
    Record,
}

impl<'a> Iterator for RNX2BIN<'a> {
    type Item = Option<Message>;
    /// Consume [Rinex] into [Message] stream
    fn next(&mut self) -> Option<Self::Item> {
        match self.header_state {
            State::VersionType => {

            },
            State::ProgRunBy => {

            },
            State::HeaderComments => {

            },
            State::Record => {

            },
        }
    }
}

impl<W: Write> RNX2BIN<W> {
    /// Creates a new [RNX2BIN].
    pub(crate) fn new(rinex: &'a Rinex) -> Self {
        Self {
            rinex,
            header_state: State::default(),
        }
    }
}
