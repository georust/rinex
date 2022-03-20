//! hatanaka.rs   
//! structures and macros suites for
//! RINEX OBS files compression and decompression.   
use thiserror::Error;

#[derive(Error, Debug)]
/// Hatanaka Kernel, compression
/// and decompression related errors
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
    /// Numerical is intended to be used    
    /// for observation data and clock offsets
    Numerical(i64),
    /// Text represents epoch descriptors,   
    /// GNSS descriptors,    
    /// observation flags..
    Text(String),
}

impl Default for Dtype {
    fn default() -> Dtype { Dtype::Numerical(0) }
 }

impl Dtype {
    pub fn as_numerical (&self) -> Option<i64> {
        match self {
            Dtype::Numerical(n) => Some(n.clone()),
            _ => None,
        }
    }
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
pub struct Kernel {
    /// internal counter
    n: usize,
    /// compression order
    order: usize,
    /// kernel initializer
    init: Dtype,
    /// state vector 
    state: Vec<i64>,
    /// previous state vector 
    p_state: Vec<i64>,
}

/// Fills given i64 vector with '0'
fn zeros (array : &mut Vec<i64>) {
    for i in 0..array.len() { array[i] = 0 }
}

impl Kernel {
    /// Builds a new kernel structure.    
    /// m: maximal Hatanaka order for this kernel to ever support,    
    ///    m=5 is hardcoded in CRN2RNX official tool
    pub fn new (m: usize) -> Kernel {
        let mut state : Vec<i64> = Vec::with_capacity(m+1);
        for _ in 0..m+1 { state.push(0) }
        Kernel {
            n: 0,
            order: 0,
            init: Dtype::default(),
            state: state.clone(),
            p_state: state.clone(),
        }
    }

    /// Initializes kernel.   
    /// order: compression order   
    /// data: kernel initializer
    pub fn init (&mut self, order: usize, data: Dtype) -> Result<(), Error> { 
        if order > self.state.len() {
            return Err(Error::OrderTooBig(self.state.len()))
        }
        // reset
        self.n = 0;
        zeros(&mut self.state);
        zeros(&mut self.p_state);
        // init
        self.init = data.clone();
        self.p_state[0] = data.as_numerical().unwrap_or(0);
        self.order = order;
        Ok(())
    }

    /// Recovers data by computing
    /// recursive differential equation
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
    /// Runs Differential Equation
    /// as defined in Hatanaka compression method
    fn numerical_data_recovery (&mut self, data: i64) -> Dtype {
        self.n += 1;
        self.n = std::cmp::min(self.n, self.order);
        zeros(&mut self.state);
        self.state[self.n] = data;
        for index in (0..self.n).rev() {
            self.state[index] = 
                self.state[index+1] 
                    + self.p_state[index] 
        }
        self.p_state = self.state.clone();
        Dtype::Numerical(self.state[0])
    }
    /// Text is very simple
    fn text_data_recovery (&mut self, data: String) -> Dtype {
        let mut init = self.init
            .as_text()
            .unwrap();
        let l = init.len();
        let mut recovered = String::from("");
        let mut p = init.as_mut_str().chars();
        let mut data = data.as_str().chars();
        for _ in 0..l {
            let next_c = p.next().unwrap();
            if let Some(c) = data.next() {
                if c == '&' {
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
                if c == '&' {
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
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    /// Tests numerical data recovery    
    /// through Hatanaka decompression.   
    /// Tests data come from an official CRINEX file and
    /// official CRX2RNX decompression tool
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
    /// Tests Hatanaka Text Recovery algorithm
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
}
