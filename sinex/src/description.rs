use crate::bias;
use crate::troposphere;

/// Description block is Document Type dependent
pub enum Description {
    BiasDescription(bias::description::Description),
    TropoDescription(troposphere::description::Description),
}

impl Description {
    pub fn bias_description (&self) -> Option<&bias::description::Description> {
        match self {
            Self::BiasDescription(d) => Some(d),
            _ => None,
        }
    }
    pub fn tropo_description (&self) -> Option<&troposhere::description::Description> {
        match self {
            Self::TroposphereDescription(d) => Some(d),
            _ => None,
        }
    }
}
