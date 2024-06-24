//! Shared analysis, that may apply to several [ProductType]
use rinex::prelude::{Duration, Epoch};

mod sampling;
pub(crate) use sampling::SamplingReport;

use maud::{html, Markup, Render};

pub(crate) struct EpochSlider {
    start: Epoch,
    end: Epoch,
    dt: Duration,
}

impl EpochSlider {
    pub fn new(start: Epoch, end: Epoch, dt: Duration) -> Self {
        Self { start, end, dt }
    }
}

impl Render for EpochSlider {
    fn render(&self) -> Markup {
        html! {
            table class="table is-bordered" {

            }
        }
    }
}
