use thiserror::Error;
use std::collections::VecDeque;

#[derive(Error, Debug)]
pub enum Error {
    #[error("maximal compression order is 7")]
    MaximalCompressionOrder,
    #[error("order cannot be greater than {0}")]
    OrderTooBig(usize),
}

/// `NumDiff` is a structure to compress    
/// or recover data using recursive defferential     
/// equations as defined by Y. Hatanaka.   
#[derive(Debug, Clone)]
pub struct NumDiff {
    m: usize,
    order: usize,
    history: VecDeque<i64>,
}

impl NumDiff {
    /// Builds a new kernel structure.    
    /// max: maximal Hatanaka order for this kernel to ever support.
    /// We only support max <= 7.
    /// For information, m = 5 is hardcoded in `CRN2RNX` and is a good compromise
    pub fn new (max: usize) -> Result<Self, Error> {
        if max > 7 {
            return Err(Error::MaximalCompressionOrder)
        }
        let mut null = VecDeque::with_capacity(max);
        null.iter_mut().map(|x| *x = 0_i64);
        Ok(Self {
            m: 0,
            order: max,
            history: null,
        })
    }

    /// Initializes or reinitializes Self.
    pub fn init (&mut self, order: usize, data: i64) -> Result<(), Error> { 
        if order > self.history.len() {
            return Err(Error::OrderTooBig(self.history.len()))
        }
        self.order = order;
        self.m = 0;
        self.rotate_history(data);
        Ok(())
    }

    fn rotate_history (&mut self, data: i64) {
        let _ = self.history.pop_front();
        self.history.rotate_right(1);
        self.history[0] = data;
    }

    /// Decompresses given data 
    pub fn decompress (&mut self, data: i64) -> i64 {
        self.m += 1;
        self.m = std::cmp::min(self.m, self.order); // restraint
        self.rotate_history(data);
        let x = &self.history ;
        match self.m {
            1 => x[0],
            2 => x[0] - x[1],
            3 => x[0] - 2*x[1] + x[2],
            4 => x[0] - 3*x[1] + 3*x[2] - x[3],
            5 => x[0] - 4*x[1] + 6*x[2] - 4*x[3] + x[4],
            6 => x[0] - 5*x[1] + 10*x[2] -10*x[3] + 5*x[4] - x[5],
            7 => x[0] - 6*x[1] + 15*x[2] -20*x[3] +15*x[4] -6*x[5] + x[6],
            _ => unimplemented!("maximal compression order: 7"),
        }
    }
    
    /// Compresses given data 
    pub fn compress (&mut self, data: i64) -> i64 {
        self.m += 1;
        self.m = std::cmp::min(self.m, self.order); // restraint 
        self.rotate_history(data);
        let x = &self.history ;
        match self.m {
            1 => x[0],
            2 => x[0] - x[1],
            3 => x[0] - 2*x[1] + x[2],
            4 => x[0] - 3*x[1] + 3*x[2] - x[3],
            5 => x[0] - 4*x[1] + 6*x[2] - 4*x[3] + x[4],
            6 => x[0] - 5*x[1] + 10*x[2] -10*x[3] + 5*x[4] - x[5],
            7 => x[0] - 6*x[1] + 15*x[2] -20*x[3] +15*x[4] -6*x[5] + x[6],
            _ => unimplemented!("maximal compression order: 7"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_decompression() {
        let mut diff = NumDiff::new(5)
            .unwrap();
        let init : i64 = 25065408994;
        diff.init(3, init)
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
            let recovered = diff.compress(data[i]);
            assert_eq!(recovered, expected[i]);
        }
        // test re-init
        let init : i64 = 24701300559;
        diff.init(3, init)
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
            let recovered = diff.decompress(data[i]);
            assert_eq!(recovered, expected[i]);
        }
    }
    #[test]
    fn test_compression() {
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
        let mut diff = NumDiff::new(5)
            .unwrap();
        diff.init(3, init)
            .unwrap();
        for i in 0..data.len() {
            let result = diff.compress(data[i]);
            assert_eq!(result, expected[i]);
        }
    }
}
