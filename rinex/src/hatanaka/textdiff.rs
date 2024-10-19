//! Y. Hatanaka lossless TextDiff algorithm

// TODO
// [TextDiff] does not support Compress/Decompress back and forth,
// it currently requires to be built for one way or the other,
// and requires new Build to change direction.
// We should avoid that and permit back & forth operations.

#[derive(Debug)]
pub struct TextDiff {
    buffer: String,
    // avoids one alloc per compression call
    compressed: String,
}

impl Default for TextDiff {
    fn default() -> Self {
        Self::new("")
    }
}

impl TextDiff {
    /// Creates a new [TextDiff] structure to apply the
    /// differential algorithm developped by Y. Hatanaka.
    pub fn new(data: &str) -> Self {
        Self {
            buffer: data.to_string(),
            compressed: data.to_string(),
        }
    }

    /// Force kernel reset using new content
    pub fn force_init(&mut self, data: &str) {
        self.buffer = data.to_string();
        self.compressed = data.to_string();
    }

    /// Decompresses given data. Returns recovered value.
    pub fn decompress(&mut self, data: &str) -> &str {
        let size = data.len();
        let buf_len = self.buffer.len();

        if size > buf_len {
            // extend new bytes as is
            self.buffer.push_str(&data[buf_len..]);
        }

        // browse all bytes and save only new content
        let bytes = data.bytes();
        unsafe {
            let buf = self.buffer.as_bytes_mut();

            // browse all new bytes
            for (i, byte) in bytes.enumerate() {
                if let Some(buf) = buf.get_mut(i) {
                    if byte != b' ' {
                        if byte == b'&' {
                            *buf = b' ';
                        } else {
                            *buf = byte;
                        }
                    }
                }
            }
            &self.buffer
        }
    }

    /// Compress given data using the Textdiff algorithm.
    /// Returns compressed text.
    pub fn compress(&mut self, data: &str) -> &str {
        let len = data.len();
        let buf_len = self.buffer.len();
        let history_len = self.buffer.len();

        // expand with new content
        if len > buf_len {
            self.buffer.push_str(&data[buf_len..]);
            self.compressed.push_str(&data[buf_len..].replace(' ', "&")); // copy & replace whitespaces
        }

        // study possible shared content
        let mut new = data.bytes();
        let mut history = self.buffer[..history_len].bytes();

        unsafe {
            let mut compressed = self.compressed.as_bytes_mut().iter_mut();

            // any new byte gets compressed or kept
            while let Some(new) = new.next() {
                let compressed = compressed.next().unwrap();
                if let Some(history) = history.next() {
                    if history == new {
                        *compressed = b' ';
                    } else {
                        *compressed = new;
                    }
                }
            }

            // supports shorter inputs than internal buffer:
            // possible '&' residues need to be whitened
            // this gives maximum flexibility when dealing with actual files
            while let Some(compressed) = compressed.next() {
                *compressed = b' ';
            }

            self.buffer = data.to_string();
            &self.compressed
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_decompression() {
        let mut diff = TextDiff::new("ABCDEFG 12 000 33 XXACQmpLf");

        let compressed: Vec<&str> = vec![
            "         3   1 44 xxACq   F",
            "        4 ",
            " 11 22   x   0 4  y     p  ",
            "              1     ",
            "                   z",
            " ",
            "                           &",
            "&                           ",
            " ",
        ];
        let expected: Vec<&str> = vec![
            "ABCDEFG 13 001 44 xxACqmpLF",
            "ABCDEFG 43 001 44 xxACqmpLF",
            "A11D22G 4x 000 44 yxACqmpLF",
            "A11D22G 4x 000144 yxACqmpLF",
            "A11D22G 4x 000144 yzACqmpLF",
            "A11D22G 4x 000144 yzACqmpLF",
            "A11D22G 4x 000144 yzACqmpLF ",
            " 11D22G 4x 000144 yzACqmpLF ",
            " 11D22G 4x 000144 yzACqmpLF ",
        ];

        for i in 0..compressed.len() {
            let decompressed = diff.decompress(compressed[i]);
            assert_eq!(
                decompressed,
                expected[i],
                "failed for {}th \"{}\"",
                i + 1,
                compressed[i]
            );
        }

        // test re-init
        diff.force_init(" 2200 123      G 07G08G09G   XX XX");

        let masks: Vec<&str> = vec![
            "        F       1  3",
            " x    1 f  f   p",
            " ",
            "  3       4       ",
        ];
        let expected: Vec<&str> = vec![
            " 2200 12F      G107308G09G   XX XX",
            " x200 12f  f   p107308G09G   XX XX",
            " x200 12f  f   p107308G09G   XX XX",
            " x300 12f 4f   p107308G09G   XX XX",
        ];
        for i in 0..masks.len() {
            let mask = masks[i];
            let result = diff.decompress(mask);
            assert_eq!(
                result,
                expected[i].to_string(),
                "failed for [{}]{}",
                i,
                mask
            );
        }
    }

    #[test]
    fn test_compression() {
        let mut diff = TextDiff::new("0");
        assert_eq!(diff.compress("0"), " ");
        assert_eq!(diff.compress("4"), "4");
        assert_eq!(diff.compress("4"), " ");
        assert_eq!(diff.compress("4 "), " &");
        assert_eq!(diff.compress("4  "), "  &");
        assert_eq!(diff.compress("0"), "0  ");

        diff.force_init("Default 1234");
        assert_eq!(diff.compress("DEfault 1234"), " E          ");
        assert_eq!(diff.compress("DEfault 1234"), "            ");
        assert_eq!(diff.compress("             "), "            &");
    }
}
