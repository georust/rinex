use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum AverageType {
    Cumulative,
    Moving(Duration),
    Exponential(f64),
}

pub struct Averager {
    buffer: Vec<f64>,
    next_epoch: Option<Epoch>,
    window: Duration,
}

impl Averager {
    pub fn new(window: Duration) -> Self {
        Self {
            buffer: Vec::new(),
            next_epoch: None,
            window,
        }
    }
    pub fn moving_average(&mut self, data: (f64, Epoch)) -> Option<f64> {
        self.buffer.push(data.0);
        if self.next_epoch.is_none() {
            self.next_epoch = Some(data.1 + self.window);
        }
        if let Some(next_epoch) = self.next_epoch {
            if data.1 >= next_epoch {
                self.next_epoch = Some(data.1 + self.window);
                let mut avg = 0.0_f64;
                for b in &self.buffer {
                    avg += b;
                }
                let ret = avg / self.buffer.len() as f64;
                self.buffer.clear();
                return Some(ret);
            }
        }
        None
    }
}

pub trait Processing {
	/// Applies desired averaging method to selected data set
    fn average(&self, avg: AverageType, target: TargetItem) -> BTreeMap<Epoch, f64>;
	/// Interpolates self to macth the given epoch axis
	fn interpolate(&self, epoch: Vec<Epoch>) -> Self;
	/// Mutable implementation see [Processing::interpolate] 
	fn interpolate_mut(&self, epoch: Vec<Epoch>);
}
