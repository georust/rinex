use binex::prelude::{Decoder, Error};

struct Reader {
    ptr: usize,
    buf: Vec<u8>,
}

impl Reader {
    pub fn new(buf: &[u8]) -> Self {
        Self {
            ptr: 0,
            buf: buf.to_vec(),
        }
    }
    pub fn append(&mut self, buf: &[u8]) {
        for i in 0..buf.len() {
            self.buf.push(buf[i]);
        }
    }
    pub fn clear(&mut self) {
        self.ptr = 0;
        self.buf.clear();
    }
}

impl std::io::Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        //let size = buf.capacity();
        //let tocopy = self.buf.len() - self.ptr;
        //if target_capacity < tocopy {
        //    buf[..capacity].copy_from_slice(&self.buf);
        //    Ok(capacity)
        //} else {
        //    Ok(self.buf.len() - self.ptr)
        //}
        Ok(0)
    }
}

#[test]
fn parser_not_enough_bytes() {
    let buf = [0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5];
    let mut reader = Reader::new(&buf);
    let mut parser = Decoder::new(reader);

    match parser.next() {
        Some(Err(e)) => match e {
            Error::NotEnoughBytes => {},
            e => {
                panic!("found invalid error: {}", e);
            },
        },
        _ => {
            panic!("invalid state");
        },
    }
}

#[test]
fn parser_missing_sync() {
    let buf = [
        0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5,
        0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5,
        0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5,
        0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5,
        0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5,
        0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5,
        0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5,
        0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5,
        0, 1, 2, 3, 4, 5,
    ];
    let mut reader = Reader::new(&buf);
    let mut parser = Decoder::new(reader);

    match parser.next() {
        Some(Ok(msg)) => {},
        Some(Err(e)) => {
            panic!("found invalid error: {}", e);
        },
        None => {
            panic!("should have parsed valid message");
        },
    }
}
