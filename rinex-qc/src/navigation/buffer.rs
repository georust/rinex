use rinex::prelude::Epoch;

pub trait TemporalIndexing {
    fn epoch(&self) -> &Epoch;
    fn set_epoch(&mut self, t: Epoch);
}

pub struct Buffer<K: TemporalIndexing, T> {
    inner: Vec<(K, T)>,
}

impl<K: TemporalIndexing, T> Buffer<K, T> {

    /// Creates a new [Buffer] with "size" prealloc
    pub fn new(size: usize) -> Self {
        Self {
            inner: Vec::with_capacity(size),
        }
    }

    /// Returns true if requested [Epoch] is contained within [Buffer]
    pub fn contains(&self, x: &Epoch) -> Option<&T> {
        self.inner
            .iter()
            .filter(|(x_i, _)| x_i.epoch() == x)
            .reduce(|k, _| k)
            .map(|(_, y)| y)
    }

    pub fn push(&mut self, k: K, y: T) {
        self.inner.push((k, y));
    }

    // pub fn interpolation_feasible(&self, t: Epoch, order: usize) -> bool {
    //     if self.inner.len() < order + 2 {
    //         return false;
    //     }
    //     let n = (order + 1) / 2;
    //     if let Some(index) = self.central_index(t) {
    //         index >= n && index <= self.inner.len() - n
    //     } else {
    //         false
    //     }
    // }

    // fn central_t(&self, t: Epoch) -> Option<&Epoch> {
    //     self.inner
    //         .iter()
    //         .min_by_key(|(k_i, _)| (*k_i.epoch() - t).abs())
    //         .and_then(|(k, _)| Some(k.epoch()))
    // }

    // fn central_index(&self, t: Epoch) -> Option<usize> {
    //     let t_c = self.central_t(t)?;
    //     self.inner.iter().position(|(k_i, _)| k_i.epoch() == t_c)
    // }

    // /// Will panic if .feasible() is not respected
    // pub fn interpolate<F: Fn(&[(Epoch, T)]) -> T>(&self, t: Epoch, order: usize, interp: F) -> T {
    //     let center = self.central_index(t).unwrap();
    //     let buf = &self.inner[center - (order + 1) / 2..center + (order + 1) / 2];
    //     interp(buf)
    // }
}
