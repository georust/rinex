//! NAVI report is include on navigation compatible contexts
use crate::QcContext;
use qc_traits::html::*;
use rinex::prelude::{Constellation, Epoch};
use std::collections::{BTreeMap, HashMap};

struct SkyView {
    view: BTreeMap<Epoch, (f64, f64)>,
}

impl SkyView {
    pub fn from_ctx(ctx: &QcContext) -> Self {
        Self {
            view: BTreeMap::new(),
        }
    }
}

/// Navigation report
pub struct NAVIReport {
    pub sky: HashMap<Constellation, SkyView>,
}

impl NAVIReport {
    pub fn new(ctx: &QcContext) -> Self {
        Self {
            sky: HashMap::new(),
        }
    }
}

impl RenderHtml for NAVIReport {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {}
    }
}
