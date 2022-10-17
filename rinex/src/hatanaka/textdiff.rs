#[derive(Debug)]
pub struct TextDiff {
    pub buffer: String,
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
    pub fn init (&mut self, data: &str) {
        self.buffer = data.to_string() ;
    }

    // Returns positions of bytes that differ, in s1 as compared
    // to s0
    fn diff (s0: &str, s1: &str) -> Vec<usize> {
        let s0_len = s0.len();
        let mut s0 = s0.chars();
        s1.chars()
            .into_iter()
            .enumerate()
            .filter_map(|(index, s1)| {
                if s1 == ' ' {
                    None
                } else {
                    if let Some(c) = s0.next() {
                        if s1 != c {
                            Some(index)
                        } else {
                            None
                        }
                    } else {
                        //new bytes => obvious difference
                        Some(index)
                    }
                }
            })
            .collect()
    }

    /// Decompresses given data
    pub fn decompress (&mut self, data: &str) -> String {
        println!("INTERNAL      \"{}\"", self.buffer);
        println!("DECOMPRESSING \"{}\"", data);
        let diffs = Self::diff(&self.buffer, data);

        for pos in diffs {
            let slice = &data[pos..pos+1];
            println!("{}..{}   | \"{}\"", pos, pos+1, slice); 
            if pos < self.buffer.len() {
                self.buffer
                    .replace_range(pos..pos+1, slice);
            } else { // new bytes
                self.buffer
                    .push_str(slice);
            }
        }
        
        // manage special whitespace insertions
        while let Some(pos) = self.buffer.as_str().find("&") {
            self.buffer
                .replace_range(pos..pos+1, " ");
        }

        //previous logic always ommits last byte in data
        /*let last_byte_pos = data.len()-1;
        let last_byte =  &data[last_byte_pos-1..last_byte_pos];
        if last_byte != " " {
            // ==> overwrite last byte
            self.buffer
                .replace_range(last_byte_pos-1..last_byte_pos, last_byte);
        }*/

        // when decompressing, we expose internal content as is
/*
        let max = std::cmp::max(internal.len(), new.len());
        for i in 0..max {
            if new[i] != ' ' {
                // not a whitespace
                //  means overwrite internal content
                if new[i] == '&' {
                    // \& means whitespace insertion
                    internal[i] = '&';
                } else {
                    internal[i] = new[i];
                }
            }
        }
*/
/*
        let l = self.buffer.len();
        let mut p = self.buffer
            .as_mut_str()
            .chars();
        let mut data = data.chars();
        for _ in 0..l {
            let next_c = p.next().unwrap();
            if let Some(c) = data.next() {
                if c == '&' { // special whitespace insertion
                    recovered.push_str(" ")
                } else if c.is_ascii_alphanumeric() {
                    recovered.push_str(&c.to_string())
                } else {
                    recovered.push_str(&next_c.to_string())
                }
            } else {
                recovered.push_str(&next_c.to_string())
            }
        }
        // mask might be longer than self
        // in case we need to extend current value
        loop {
            if let Some(c) = data.next() {
                if c == '&' { // special whitespace insertion
                    recovered.push_str(" ")
                } else if c.is_ascii_alphanumeric() {
                    recovered.push_str(&c.to_string())
                }
            } else {
                break
            }
        }
        self.buffer = recovered.clone(); // for next time
        recovered */
        self.buffer.clone()
    }
    
    /// Compresses given data
    pub fn compress (&mut self, data: &str) -> String {
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
                    result.push_str(" ");
                }
            }
        }

        for i in inner.len()..data.len() {
            if let Some(c) = to_compress.get(i) {
                if c.is_ascii_whitespace() {
                    self.buffer.push_str("&");
                    result.push_str("&");
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
        let expected : Vec<&str> = vec![
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
        let expected : Vec<&str> = vec![
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
        diff.init(        "Default Phrase 1234");
        let to_compress = "DEfault Phrase 1234";
        let result = diff.compress(to_compress);
        assert_eq!(result," E                 ");
        
        let to_compress = "DEfault Phrase 1234";
        let result = diff.compress(to_compress);
        assert_eq!(result,"                   ");
        
        let to_compress = "DEFault Phrase 1234";
        let result = diff.compress(to_compress);
        assert_eq!(result,"  F                ");
        
        let to_compress = "DEFault Phrase 1234  ";
        let result = diff.compress(to_compress);
        assert_eq!(result, "                   &&");
        
        let to_compress = " EFault Phrase 1234  ";
        let result = diff.compress(to_compress);
        assert_eq!(result, "                     ");
        
        let to_compress = "__ abcd Phrase 1222    ";
        let result = diff.compress(to_compress);
        assert_eq!(result,"__  bcd          22  &&");

        diff.init(" ");
        assert_eq!(diff.compress("3"), "3");
        assert_eq!(diff.compress("3"), " ");
    }
}
