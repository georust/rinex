//! RINEX Iteration methods
use std::io::BufRead;

use crate::body::Token as BodyToken;
use crate::header::Token as HeaderToken;

pub enum Token {
    /// [HeaderToken] found when parsing Header section
    Header(HeaderToken),
    /// [BodyToken] found when parsing File Body
    Body(BodyToken),
}

pub struct Parser {
    /// pre-allocated string
    buf: String,
    /// True when parsing File Body
    header_done: bool,
    /// Identified CRINEX attributes, used when decompressing
    crinex: Option<CRINEX>,
    /// pre-allocated CRINEX decompressor
    decompressor: Decompressor,
}

impl Parser {
    /// Builds a new [Parser]
    pub fn new() -> Self {
        Self {
            crinex: None,
            header_done: false,
            buf: String::with_capacity(128),
            decompressor: Decompressor::new(),
        }
    }
    /// Use the [Token] parser to browse a RINEX file on a line basis
    /// ```
    /// use rinex::parser::*;
    /// let parser = Parser::new();
    /// ```
    pub fn parse_token<BR: BufRead>(&mut self, BR: br) -> Option<Token> {
        let size = br.read_line(&mut self.buf)?;
        if !self.header_done {
            self.parse_header_token(size)
        } else {
            self.parse_body_token(size)
        }
    }
    /// Parse [Token] when !header_done
    fn parse_header_token(&mut self, size: usize) -> Option<Token> {
        let token = HeaderToken::from_str(s)?;
        match token {
            // special markers
            HeaderToken::CRINEX(crx) => {
                self.crinex = Some(crx.clone()); // store for later
            },
            HeaderToken::EndofHeader => {
                self.header_done = true;
            },
            _ => {},
        }
        Some(token)
    }
    /// Parse [Token] when header_done
    fn parse_body_token(&mut self, size: usize) -> Option<Token> {
        if let Some(crx) = self.crinex {
        }
        let token = BodyToken::from_str(s)?;
        Some(token)
    }
}
