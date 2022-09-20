use thiserror::Error;
use std::collections::VecDeque;

#[derive(Error, Debug)]
pub enum Error {
    #[error("order cannot be greater than {0}")]
    OrderTooBig(usize),
    #[error("cannot recover a different type than init type!")]
    TypeMismatch,
}

/// `Dtype` describes the type of data
/// we can compress / decompress
#[derive(Clone, Debug)]
pub enum Dtype {
    /// Text represents epoch descriptors, 
    /// and observation flags
    Text(String),
    /// Numerical is intended to be used
    /// for observation data and clock offsets
    Numerical(i64),
}

impl Default for Dtype {
    /// Generates a default numerical = 0 `Dtype`
    fn default() -> Dtype { Dtype::Numerical(0) }
 }

impl Dtype {
    /// Unwraps `Dtype` as numerical data
    pub fn as_numerical (&self) -> Option<i64> {
        match self {
            Dtype::Numerical(n) => Some(n.clone()),
            _ => None,
        }
    }
    /// Unwraps `Dtype` as text data
    pub fn as_text (&self) -> Option<String> {
        match self {
            Dtype::Text(s) => Some(s.to_string()),
            _ => None,
        }
    }
}

/// `Kernel` is a structure to compress    
/// or recover data using recursive defferential     
/// equations as defined by Y. Hatanaka.   
/// No compression limitations but maximal order   
/// to be supported must be defined on structure     
/// creation for memory allocation efficiency.
#[derive(Debug, Clone)]
pub struct Kernel {
    m: usize,
    order: usize,
    init: Dtype,
    history: VecDeque<i64>,
}

impl Kernel {
    /// Builds a new kernel structure.    
    /// max: maximal Hatanaka order for this kernel to ever support.
    /// We only support max <= 7.
    /// For information, m = 5 is hardcoded in `CRN2RNX` and is a good compromise
    pub fn new (max: usize) -> Kernel {
        let mut null = Vec::with_capacity(max);
        null.fill_with(Default::default);
        Kernel {
            m: 0,
            order: max,
            init: Dtype::default(),
            history: null.clone(), 
        }
    }

    /// Initializes or reinitializes Self.
    /// Order: compression order   
    /// Data: kernel initializer
    pub fn init (&mut self, order: usize, data: Dtype) -> Result<(), Error> { 
        if order > self.vector.len() {
            return Err(Error::OrderTooBig(self.vector.len()))
        }
        // reset
        self.m = 0;
        if let Some(data) = data.as_numerical() {
            self.history.rotate_right(1);
            self.history[0] = data;
        }
        // init
        self.order = order;
        self.init = data.clone();
        Ok(())
    }

    /// Recovers data by computing recursive differential equation
    pub fn recover (&mut self, data: Dtype) -> Result<Dtype, Error> {
        match data {
            Dtype::Numerical(data) => {
                if let Some(_) = self.init.as_numerical() {
                    Ok(self.numerical_data_recovery(data))
                } else {
                    Err(Error::TypeMismatch)
                }
            },
            Dtype::Text(data) => {
                if let Some(_) = self.init.as_text() {
                   Ok(self.text_data_recovery(data))
                } else {
                    Err(Error::TypeMismatch)
                }
            },
        }
    }
    
    /// Compresses data with differential equation
    pub fn compress (&mut self, data: Dtype) -> Result<Dtype, Error> {
        match data {
            Dtype::Numerical(data) => {
                if let Some(_) = self.init.as_numerical() {
                    Ok(self.numerical_data_compression(data))
                } else {
                    Err(Error::TypeMismatch)
                }
            },
            Dtype::Text(data) => {
                if let Some(_) = self.init.as_text() {
                    Ok(Dtype::Text(self.text_data_compression(data)))
                } else {
                    Err(Error::TypeMismatch)
                }
            },
        }
    }

    fn rotate_history (&mut self, data: i64) {
        let _ = self.history.pop_front();
        self.history.rotate_right(1);
        self.history[0] = data;
    }

    /// Computes Differential Equation as defined in Hatanaka compression method
    fn numerical_data_recovery (&mut self, data: i64) -> Dtype {
        self.m += 1;
        self.m = std::cmp::min(self.m, self.order); // restraint
        self.rotate_history(data);
        let uncompressed :i64 = match self.m {
            0 => {
                self.history[0] - self.history[1]
            }
            1 => {
                self.history[0] - self.history[1]
                    - (self.history
            },
            2 => {
                self.history[0] - self.history[1];
            },
            3 => {
                self.history[0] - self.history[1];
            },
            4 => {
                self.history[0] - self.history[1];
            },
            5 => {
                self.history[0] - self.history[1];
            },
            6 => {
                self.history[0] - self.history[1];
            },
            7 => {
                self.history[0] - self.history[1];
            },
            _ => unreachable!(),
        };
        Dtype::Numerical(uncompressed)
    }
    
    /// Compresses numerical data using Hatanaka method
    fn numerical_data_compression (&mut self, data: i64) -> Dtype {
        self.m += 1;
        self.m = std::cmp::min(self.m, self.order); // restraint 
        self.rotate_history(data);

        let compressed :i64 = match self.m {
            0 => {
                self.history[0] - self.history[1]
            }
            1 => {
                self.history[0] - self.history[1]
                    - (self.history
            },
            2 => {
                self.history[0] - self.history[1];
            },
            3 => {
                self.history[0] - self.history[1];
            },
            4 => {
                self.history[0] - self.history[1];
            },
            5 => {
                self.history[0] - self.history[1];
            },
            6 => {
                self.history[0] - self.history[1];
            },
            7 => {
                self.history[0] - self.history[1];
            },
            _ => unreachable!(),
        };
        Dtype::Numerical(compressed)
    }

    /// Performs TextDiff operation as defined in Hatanaka compression method 
    fn text_data_recovery (&mut self, data: String) -> Dtype {
        let mut init = self.init
            .as_text()
            .unwrap();
        let l = init.len();
        let mut recovered = String::from("");
        let mut p = init
            .as_mut_str()
            .chars();
        let mut data = data
            .as_str()
            .chars();
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
        self.init = Dtype::Text(recovered.clone()); // for next time
        Dtype::Text(String::from(&recovered))
    }
    
    /// Compresses text data using Hatanaka method
    fn text_data_compression (&mut self, data: String) -> String {
        let inner = self
            .init
            .as_text()
            .unwrap();
        let mut result = String::new();
        let mut ptr_inner = inner
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
        self.init = Dtype::Text(data.replace("&"," ")
            .clone());
        result
    }
}

#[cfg(test)]
mod test {
    use super::{Kernel,Dtype};
    #[test]
    fn test_num_recovery() {
        let mut krn = Kernel::new(5);
        let init : i64 = 25065408994;
        krn.init(3, Dtype::Numerical(init))
            .unwrap();
        let data : Vec<i64> = vec![
            5918760,
            92440,
            -240,
            -320,
            -160,
            -580,
            360,
            -1380,
            220,
            -140,
        ];
        let expected : Vec<i64> = vec![
            25071327754,
            25077338954,
            25083442354,
            25089637634,
            25095924634,
            25102302774,
            25108772414,
            25115332174,
            25121982274,
            25128722574,
        ];
        for i in 0..data.len() {
            let recovered = krn.recover(Dtype::Numerical(data[i]))
                .unwrap()
                    .as_numerical()
                    .unwrap();
            assert_eq!(recovered, expected[i]);
        }
        // test re-init
        let init : i64 = 24701300559;
        krn.init(3, Dtype::Numerical(init))
            .unwrap();
        let data : Vec<i64> = vec![
            -19542118,
            29235,
            -38,
            1592,
            -931,
            645,
            1001,
            -1038,
            2198,
            -2679,
            2804,
            -892,
        ];
        let expected : Vec<i64> = vec![
            24681758441,
            24662245558,
            24642761872,
            24623308975,
            24603885936,
            24584493400,
            24565132368,
            24545801802,
            24526503900,
            24507235983,
            24488000855,
            24468797624,
        ];
        for i in 0..data.len() {
            let recovered = krn.recover(Dtype::Numerical(data[i]))
                .unwrap()
                    .as_numerical()
                    .unwrap();
            assert_eq!(recovered, expected[i]);
        }
    }
    #[test]
    fn test_text_recovery() {
        let init = "ABCDEFG 12 000 33 XXACQmpLf";
        let mut krn = Kernel::new(5);
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
        krn.init(3, Dtype::Text(String::from(init)))
            .unwrap();
        for i in 0..masks.len() {
            let mask = masks[i];
            let result = krn.recover(Dtype::Text(String::from(mask)))
                .unwrap()
                    .as_text()
                    .unwrap();
            assert_eq!(result, String::from(expected[i]));
        }
        // test re-init
        let init = " 2200 123      G 07G08G09G   XX XX";
        krn.init(3, Dtype::Text(String::from(init)))
            .unwrap();
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
            let result = krn.recover(Dtype::Text(String::from(mask)))
                .unwrap()
                    .as_text()
                    .unwrap();
            assert_eq!(result, String::from(expected[i]));
        }
    }
    #[test]
    fn test_numerical_compression() {
        let init : i64 = 25065408994;
        let data : Vec<i64> = vec![
            25071327754,
            25077338954,
            25083442354,
            25089637634,
            25095924634,
            25102302774,
            25108772414,
            25115332174,
            25121982274,
            25128722574,
        ];
        let expected : Vec<i64> = vec![
            5918760,
            92440,
            -240,
            -320,
            -160,
            -580,
            360,
            -1380,
            220,
            -140,
        ];
        let mut krn = Kernel::new(5);
        krn
            .init(3, Dtype::Numerical(init))
            .unwrap();
        for i in 0..data.len() {
            let result = krn
                .compress(Dtype::Numerical(data[i]))
                .unwrap()
                .as_numerical()
                .unwrap();
            assert_eq!(result, expected[i]);
        }
    }
    #[test]
    fn test_text_compression() {
        let mut krn = Kernel::new(5);
        let init = "Default Phrase 1234";
        krn
            .init(0, Dtype::Text(init.to_string()))
            .unwrap();
        let to_compress = "DEfault Phrase 1234";
        let result = krn
            .compress(Dtype::Text(to_compress.to_string()))
            .unwrap()
            .as_text()
            .unwrap();
        assert_eq!(result, " E                 ");
        
        let to_compress = "DEfault Phrase 1234";
        let result = krn
            .compress(Dtype::Text(to_compress.to_string()))
            .unwrap()
            .as_text()
            .unwrap();
        assert_eq!(result, "                   ");
        
        let to_compress = "DEFault Phrase 1234";
        let result = krn
            .compress(Dtype::Text(to_compress.to_string()))
            .unwrap()
            .as_text()
            .unwrap();
        assert_eq!(result, "  F                ");
        
        let to_compress = "DEFault Phrase 1234  ";
        let result = krn
            .compress(Dtype::Text(to_compress.to_string()))
            .unwrap()
            .as_text()
            .unwrap();
        assert_eq!(result, "                     ");
        
        let to_compress = "&EFault Phrase 1234  ";
        let result = krn
            .compress(Dtype::Text(to_compress.to_string()))
            .unwrap()
            .as_text()
            .unwrap();
        assert_eq!(result, "                     ");
        
        let to_compress = "__&abcd Phrase 1222    ";
        let result = krn
            .compress(Dtype::Text(to_compress.to_string()))
            .unwrap()
            .as_text()
            .unwrap();
        assert_eq!(result, "__  bcd          22    ");
    }
}
