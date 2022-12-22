use crate::prelude::*;

pub enum AverageType {
    MovingAverage,
    Exponential,
    Cummulative,
}

pub struct Averager {
    buffer: Vec<f64>,
    next_epoch: Option<Epoch>,
    window: Duration,
    avgtype: AverageType,
}

impl Averager {
    pub fn new(window: Duration) -> Self {
        Self {
            buffer: Vec::new(),
            next_epoch: None,
            window,
        }
    }
    pub fn average(&mut self, data: (f64, Epoch)) -> Option<f64> {
        match self.avgtype {
            AverageType::MovingAverage => self.moving_average(data),
            AverageType::Exponential => self.moving_average(data),
            AverageType::Cummulative => self.moving_average(data),
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

#[cfg(test)]
mod test {
    #[test]
    fn test_moving_average() {
     
    }
}
