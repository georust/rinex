pub mod cggtts;
pub mod ppp;

use crate::prelude::{html, Markup, Render};

pub struct QcNavPostSolutions {
    pub cggtts: Option<QcNavPostCggttsSolutions>,
    pub ppp: Option<QcNavPostPPPSolutions>,
}

impl Render for QcNavPostSolutions {
    fn render(&self) -> Markup {
        html! {}
    }
}
