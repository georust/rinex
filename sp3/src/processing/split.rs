use crate::prelude::{Duration, Epoch, SP3};
use qc_traits::Split;

impl Split for SP3 {
    fn split(&self, t: Epoch) -> (Self, Self)
    where
        Self: Sized,
    {
        let mut lhs = self.clone();
        let rhs = lhs.split_mut(t);
        (lhs, rhs)
    }

    fn split_mut(&mut self, _t: Epoch) -> Self {
        Self::default()
    }

    fn split_even_dt(&self, _dt: Duration) -> Vec<Self>
    where
        Self: Sized,
    {
        Default::default()
    }
}
