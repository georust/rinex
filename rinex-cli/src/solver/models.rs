#[derive(Default, Debug, Copy, Clone)]
pub struct Models {
    pub tropo: Option<TropoModel>,
    pub iono: Option<IonoModel>,
    pub tgd: Option<TGDModel>,
}

#[derive(Debug, Copy, Clone)]
pub struct TropoModel {}

#[derive(Debug, Copy, Clone)]
pub struct IonoModel {}

#[derive(Debug, Copy, Clone)]
pub struct TGDModel {}
