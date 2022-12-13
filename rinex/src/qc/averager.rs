use crate::prelude::*;

pub struct Averager {
    buffer: Vec<f64>,
    prev_avg: Option<Epoch>,
    window: Duration,
}

impl Averager {
    pub fn new(window: Duration) -> Self {
        Self {
            buffer: Vec::new(),
            prev_avg: None,
            window,
        }
    }
    pub fn moving_average(&mut self, data: (f64, Epoch)) -> Option<f64> {
        self.buffer.push(data.0);
        if self.prev_avg.is_none() {
            self.prev_avg = Some(data.1);
        }
        if let Some(mut prev_avg) = self.prev_avg {
            if data.1 >= prev_avg + self.window {
                prev_avg = data.1;
                let mut avg = 0.0_f64;
                for b in &self.buffer {
                    avg += b;
                }
                self.buffer.clear();
                return Some(avg / self.buffer.len() as f64);
            }
        }
        None
    }
}
