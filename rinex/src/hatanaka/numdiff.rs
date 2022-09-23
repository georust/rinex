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
    pub const MAX_COMPRESSION_ORDER: usize = 6;
    /// Builds a new kernel structure.    
    /// max: maximal Hatanaka order for this kernel to ever support.
    /// We only support max <= Self::MAX_COMPRESSION_ORDER.
    /// For information, m = 5 is hardcoded in `CRN2RNX` and is a good compromise
    pub fn new (max: usize) -> Result<Self, Error> {
        if max > Self::MAX_COMPRESSION_ORDER {
            return Err(Error::MaximalCompressionOrder)
        }
        let mut null = VecDeque::with_capacity(max);
        for _ in 0..max {
            null.push_back(0_i64);
        }
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
        self.history.pop_back();
        self.history.push_front(data);
    }

    /// Decompresses given data 
    pub fn decompress (&mut self, data: i64) -> i64 {
        self.m += 1;
        self.m = std::cmp::min(self.m, self.order); // restraint
        let x = &self.history ;
        let result: i64 = match self.m {
            1 => data + x[0],
            2 => data + 2*x[0] - x[1],
            3 => data + 3*x[0] - 3*x[1] + x[2],
            4 => data + 4*x[0] - 6*x[1] + 4*x[2] - x[3],
            5 => data + 5*x[0] - 10*x[1] +10*x[2] - 5*x[3] + x[4],
            6 => data + 6*x[0] - 15*x[1] +20*x[2] -15*x[3] +6*x[4] - x[5],
            _ => unimplemented!("maximal compression order: {}", Self::MAX_COMPRESSION_ORDER),
        };
        self.rotate_history(result);
        result 
    }
    
    /// Compresses given data 
    pub fn compress (&mut self, data: i64) -> i64 {
        self.m += 1;
        self.m = std::cmp::min(self.m, self.order); // restraint 
        self.rotate_history(data);
        let x = &self.history ;
        match self.m {
            1 => x[0] - x[1],
            2 => x[0] - 2*x[1] + x[2],
            3 => x[0] - 3*x[1] + 3*x[2] - x[3],
            4 => x[0] - 4*x[1] + 6*x[2] - 4*x[3] + x[4],
            5 => x[0] - 5*x[1] + 10*x[2] -10*x[3] + 5*x[4] - x[5],
            6 => x[0] - 6*x[1] + 15*x[2] -20*x[3] +15*x[4] -6*x[5] + x[6],
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
        diff.init(3, 25065408994)
            .unwrap();
        assert_eq!(diff.decompress(5918760),25071327754);
        assert_eq!(diff.decompress(92440),25077338954);
        assert_eq!(diff.decompress(-240),25083442354);
        assert_eq!(diff.decompress(-320),25089637634);
        assert_eq!(diff.decompress(-160),25095924634);
        assert_eq!(diff.decompress(-580),25102302774);
        assert_eq!(diff.decompress(360),25108772414);
        assert_eq!(diff.decompress(-1380),25115332174);
        assert_eq!(diff.decompress(220),25121982274);
        assert_eq!(diff.decompress(-140),25128722574);

        // test re-init
        diff.init(3, 24701300559)
            .unwrap();
        assert_eq!(diff.decompress(-19542118),24681758441);
        assert_eq!(diff.decompress(29235),24662245558);
        assert_eq!(diff.decompress(-38),24642761872);
        assert_eq!(diff.decompress(1592),24623308975);
        assert_eq!(diff.decompress(-931),24603885936);
        assert_eq!(diff.decompress(645),24584493400);
        assert_eq!(diff.decompress(1001),24565132368);
        assert_eq!(diff.decompress(-1038),24545801802);
        assert_eq!(diff.decompress(2198),24526503900,);
        assert_eq!(diff.decompress(-2679),24507235983);
        assert_eq!(diff.decompress(2804),24488000855);
        assert_eq!(diff.decompress(-892),24468797624);
    }
    #[test]
    fn test_compression() {
        let mut diff = NumDiff::new(5)
            .unwrap();
        let init : i64 = 25065408994;
        diff.init(3, init)
            .unwrap();
        assert_eq!(diff.compress(25071327754), 5918760);
        assert_eq!(diff.compress(25077338954), 92440);
        assert_eq!(diff.compress(25083442354),-240);
        assert_eq!(diff.compress(25089637634),-320);
        assert_eq!(diff.compress(25095924634),-160);
        assert_eq!(diff.compress(25102302774), -580);
        assert_eq!(diff.compress(25108772414), 360);
        assert_eq!(diff.compress(25115332174),-1380);
        assert_eq!(diff.compress(25121982274), 220);
        assert_eq!(diff.compress(25128722574),-140);
        /*
        let init : i64 = 126298057858;
        diff.init(3, init)
            .unwrap();
        assert_eq!(diff.compress(25071327754), 5918760);
        assert_eq!(diff.compress(25077338954), 92440);
        assert_eq!(diff.compress(25083442354),-240);
        assert_eq!(diff.compress(25089637634),-320);
        assert_eq!(diff.compress(25095924634),-160);
        assert_eq!(diff.compress(25102302774), -580);
        assert_eq!(diff.compress(25108772414), 360);
        assert_eq!(diff.compress(25115332174),-1380);
        assert_eq!(diff.compress(25121982274), 220);
        assert_eq!(diff.compress(25128722574),-140);
        */
    }
}
