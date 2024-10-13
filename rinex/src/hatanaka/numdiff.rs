use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("maximal compression order is 7")]
    MaximalCompressionOrder,
    #[error("order cannot be greater than {0}")]
    OrderTooBig(usize),
}

/// [NumDiff] is dedicated to numerical (de-)compression, following
/// algorithm developped by Y. Hatanaka. We recommend fixing M = 5
/// in the application. You must not use M > 6 with this library or it will eventually panic!!
#[derive(Debug, Clone)]
pub struct NumDiff<const M: usize> {
    /// level/iteration counter
    m: usize,
    /// internal data history
    buf: [i64; M],
}

impl<const M: usize> NumDiff<M> {
    /// Builds a [NumDiff] structure dedicated to numerical (de-)compression
    /// with `data` initial point.    
    pub fn new(data: i64) -> Self {
        let mut buf = [0; M];
        buf[0] = data;
        Self { buf, m: 0 }
    }
    /// [NumDiff] needs to be reinit   when ???
    pub fn force_init(&mut self, data: i64) {
        self.m = 0;
        self.rotate_history(data);
    }

    /// Rotate internal buffer, take new sample into account.
    fn rotate_history(&mut self, data: i64) {
        self.buf.copy_within(0..M - 2, 1);
        self.buf[0] = data;
    }

    /// Decompresses input data point, returns recovered data point.
    pub fn decompress(&mut self, data: i64) -> i64 {
        if self.m < M - 1 {
            self.m += 1;
        }

        let new: i64 = match self.m {
            1 => data + self.buf[0],
            2 => data + 2 * self.buf[0] - self.buf[1],
            3 => data + 3 * self.buf[0] - 3 * self.buf[1] + self.buf[2],
            4 => data + 4 * self.buf[0] - 6 * self.buf[1] + 4 * self.buf[2] - self.buf[3],
            5 => {
                data + 5 * self.buf[0] - 10 * self.buf[1] + 10 * self.buf[2] - 5 * self.buf[3]
                    + self.buf[4]
            },
            6 => {
                data + 6 * self.buf[0] - 15 * self.buf[1] + 20 * self.buf[2] - 15 * self.buf[3]
                    + 6 * self.buf[4]
                    - self.buf[5]
            },
            _ => panic!("numdiff is limited to M<=6!"),
        };

        self.rotate_history(new);
        new
    }

    /// Compresses input data point, returns "compressed" data point.
    pub fn compress(&mut self, data: i64) -> i64 {
        if self.m < M - 1 {
            self.m += 1;
        }
        self.rotate_history(data);

        match self.m {
            1 => self.buf[0] - self.buf[1],
            2 => self.buf[0] - 2 * self.buf[1] + self.buf[2],
            3 => self.buf[0] - 3 * self.buf[1] + 3 * self.buf[2] - self.buf[3],
            4 => self.buf[0] - 4 * self.buf[1] + 6 * self.buf[2] - 4 * self.buf[3] + self.buf[4],
            5 => {
                self.buf[0] - 5 * self.buf[1] + 10 * self.buf[2] - 10 * self.buf[3]
                    + 5 * self.buf[4]
                    - self.buf[5]
            },
            6 => {
                self.buf[0] - 6 * self.buf[1] + 15 * self.buf[2] - 20 * self.buf[3]
                    + 15 * self.buf[4]
                    - 6 * self.buf[5]
                    + self.buf[6]
            },
            _ => panic!("numdiff is limited to M<=6"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decompression() {
        let mut diff = NumDiff::<6>::new(25065408994);
        assert_eq!(diff.decompress(5918760), 25071327754);
        assert_eq!(diff.decompress(92440), 25077338954);
        assert_eq!(diff.decompress(-240), 25083442354);
        assert_eq!(diff.decompress(-320), 25089637634);
        assert_eq!(diff.decompress(-160), 25095924634);
        assert_eq!(diff.decompress(-580), 25102302774);
        assert_eq!(diff.decompress(360), 25108772414);
        assert_eq!(diff.decompress(-1380), 25115332174);
        assert_eq!(diff.decompress(220), 25121982274);
        assert_eq!(diff.decompress(-140), 25128722574);

        // test re-init
        diff.force_init(24701300559);
        assert_eq!(diff.decompress(-19542118), 24681758441);
        assert_eq!(diff.decompress(29235), 24662245558);
        assert_eq!(diff.decompress(-38), 24642761872);
        assert_eq!(diff.decompress(1592), 24623308975);
        assert_eq!(diff.decompress(-931), 24603885936);
        assert_eq!(diff.decompress(645), 24584493400);
        assert_eq!(diff.decompress(1001), 24565132368);
        assert_eq!(diff.decompress(-1038), 24545801802);
        assert_eq!(diff.decompress(2198), 24526503900,);
        assert_eq!(diff.decompress(-2679), 24507235983);
        assert_eq!(diff.decompress(2804), 24488000855);
        assert_eq!(diff.decompress(-892), 24468797624);
    }

    #[test]
    fn test_compression() {
        let mut diff = NumDiff::<3>::new(25065408994);
        assert_eq!(diff.compress(25071327754), 5918760);
        assert_eq!(diff.compress(25077338954), 92440);
        assert_eq!(diff.compress(25083442354), -240);
        assert_eq!(diff.compress(25089637634), -320);
        assert_eq!(diff.compress(25095924634), -160);
        assert_eq!(diff.compress(25102302774), -580);
        assert_eq!(diff.compress(25108772414), 360);
        assert_eq!(diff.compress(25115332174), -1380);
        assert_eq!(diff.compress(25121982274), 220);
        assert_eq!(diff.compress(25128722574), -140);

        //TODO: test reinit

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
