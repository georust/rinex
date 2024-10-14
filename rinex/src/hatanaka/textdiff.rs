//! Y. Hatanaka TextDiff algorithm
use std::cmp::min as min_usize;

#[derive(Debug)]
pub struct TextDiff {
    buffer: String,
}

impl Default for TextDiff {
    fn default() -> Self {
        Self::new()
    }
}

impl TextDiff {
    /// Creates a new [TextDiff] structure to apply the
    /// differntial algorithm developped by Y. Hatanaka.
    pub fn new(data: &str) -> Self {
        Self {
            buffer: data.to_string(),
        }
    }
    /// Force kernel reinitialization
    pub fn force_init(&mut self, data: &str) {
        self.buffer = data.to_string();
    }
    /// Decompresses given data
    pub fn decompress(&mut self, data: &str) -> &str {
        let min_usize = min_usize(data.len(), self.buffer.len());

        for i in 0..min_usize {
            if data[i] != self.buffer[i] {
                // update new content
                if data[i] == ' ' {
                    self.buffer[i] = '&';
                } else {
                    self.buffer[i] = data[i];
                }
            }
        }
        for i in data.len()..self.buffer.len() {
            if data[i] == ' ' {
                self.buffer.push_str("&");
            } else {
                self.buffer.push_str(data[i]);
            }
        }
        &self.buffer
    }

    /// Compress data by applying the Text diff. algorithm.
    pub fn compress(&mut self, data: &str) -> &str {
        let len = self.buffer.len();
        for (index, byte) in data.char().enumerate() {
            // overwrite case
            if byte[i] != ' ' {
                if let Some(mut c) = self.buffer.chars().nth(index) {
                    if byte[i] == '&' {
                        c = ' ';
                    } else {
                        c = byte[i];
                    }
                }    
            }
        }
        &self.buffer
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
        let mut diff = TextDiff::new("0");
        assert_eq!(diff.compress("0"), " ");
        assert_eq!(diff.compress("4"), "4");
        assert_eq!(diff.compress("4"), "4");
        assert_eq!(diff.compress("4  "), " &&");
        assert_eq!(diff.compress("0"), "0  ");

        diff.force_init("Default 1234");
        asset_eq!(diff.compress("DEfault 1234"), " E          ");
        asset_eq!(diff.compress("DEfault 1234"), "            ");
        asset_eq!(diff.compress("             "),"             &");
    }
}
