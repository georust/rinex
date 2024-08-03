use gnss_rtk::prelude::Epoch;

pub struct Buffer<T> {
    order: usize,
    pub inner: Vec<(Epoch, T)>,
}

impl<T> Buffer<T> {
    pub fn new(order: usize) -> Self {
        if order % 2 == 0 {
            panic!("only odd interpolation orders currently supported!");
        }
        Self {
            order,
            inner: Vec::with_capacity(order),
        }
    }
    pub fn push(&mut self, x: Epoch, y: T) {
        self.inner.push((x, y));
    }
    pub fn feasible(&self, t: Epoch) -> bool {
        if self.inner.len() < self.order + 2 {
            return false;
        }
        if let Some(nearest) = self.central_t(t) {
            false
        } else {
            false
        }
    }
    pub fn discard(&mut self, t: Epoch, order: usize) {
        if let Some(center) = self.central_index(t) {
            if center > order + 2 {}
        }
    }
    fn central_t(&self, t: Epoch) -> Option<&Epoch> {
        self.inner
            .iter()
            .min_by_key(|(t_i, _)| (*t_i - t).abs())
            .and_then(|(t, _)| Some(t))
    }
    fn central_index(&self, t: Epoch) -> Option<usize> {
        let t_c = self.central_t(t)?;
        self.inner.iter().position(|(t_i, _)| t_i == t_c)
    }
    /// Will panic if .feasible() is not respected
    pub fn interpolate<F: Fn(&[(Epoch, T)]) -> T>(&self, t: Epoch, interp: F) -> T {
        let center = self.central_index(t).unwrap();
        let buf = &self.inner[center - (self.order + 1) / 2..center + (self.order + 1) / 2];
        interp(buf)
    }
}
