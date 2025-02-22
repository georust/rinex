use crate::bias::description::Description as BiasDescription;

/// Description block is Document Type dependent
#[derive(Debug, Clone)]
pub enum Description {
    BiasDescription(BiasDescription),
}

impl Description {
    pub fn bias_description(&self) -> Option<&BiasDescription> {
        match self {
            Self::BiasDescription(d) => Some(d),
        }
    }
}
