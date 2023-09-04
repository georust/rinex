use hifitime::{Duration, Epoch};

fn moving_average<T: std::default::Default>(
    data: Vec<(Epoch, T)>,
    window: Duration,
) -> Vec<(Epoch, T)> {
    let mut acc = T::default();
    let mut prev_epoch: Option<Epoch> = None;
    let mut ret: Vec<(Epoch, T)> = Vec::new();
    for (epoch, value) in data {}
    ret
}

#[derive(Debug, Clone, Copy)]
pub enum Averager {
    MovingAverage(Duration),
}

impl Default for Averager {
    fn default() -> Self {
        Self::MovingAverage(Duration::from_seconds(600.0_f64))
    }
}

impl Averager {
    pub fn mov(window: Duration) -> Self {
        Self::MovingAverage(window)
    }
    pub fn eval<T: std::default::Default>(&self, input: Vec<(Epoch, T)>) -> Vec<(Epoch, T)> {
        match self {
            Self::MovingAverage(dt) => moving_average(input, *dt),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_moving_average() {
        let mov = Averager::mov(Duration::from_seconds(10.0_f64));
    }
}
