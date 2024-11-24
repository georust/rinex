use crate::prelude::{Duration, Epoch, SP3, SV};
use qc_traits::Split;

impl Split for SP3 {
    fn split(&self, t: Epoch) -> (Self, Self)
    where
        Self: Sized,
    {
        (Self::default(), Self::default())
    }

    fn split_mut(&mut self, t: Epoch) -> Self {
        let mut ret = Self::default();

        let mut init_sv = self.sv.clone();
        let mut sv_drop = Vec::<SV>::new();
        let mut sv_preserved = Vec::<SV>::new();

        let collected = self
            .data
            .iter()
            .filter_map(|(k, v)| {
                if k.epoch > t {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();

        self.data.retain(|k, _| k.epoch <= t);

        // browse remaining data: adapt self & returned
        for (_, v) in self.data.iter() {}

        ret.data = collected;
        ret
    }

    fn split_even_dt(&self, dt: Duration) -> Vec<Self>
    where
        Self: Sized,
    {
        Default::default()
    }
}
