//! NAVI report is include on navigation compatible contexts

mod sky;
use sky::SkyView;

struct SkyView {
    view: BTreeMap<Epoch, (f64, f64)>,
}

impl SkyView {
    pub fn from_ctx(ctx: &Context) -> Self {}
}

/// Navigation report
pub struct NAVIReport {
    pub sky: HashMap<Constellation, SkyView>,
}

impl NAVIReport {
    pub fn new(ctx: &Context) -> Self {}
}
