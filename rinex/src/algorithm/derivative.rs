use hifitime::Epoch;

pub struct Derivative {
    order: usize,
}

/*
 * Derivative of an number of array sorted by chronological Epoch
 */
pub(crate) fn derivative(input: Vec<(Epoch, f64)>, buf: &mut Vec<(Epoch, f64)>) {
    let mut prev: Option<(Epoch, f64)> = None;
    for (e, value) in input {
        if let Some((prev_e, prev_v)) = prev {
            let dt = e - prev_e;
            let dy = (value - prev_v) / dt.to_seconds();
            buf.push((e, dy));
        }
        prev = Some((e, value));
    }
}

/*
 * Derivative^2 of an number of array sorted by chronological Epoch
 */
impl Derivative {
    pub fn new(order: usize) -> Self {
        Self { order }
    }
    pub fn eval(&self, input: Vec<(Epoch, f64)>) -> Vec<(Epoch, f64)> {
        let mut buf: Vec<(Epoch, f64)> = Vec::with_capacity(input.len());
        derivative(input, &mut buf);
        //for i in 1..self.order {
        //    derivative(&ret, &mut ret);
        //}
        //ret
        buf
    }
}
