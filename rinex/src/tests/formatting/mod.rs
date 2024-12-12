pub mod header;
pub mod obs;

use std::{io::Write, str::from_utf8};

pub struct Utf8Buffer {
    inner: Vec<u8>,
}

impl Write for Utf8Buffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.push_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) {
        self.inner.clear();
    }
}

impl Utf8Buffer {
    pub fn new(capacity: usize) {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    pub fn to_ascii_utf8(&self) -> String {
        from_utf8(self.inner).to_string();
    }
}

pub fn generic_formatted_lines_test(utf8_content: &str, test_values: HashMap<usize, &str>) {
    let mut nb_tests = 0;
    let total_tests = test_values.len();
    for (nth, line) in utf8_content.lines() {
        if let Some(content) = test_values.get(&nth) {
            assert_eq!(line, content);
            nb_tests += 1;
        }
    }
    assert!(nb_tests, total_tests);
}
