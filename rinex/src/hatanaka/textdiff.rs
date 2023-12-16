#[derive(Debug)]
pub struct TextDiff {
    pub buffer: String,
}

impl Default for TextDiff {
    fn default() -> Self {
        Self::new()
    }
}

impl TextDiff {
    /// Creates a new `Text` differentiator.
    /// Text compression has no limitations
    pub fn new() -> Self {
        Self {
            buffer: String::with_capacity(64),
        }
    }

    /// Initializes `Text` differentiator
    pub fn init(&mut self, data: &str) {
        self.buffer = data.to_string();
    }

    /// Decompresses given data
    pub fn decompress(&mut self, data: &str) -> &str {
        let s0_len = self.buffer.len();
        let s0 = unsafe { self.buffer.as_bytes_mut() };
        let s1_len = data.len();
        let s1 = data.as_bytes();
        let min = std::cmp::min(s1_len, s0_len);

        // browse shared content
        for index in 0..min {
            if s1[index] != b' ' {
                // not a differenced out character
                // ==> needs to overwrite internal content
                if s1[index] == b'&' {
                    // special whitespace insertion
                    // overwrite with space
                    s0[index] = b' ';
                } else {
                    // regular content
                    s0[index] = s1[index]; // overwrite
                }
            }
        }

        if s1_len > s0_len {
            // got new bytes to latch
            let new_slice = &data[min..s1_len].replace('&', " ");
            self.buffer.push_str(new_slice);
        }

        &self.buffer
    }

    /// Compresses given data
    pub fn compress(&mut self, data: &str) -> String {
        let mut result = String::new();
        let inner: Vec<_> = self.buffer.chars().collect();
        self.buffer.clear();
        let to_compress: Vec<_> = data.chars().collect();

        for i in 0..inner.len() {
            if let Some(c) = to_compress.get(i) {
                self.buffer.push_str(&c.to_string());
                if c != &inner[i] {
                    result.push_str(&c.to_string());
                } else {
                    result.push(' ');
                }
            }
        }

        for i in inner.len()..data.len() {
            if let Some(c) = to_compress.get(i) {
                if c.is_ascii_whitespace() {
                    self.buffer.push('&');
                    result.push('&');
                } else {
                    self.buffer.push_str(&c.to_string());
                    result.push_str(&c.to_string());
                }
            }
        }

        result.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_decompression() {
        let init = "ABCDEFG 12 000 33 XXACQmpLf";
        let mut diff = TextDiff::new();
        let masks: Vec<&str> = vec![
            //"ABCDEFG 12 000 33 XXACQmpLf"
            "         3   1 44 xxACq   F",
            "        4 ",
            " 11 22   x   0 4  y     p  ",
            "              1     ",
            "                   z",
            " ",
            "                           &",
        ];
        let expected: Vec<&str> = vec![
            "ABCDEFG 13 001 44 xxACqmpLF",
            "ABCDEFG 43 001 44 xxACqmpLF",
            "A11D22G 4x 000 44 yxACqmpLF",
            "A11D22G 4x 000144 yxACqmpLF",
            "A11D22G 4x 000144 yzACqmpLF",
            "A11D22G 4x 000144 yzACqmpLF",
            "A11D22G 4x 000144 yzACqmpLF ",
        ];
        diff.init(init);
        for i in 0..masks.len() {
            let mask = masks[i];
            let result = diff.decompress(mask);
            assert_eq!(result, String::from(expected[i]));
        }

        // test re-init
        let init = " 2200 123      G 07G08G09G   XX XX";
        diff.init(init);

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
            assert_eq!(result, String::from(expected[i]));
        }
    }
    #[test]
    fn test_compression() {
        let mut diff = TextDiff::new();

        diff.init("0");
        let compressed = diff.compress("0");
        assert_eq!(compressed, " ");
        let compressed = diff.compress("4");
        assert_eq!(compressed, "4");

        let compressed = diff.compress("4  ");
        assert_eq!(compressed, " &&");

        let compressed = diff.compress("0");
        assert_eq!(compressed, "0");

        // test re-init
        diff.init("Default Phrase 1234");
        let to_compress = "DEfault Phrase 1234";
        let result = diff.compress(to_compress);
        assert_eq!(result, " E                 ");

        let to_compress = "DEfault Phrase 1234";
        let result = diff.compress(to_compress);
        assert_eq!(result, "                   ");

        let to_compress = "DEFault Phrase 1234";
        let result = diff.compress(to_compress);
        assert_eq!(result, "  F                ");

        let to_compress = "DEFault Phrase 1234  ";
        let result = diff.compress(to_compress);
        assert_eq!(result, "                   &&");

        let to_compress = " EFault Phrase 1234  ";
        let result = diff.compress(to_compress);
        assert_eq!(result, "                     ");

        let to_compress = "__ abcd Phrase 1222    ";
        let result = diff.compress(to_compress);
        assert_eq!(result, "__  bcd          22  &&");

        diff.init(" ");
        assert_eq!(diff.compress("3"), "3");
        assert_eq!(diff.compress("3"), " ");
    }
}
