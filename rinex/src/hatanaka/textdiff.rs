#[derive(Debug)]
pub struct TextDiff {
    pub init: String,
}

impl TextDiff {
    /// Creates a new `Text` differentiator.
    /// Text compression has no limitations
    pub fn new() -> Self {
        Self {
            init: String::new(),
        }
    }
    
    /// Initializes `Text` differentiator
    pub fn init (&mut self, data: &str) {
        self.init = data.to_string() ;
    }

    /// Decompresses given data
    pub fn decompress (&mut self, data: &str) -> String {
        let mut recovered = String::from("");
        let l = self.init.len();
        let mut p = self.init
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
        self.init = recovered.clone(); // for next time
        recovered 
    }
    /// Compresses given data
    pub fn compress (&mut self, data: &str) -> String {
        let mut result = String::new();
        let mut ptr_inner = self.init
            .chars();
        let ptr_data = data
            .chars();     
        for c in ptr_data {
            if c == '&' { // special whitespace insertion
                result.push_str(" ");
                let _ = ptr_inner.next(); // overwrite
            } else {
                if let Some(c_inner) = ptr_inner.next() {
                    if c == c_inner {
                        result.push_str(" ");
                    } else {
                        result.push(c);
                    }
                } else {
                    result.push(c);
                }
            }
        }
        self.init = data.replace("&", " ")
            .clone();
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_decompression() {
        let init = "ABCDEFG 12 000 33 XXACQmpLf";
        let mut diff = TextDiff::new();
        let masks : Vec<&str> = vec![
            "        13   1 44 xxACq   F",
            " 11 22   x   0 4  y     p  ",
            "              1     ",
            "                   z",
            " ",
        ];
        let expected : Vec<&str> = vec![
            "ABCDEFG 13 001 44 xxACqmpLF",
            "A11D22G 1x 000 44 yxACqmpLF",
            "A11D22G 1x 000144 yxACqmpLF",
            "A11D22G 1x 000144 yzACqmpLF",
            "A11D22G 1x 000144 yzACqmpLF",
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
        
        let masks : Vec<&str> = vec![
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
        assert_eq!(result, "                     ");
        
        let to_compress = "&EFault Phrase 1234  ";
        let result = diff.compress(to_compress);
        assert_eq!(result, "                     ");
        
        let to_compress = "__&abcd Phrase 1222    ";
        let result = diff.compress(to_compress);
        assert_eq!(result, "__  bcd          22    ");
    }
}
