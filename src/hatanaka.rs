//! hatanaka.rs   
//! structures and macros suites for
//! RINEX OBS files compression and decompression.   
use thiserror::Error;

/* notes sur les preicions numeriques
    RINEX < 3
        OBS = F14.3
        CLOCKS = F12.9

    RINEX > 2
        OBS = F14.3 inchange'
        CLOCKS = F15.12 increased

*/

#[derive(Error, Debug)]
/// Hatanaka Kernel, compression
/// and decompression related errors
pub enum Error {
    #[error("order cannot be greater than {0}")]
    OrderTooBig(usize),
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

impl Dtype {
    fn as_numerical (&self) -> Option<i64> {
        match self {
            Dtype::Numerical(n) => Some(n.clone()),
            _ => None,
        }
    }
    fn as_text (&self) -> Option<&str> {
        match self {
            Dtype::Text(s) => Some(&s),
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
        for i in 0..m+1 { state.push(0) }
        Kernel {
            n: 0,
            order: 0,
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
        self.p_state[0]
            = data.as_numerical().unwrap_or(0);
        self.order = order;
        Ok(())
    }

    /// Recovers data by computing
    /// recursive differential equation
    pub fn recover (&mut self, data: Dtype) -> Dtype {
        match data {
            Dtype::Numerical(data) => self.numerical_data_recovery(data),
            Dtype::Text(data) => self.text_data_recovery(data),
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
    /// TextDiff is simplistic
    fn text_data_recovery (&mut self, data: String) -> Dtype {
        Dtype::Numerical(0)    
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
                .as_numerical()
                .unwrap();
            assert_eq!(recovered, expected[i]);
        }
    }
}
