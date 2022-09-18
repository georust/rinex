pub enum Fsm {
    EpochDescriptor,
    Body,
}

impl Default for Fsm {
    fn default() -> Self {
        Self::EpochDescriptor
    }
}

impl Fsm {
    /// Resets Finite State Machine
    fn reset (&mut self) {
        *self = Self::default()
    }
}

