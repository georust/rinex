use super::Plot; 

pub struct Context {
    nb_plots: usize,
    plot: Plot, 
}

impl Context {
    pub fn new() -> Self {
        Self {
            nb_plots: 0,
            plot: Plot::new(),
        }
    }
}
