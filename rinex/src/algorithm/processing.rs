/*use crate::prelude::*;

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
}*/

/*
pub trait Processing<A> {
	/// averages this subset with desired method
    fn average(&self) -> A;
    /// computes 1st order derivative of this subset
    fn derivative(&self) -> A;
    /// computes nth order derivative of this subset
    fn derivative_nth(&self, order: u8) -> A;
    /// applies smoothing to this subset
    fn smoothing(&self) -> A;
    fn smoothing_mut(&mut self);
	/// Interpolates self to macth the given time serie
	fn interpolate(&self, serie: hifitime::TimeSeries) -> A;
	fn interpolate_mut(&mut self, serie: hifitime::TimeSeries);
}
*/
