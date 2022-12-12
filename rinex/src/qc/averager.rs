use crate::prelude::*;

pub struct Averager {
    buffer: Vec<f64>,
    prev_epoch: Option<Epoch>,
    pub time_window: Duration,
}

impl Averager {
    pub fn moving_average(&mut self, data: (f64, Epoch)) -> Option<f64> {
        if let Some(mut p) = self.prev_epoch {
            if p + self.time_window <= data.1 {
                let mut avg = data.0; 
                for b in &self.buffer {
                    avg += b;
                }
                avg /= self.buffer.len() as f64;
                self.buffer.clear();
                p = data.1;
                Some(avg)
            } else {
                self.buffer.push(data.0);
                None
            }
        } else { // 1st call
            self.prev_epoch = Some(data.1);
            self.buffer.push(data.0);
            None
        }
    }
}
