use crate::context::QcContext;

// enum Format {
//     RINEx,
//     GZipRINEx,
// }

// impl std::fmt::Display for Format {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::RINEx => f.write_str("RINEx"),
//             Self::GZipRINEx => f.write_str("RINEx + gzip"),
//         }
//     }
// }

pub struct QcNavigationSummary {
    pub agency: Option<String>,
}

impl QcNavigationSummary {
    pub fn new(ctx: &QcContext) -> Self {
        let nav_dataset = ctx.nav_dataset.as_ref().unwrap();

        Self {
            agency: nav_dataset.header.agency.clone(),
        }
    }
}
