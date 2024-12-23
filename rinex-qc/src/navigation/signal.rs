use crate::context::{meta::MetaData, QcContext};

use rinex::prelude::{obs::SignalObservation, Epoch};

pub struct SignalSource<'a> {
    pub t: Option<Epoch>,
    pub pending: Vec<SignalObservation>,
    pub next: Option<SignalObservation>,
    pub iter: Box<dyn Iterator<Item = (Epoch, &'a SignalObservation)> + 'a>,
}

impl<'a> SignalSource<'a> {
    pub fn collect_epoch(&mut self) -> Option<(Epoch, &[SignalObservation])> {
        self.pending.clear();

        if let Some(next) = &self.next {
            self.pending.push(next.clone());
        }

        loop {
            if let Some((t, signal)) = self.iter.next() {
                if let Some(current_t) = self.t {
                    if t > current_t {
                        // new epoch: store pending signal and exit
                        self.next = Some(signal.clone());
                        self.t = Some(t);
                        break; // abort
                    } else {
                        self.pending.push(signal.clone());
                    }
                } else {
                    self.t = Some(t);
                    self.pending.push(signal.clone());
                }
            } else {
                return None;
            }
        }

        Some((self.t?, &self.pending))
    }
}

impl QcContext {
    /// Obtain [SignalSource] from this [QcContext] for this particular [MetaData].
    pub fn signal_source(&self, meta: &MetaData) -> Option<SignalSource> {
        let rinex = self.obs_dataset.get(meta)?;
        let iter = rinex.signal_observations_sampling_ok_iter();

        Some(SignalSource {
            iter,
            t: None,
            next: None,
            pending: Vec::with_capacity(128),
        })
    }
}
