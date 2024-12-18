use rinex::prelude::Epoch;

pub struct Buffer<T> {
    pub inner: Vec<(Epoch, T)>,
}

impl<T> Buffer<T> {
    pub fn new(size: usize) -> Self {
        Self {
            inner: Vec::with_capacity(size),
        }
    }

    pub fn contains(&self, x: &Epoch) -> Option<&T> {
        self.inner
            .iter()
            .filter(|(x_i, _)| x_i == x)
            .reduce(|k, _| k)
            .map(|(_, y)| y)
    }

    pub fn push(&mut self, x: Epoch, y: T) {
        self.inner.push((x, y));
    }

    pub fn feasible(&self, t: Epoch, order: usize) -> bool {
        if self.inner.len() < order + 2 {
            return false;
        }
        let n = (order + 1) / 2;
        if let Some(index) = self.central_index(t) {
            index >= n && index <= self.inner.len() - n
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
    pub fn interpolate<F: Fn(&[(Epoch, T)]) -> T>(&self, t: Epoch, order: usize, interp: F) -> T {
        let center = self.central_index(t).unwrap();
        let buf = &self.inner[center - (order + 1) / 2..center + (order + 1) / 2];
        interp(buf)
    }
}
