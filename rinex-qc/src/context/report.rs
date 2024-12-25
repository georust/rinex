use crate::{prelude::QcContext, report::QcReport};

impl QcContext {
    /// Generates a [QcReport] that you can then render
    /// in the desired format. The reported content is heavily dependent
    /// on the current [QcContext].
    pub fn generate_report(&self) -> QcReport {
        QcReport::new(&self)
    }
}
