use hifitime::Epoch;

pub struct Derivative {
    order: usize,
}

/*
 * Derivative of an number of array sorted by chronological Epoch
 */
pub(crate) fn derivative<T: std::ops::Sub<Output = T> + std::marker::Copy>(
    input: Vec<(Epoch, T)>,
    buf: &mut Vec<(Epoch, T)>,
) {
    let mut prev: Option<(Epoch, T)> = None;
    for (e, value) in input {
        if let Some((prev_e, prev_v)) = prev {
            let dt = e - prev_e;
            let dy = value - prev_v;
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
    pub fn eval<T: std::ops::Sub<Output = T> + std::marker::Copy>(
        input: Vec<(Epoch, T)>,
    ) -> Vec<(Epoch, T)> {
        let mut buf: Vec<(Epoch, T)> = Vec::with_capacity(input.len());
        derivative(input, &mut buf);
        //for i in 1..self.order {
        //    derivative(&ret, &mut ret);
        //}
        //ret
        buf
    }
}
