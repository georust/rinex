pub mod cggtts;
pub mod ppp;

use crate::prelude::{Render, html, Markup};

pub struct QcNavPostSolutions {
    pub cggtts: Option<QcNavPostCggttsSolutions>,
    pub ppp: Option<QcNavPostPPPSolutions>,
}

impl Render for QcNavPostSolutions {
    fn render(&self) -> Markup {
        html! {
        }
    }
}
